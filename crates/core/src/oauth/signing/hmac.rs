// Signing primitives for IBKR's OAuth flow.
//
// Which algorithm runs where:
//   * `rsa_sha256`        — signs the live-session-token (LST) request only.
//                           IBKR verifies it with the public half of the
//                           signature key on file with their portal.
//   * `decrypt_prepend`   — one-time per LST request: RSA-PKCS1v15-decrypts
//                           the stored access-token secret, producing the
//                           "prepend" hex string that goes in front of the
//                           LST base string.
//   * `hmac_sha256`       — signs every *protected* API request once we have
//                           an LST. The key is the LST itself, base64-decoded.
//   * `hmac_sha1_b64/hex` — internal helpers used by `session_token::compute`
//                           and `session_token::validate`; they implement the
//                           DH-derived LST construction and the round-trip
//                           verification IBKR documents for it.

use base64::Engine;
use hmac::{Hmac, Mac};
use rsa::pkcs1v15::SigningKey;
use rsa::{Pkcs1v15Encrypt, RsaPrivateKey};
use sha1::Sha1;
use sha2::Sha256;
use signature::{SignatureEncoding, Signer};

use crate::{IbkrError, Result};

use super::percent;

// PKCS#1 v1.5 SHA-256 over the OAuth base string. Output is percent-encoded
// base64 because it goes directly into the `oauth_signature` header value.
pub fn rsa_sha256(base_string: &str, key: RsaPrivateKey) -> String {
    let signing_key = SigningKey::<Sha256>::new(key);
    let signature = signing_key.sign(base_string.as_bytes());
    let b64 = base64::engine::general_purpose::STANDARD.encode(signature.to_bytes());
    percent::encode(&b64)
}

// HMAC-SHA256 over the OAuth base string, keyed by the live session token.
// The LST is delivered as base64 and must be decoded to its raw bytes before
// use — feeding the base64 string in directly would silently produce a wrong
// (but well-formed) signature.
pub fn hmac_sha256(base_string: &str, live_session_token: &str) -> Result<String> {
    let key = base64::engine::general_purpose::STANDARD
        .decode(live_session_token)
        .map_err(|err| IbkrError::Crypto(format!("invalid live session token base64: {err}")))?;
    let mut mac = Hmac::<Sha256>::new_from_slice(&key)
        .map_err(|err| IbkrError::Crypto(format!("invalid HMAC-SHA256 key: {err}")))?;
    mac.update(base_string.as_bytes());
    let b64 = base64::engine::general_purpose::STANDARD.encode(mac.finalize().into_bytes());
    Ok(percent::encode(&b64))
}

// Recovers the LST "prepend" by RSA-decrypting the stored access-token secret
// with the encryption key. PKCS#1 v1.5 padding is the scheme IBKR's portal
// uses when it issues the secret — OAEP would silently produce garbage.
// The output is hex because that is the form the LST base-string prefix needs.
pub fn decrypt_prepend(access_token_secret: &str, key: RsaPrivateKey) -> Result<String> {
    let encrypted = base64::engine::general_purpose::STANDARD
        .decode(access_token_secret)
        .map_err(|err| IbkrError::Crypto(format!("invalid access token secret base64: {err}")))?;
    let decrypted = key
        .decrypt(Pkcs1v15Encrypt, &encrypted)
        .map_err(|err| IbkrError::Crypto(format!("access token secret decrypt failed: {err}")))?;
    Ok(hex::encode(decrypted))
}

pub fn hmac_sha1_b64(key: &[u8], message: &[u8]) -> Result<String> {
    let mut mac = Hmac::<Sha1>::new_from_slice(key)
        .map_err(|err| IbkrError::Crypto(format!("invalid HMAC-SHA1 key: {err}")))?;
    mac.update(message);
    Ok(base64::engine::general_purpose::STANDARD.encode(mac.finalize().into_bytes()))
}

pub fn hmac_sha1_hex(key: &[u8], message: &[u8]) -> Result<String> {
    let mut mac = Hmac::<Sha1>::new_from_slice(key)
        .map_err(|err| IbkrError::Crypto(format!("invalid HMAC-SHA1 key: {err}")))?;
    mac.update(message);
    Ok(hex::encode(mac.finalize().into_bytes()))
}
