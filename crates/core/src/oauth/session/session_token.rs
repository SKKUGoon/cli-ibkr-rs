use base64::Engine;
use rsa::BigUint;

use crate::{IbkrError, Result};

use crate::oauth::signing::{hmac, nonce};
use crate::oauth::OAuthConfig;

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
    hmac::hmac_sha1_b64(&key, &prepend_bytes)
}

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

fn shared_secret_bytes(value: &BigUint) -> Vec<u8> {
    let mut bytes = value.to_bytes_be();
    if value.bits() % 8 == 0 {
        bytes.insert(0, 0);
    }
    bytes
}
