use std::fs;
use std::path::Path;

use rsa::BigUint;

use crate::{IbkrError, Result};

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

fn pem_body(pem: &str) -> Option<Vec<u8>> {
    let body = pem
        .lines()
        .filter(|line| !line.starts_with("-----"))
        .collect::<String>();
    base64::Engine::decode(&base64::engine::general_purpose::STANDARD, body).ok()
}

fn parse_sequence(der: &[u8]) -> std::result::Result<DhParams, String> {
    let mut pos = 0;
    expect_tag(der, &mut pos, 0x30)?;
    let len = read_len(der, &mut pos)?;
    let end = pos + len;
    let prime = read_integer(der, &mut pos)?;
    let generator = read_integer(der, &mut pos)?;
    if pos < end && der.get(pos) == Some(&0x02) {
        let _private_length = read_integer(der, &mut pos)?;
    }
    if pos != end {
        return Err("unexpected trailing data in DH params".to_string());
    }
    Ok(DhParams { prime, generator })
}

fn read_integer(der: &[u8], pos: &mut usize) -> std::result::Result<BigUint, String> {
    expect_tag(der, pos, 0x02)?;
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

fn read_len(der: &[u8], pos: &mut usize) -> std::result::Result<usize, String> {
    let first = *der.get(*pos).ok_or("missing DER length")?;
    *pos += 1;
    if first & 0x80 == 0 {
        return Ok(first as usize);
    }
    let width = (first & 0x7f) as usize;
    let bytes = der.get(*pos..*pos + width).ok_or("truncated DER length")?;
    *pos += width;
    Ok(bytes
        .iter()
        .fold(0_usize, |acc, byte| (acc << 8) | *byte as usize))
}

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
}
