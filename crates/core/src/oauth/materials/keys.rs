use std::fs;
use std::path::{Path, PathBuf};

use rsa::pkcs1::DecodeRsaPrivateKey;
use rsa::pkcs8::DecodePrivateKey;
use rsa::RsaPrivateKey;

use crate::{IbkrError, Result};

pub fn read_private_key(path: &Path) -> Result<RsaPrivateKey> {
    let pem = fs::read_to_string(path).map_err(|source| IbkrError::ReadFile {
        path: path.to_path_buf(),
        source,
    })?;

    if pem.contains("BEGIN RSA PRIVATE KEY") {
        return RsaPrivateKey::from_pkcs1_pem(&pem).map_err(|err| key_error(path, err));
    }

    if pem.contains("BEGIN PRIVATE KEY") {
        return RsaPrivateKey::from_pkcs8_pem(&pem).map_err(|err| key_error(path, err));
    }

    Err(IbkrError::InvalidKeyFormat {
        path: PathBuf::from(path),
        expected: "PKCS#1 RSA PRIVATE KEY or PKCS#8 PRIVATE KEY PEM",
    })
}

fn key_error(path: &Path, err: impl std::fmt::Display) -> IbkrError {
    IbkrError::RsaKey {
        path: path.to_path_buf(),
        message: err.to_string(),
    }
}
