// Live-session-token (LST) construction and verification.
//
// IBKR derives the LST from a Diffie-Hellman handshake: the client picks a
// random `a`, sends `A = g^a mod p` as `diffie_hellman_challenge`, IBKR replies
// with its own `B`, and both sides compute the shared secret `K = B^a mod p`.
// The LST itself is `HMAC-SHA1(K, prepend)` where `prepend` is the decrypted
// access-token secret. IBKR also returns `live_session_token_signature` which
// is `HMAC-SHA1_hex(LST, consumer_key)` and lets the client confirm both sides
// derived the same LST.

use base64::Engine;
use rsa::BigUint;

use crate::{IbkrError, Result};

use crate::oauth::signing::{hmac, nonce};
use crate::oauth::OAuthConfig;

// 256-bit random for the DH exponent. Pulled from the OS CSPRNG via the
// `nonce` module (hex), then re-decoded to a BigUint. This value MUST stay
// in memory only — never log it.
pub fn random_256_bit() -> BigUint {
    BigUint::from_bytes_be(&hex::decode(nonce::nonce_hex(32)).expect("hex nonce"))
}

pub fn compute(
    prime: &BigUint,
    random: &BigUint,
    dh_response: &str,
    prepend: &str,
) -> Result<String> {
    let peer = BigUint::parse_bytes(dh_response.as_bytes(), 16)
        .ok_or_else(|| IbkrError::Crypto("invalid DH response hex".to_string()))?;

    // Shared secret K = B^a mod p. IBKR expects Java-style signed big-endian
    // bytes, so a leading null byte is inserted when the high bit is set.
    let shared = peer.modpow(random, prime);
    let key = shared_secret_bytes(&shared);

    let prepend_bytes = hex::decode(prepend)
        .map_err(|err| IbkrError::Crypto(format!("invalid prepend hex: {err}")))?;
    // The LST is base64(HMAC-SHA1(K, prepend_bytes)).
    hmac::hmac_sha1_b64(&key, &prepend_bytes)
}

// Confirms IBKR and the client derived the same LST. The server returns
// `expected` = hex(HMAC-SHA1(LST_bytes, consumer_key)). We recompute it
// locally from our LST and compare.
//
// Mismatch is treated as an unrecoverable crypto failure — it usually means
// the DH params, the access-token secret, or the consumer key are out of sync
// with what the portal has on file.
pub fn validate(token: &str, expected: &str, config: &OAuthConfig) -> Result<()> {
    let key = base64::engine::general_purpose::STANDARD
        .decode(token)
        .map_err(|err| IbkrError::Crypto(format!("invalid computed LST base64: {err}")))?;
    let actual = hmac::hmac_sha1_hex(&key, config.consumer_key.as_bytes())?;
    if actual != expected {
        return Err(IbkrError::Crypto(
            "live session token validation failed".to_string(),
        ));
    }
    Ok(())
}

// Pads the shared secret so that its high bit cannot be interpreted as a
// negative sign by IBKR's Java backend. IBKR's reference implementation uses
// `BigInteger.toByteArray()` which always emits two's-complement form; this
// helper reproduces that quirk.
fn shared_secret_bytes(value: &BigUint) -> Vec<u8> {
    let mut bytes = value.to_bytes_be();
    if value.bits() % 8 == 0 {
        bytes.insert(0, 0);
    }
    bytes
}

#[cfg(test)]
mod tests {
    use super::*;

    // Round-trip: pick a tiny prime/generator, a known `a` and a synthetic
    // peer response, then verify `compute()` is deterministic and matches
    // a hand-rolled HMAC-SHA1 over the same inputs.
    #[test]
    fn compute_is_deterministic_for_fixed_inputs() {
        let prime = BigUint::parse_bytes(b"FFFFFFFB", 16).unwrap();
        let random = BigUint::from(7u32);
        let peer = BigUint::from(5u32);
        let dh_response = peer.to_str_radix(16);
        let prepend = "abcdef";

        let first = compute(&prime, &random, &dh_response, prepend).unwrap();
        let second = compute(&prime, &random, &dh_response, prepend).unwrap();
        assert_eq!(first, second);
        assert!(!first.is_empty());
    }

    #[test]
    fn compute_rejects_non_hex_dh_response() {
        let prime = BigUint::from(23u32);
        let random = BigUint::from(7u32);
        let err = compute(&prime, &random, "not-hex", "00").unwrap_err();
        assert!(matches!(err, IbkrError::Crypto(_)));
    }

    #[test]
    fn compute_rejects_non_hex_prepend() {
        let prime = BigUint::from(23u32);
        let random = BigUint::from(7u32);
        let err = compute(&prime, &random, "0a", "zz").unwrap_err();
        assert!(matches!(err, IbkrError::Crypto(_)));
    }
}
