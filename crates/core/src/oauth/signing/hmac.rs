use base64::Engine;
use hmac::{Hmac, Mac};
use rsa::pkcs1v15::SigningKey;
use rsa::{Pkcs1v15Encrypt, RsaPrivateKey};
use sha1::Sha1;
use sha2::Sha256;
use signature::{SignatureEncoding, Signer};

use crate::{IbkrError, Result};

use super::percent;

pub fn rsa_sha256(base_string: &str, key: RsaPrivateKey) -> String {
    let signing_key = SigningKey::<Sha256>::new(key);
    let signature = signing_key.sign(base_string.as_bytes());
    let b64 = base64::engine::general_purpose::STANDARD.encode(signature.to_bytes());
    percent::encode(&b64)
}

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
