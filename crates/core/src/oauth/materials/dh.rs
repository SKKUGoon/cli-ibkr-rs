// Minimal ASN.1/DER parser for OpenSSL-produced `DH PARAMETERS` PEM files.
//
// IBKR's portal hands out DH parameters in PEM form. The DER payload inside
// follows PKCS#3 (`DHParameter`):
//
//     DHParameter ::= SEQUENCE {
//         prime           INTEGER,
//         base            INTEGER,
//         privateValueLength INTEGER OPTIONAL
//     }
//
// We only need `prime` and `base` (a.k.a. the generator). The optional
// `privateValueLength` shows up when OpenSSL is invoked with `-2`/`-5` and is
// just a recommendation — we read past it but discard the value.
//
// A full ASN.1 library would be overkill here: PKCS#3 only uses two tags
// (SEQUENCE, INTEGER) and DER's definite-length encoding, so the parser fits
// in ~50 lines. The trade-off is that any non-trivial DER (BIT STRING,
// CONTEXT-SPECIFIC, indefinite length) would be misread as a structural
// error rather than silently mis-parsed — which is the right failure mode
// for a single-purpose loader.

use std::fs;
use std::path::Path;

use rsa::BigUint;

use crate::{IbkrError, Result};

// DER tag bytes from X.690 §8.1.2.
const DER_TAG_SEQUENCE: u8 = 0x30;
const DER_TAG_INTEGER: u8 = 0x02;
// Length-of-length flag from X.690 §8.1.3.5. When set on the first length
// byte, the lower seven bits give the *number* of subsequent big-endian
// bytes that hold the actual length.
const DER_LENGTH_LONG_FORM_FLAG: u8 = 0x80;
const DER_LENGTH_VALUE_MASK: u8 = 0x7f;

#[derive(Debug, Clone)]
pub struct DhParams {
    pub prime: BigUint,
    pub generator: BigUint,
}

pub fn read_dh_params(path: &Path) -> Result<DhParams> {
    let pem = fs::read_to_string(path).map_err(|source| IbkrError::ReadFile {
        path: path.to_path_buf(),
        source,
    })?;
    let der = pem_body(&pem).ok_or_else(|| dh_error(path, "missing DH PARAMETERS PEM body"))?;
    parse_sequence(&der).map_err(|message| dh_error(path, &message))
}

// Strips PEM armor lines and base64-decodes the rest. We don't validate the
// header label (e.g. `-----BEGIN DH PARAMETERS-----`) on purpose — if the
// caller hands us the wrong PEM, the DER parser will catch it with a clearer
// error than "label mismatch".
fn pem_body(pem: &str) -> Option<Vec<u8>> {
    let body = pem
        .lines()
        .filter(|line| !line.starts_with("-----"))
        .collect::<String>();
    base64::Engine::decode(&base64::engine::general_purpose::STANDARD, body).ok()
}

fn parse_sequence(der: &[u8]) -> std::result::Result<DhParams, String> {
    let mut pos = 0;
    // Outer SEQUENCE wrapping the two (or three) INTEGERs.
    expect_tag(der, &mut pos, DER_TAG_SEQUENCE)?;
    let len = read_len(der, &mut pos)?;
    let end = pos + len;

    let prime = read_integer(der, &mut pos)?;
    let generator = read_integer(der, &mut pos)?;

    // `privateValueLength` is OPTIONAL — skip it if present so we don't
    // trip the trailing-data check below.
    if pos < end && der.get(pos) == Some(&DER_TAG_INTEGER) {
        let _private_length = read_integer(der, &mut pos)?;
    }
    if pos != end {
        return Err("unexpected trailing data in DH params".to_string());
    }
    Ok(DhParams { prime, generator })
}

fn read_integer(der: &[u8], pos: &mut usize) -> std::result::Result<BigUint, String> {
    expect_tag(der, pos, DER_TAG_INTEGER)?;
    let len = read_len(der, pos)?;
    let bytes = der.get(*pos..*pos + len).ok_or("truncated integer")?;
    *pos += len;
    Ok(BigUint::from_bytes_be(trim_sign_byte(bytes)))
}

fn expect_tag(der: &[u8], pos: &mut usize, tag: u8) -> std::result::Result<(), String> {
    match der.get(*pos) {
        Some(value) if *value == tag => {
            *pos += 1;
            Ok(())
        }
        _ => Err(format!("expected ASN.1 tag 0x{tag:02x}")),
    }
}

// Decodes a DER definite-length field (X.690 §8.1.3).
//
// Short form (first byte high bit clear): the byte itself is the length.
// Long form  (first byte high bit set):   the low 7 bits give a byte count
//                                         `n`, and the next `n` big-endian
//                                         bytes hold the actual length.
// Indefinite form is illegal under DER and is not handled.
fn read_len(der: &[u8], pos: &mut usize) -> std::result::Result<usize, String> {
    let first = *der.get(*pos).ok_or("missing DER length")?;
    *pos += 1;
    if first & DER_LENGTH_LONG_FORM_FLAG == 0 {
        return Ok(first as usize);
    }
    let width = (first & DER_LENGTH_VALUE_MASK) as usize;
    let bytes = der.get(*pos..*pos + width).ok_or("truncated DER length")?;
    *pos += width;
    Ok(bytes
        .iter()
        .fold(0_usize, |acc, byte| (acc << 8) | *byte as usize))
}

// DER INTEGERs are signed. A leading 0x00 is inserted whenever the high bit
// of the magnitude would otherwise be interpreted as a sign bit. Strip it so
// `BigUint` (unsigned) sees the natural-size magnitude.
fn trim_sign_byte(bytes: &[u8]) -> &[u8] {
    if bytes.first() == Some(&0) {
        &bytes[1..]
    } else {
        bytes
    }
}

fn dh_error(path: &Path, message: &str) -> IbkrError {
    IbkrError::DhParams {
        path: path.to_path_buf(),
        message: message.to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_two_integer_dh_params() {
        let params = parse_sequence(&[
            0x30, 0x06, // SEQUENCE, 6 bytes
            0x02, 0x01, 0x17, // prime = 23
            0x02, 0x01, 0x05, // generator = 5
        ])
        .expect("two-integer DH params should parse");

        assert_eq!(params.prime, BigUint::from(23_u8));
        assert_eq!(params.generator, BigUint::from(5_u8));
    }

    #[test]
    fn parses_three_integer_openssl_dh_params() {
        let params = parse_sequence(&[
            0x30, 0x09, // SEQUENCE, 9 bytes
            0x02, 0x01, 0x17, // prime = 23
            0x02, 0x01, 0x05, // generator = 5
            0x02, 0x01, 0xe1, // optional recommended private length
        ])
        .expect("three-integer OpenSSL DH params should parse");

        assert_eq!(params.prime, BigUint::from(23_u8));
        assert_eq!(params.generator, BigUint::from(5_u8));
    }

    #[test]
    fn rejects_trailing_non_integer_data() {
        let err = parse_sequence(&[
            0x30, 0x09, // SEQUENCE, 9 bytes
            0x02, 0x01, 0x17, // prime = 23
            0x02, 0x01, 0x05, // generator = 5
            0x05, 0x00, 0x00, // invalid trailing non-integer data
        ])
        .expect_err("non-integer trailing data should fail");

        assert_eq!(err, "unexpected trailing data in DH params");
    }

    // Realistic 2048-bit DH params encode their prime length as 0x82 0x01 0x01
    // (long-form length, two bytes, value 0x0101 = 257). Exercise that path
    // explicitly so a regression in length parsing surfaces here, not at the
    // crypto layer.
    #[test]
    fn parses_long_form_length() {
        // SEQUENCE containing INTEGER(prime) of 200 bytes + INTEGER(generator)=2.
        // Inner bytes: 1 (tag) + 2 (long-form len) + 200 (content) + 3 (gen) = 206.
        let prime_bytes = vec![0xab_u8; 200];
        let mut der = vec![DER_TAG_SEQUENCE, DER_LENGTH_LONG_FORM_FLAG | 0x01, 206];
        der.push(DER_TAG_INTEGER);
        der.push(DER_LENGTH_LONG_FORM_FLAG | 0x01);
        der.push(200);
        der.extend_from_slice(&prime_bytes);
        der.extend_from_slice(&[DER_TAG_INTEGER, 0x01, 0x02]);

        let params = parse_sequence(&der).expect("long-form length should parse");
        assert_eq!(params.generator, BigUint::from(2_u8));
        assert_eq!(params.prime, BigUint::from_bytes_be(&prime_bytes));
    }

    #[test]
    fn pem_body_decodes_base64_between_armor_lines() {
        // base64("hi") = "aGk=". The armor labels are deliberately wrong-but-
        // present to confirm we ignore them rather than validating.
        let pem = "-----BEGIN WHATEVER-----\naGk=\n-----END WHATEVER-----\n";
        assert_eq!(pem_body(pem).as_deref(), Some(b"hi".as_slice()));
    }

    #[test]
    fn pem_body_returns_none_for_invalid_base64() {
        assert!(pem_body("-----BEGIN X-----\n!!!\n-----END X-----\n").is_none());
    }
}
