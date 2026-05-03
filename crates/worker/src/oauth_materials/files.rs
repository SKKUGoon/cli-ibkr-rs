use std::fs;
use std::path::{Path, PathBuf};

use crate::error::WorkerError;

const GENERATED_FILES: &[&str] = &[
    "dhparam.pem",
    "private_signature.pem",
    "private_encryption.pem",
    "public_signature.pem",
    "public_encryption.pem",
    "private_encryption.pk8",
    "private_signature.pk8",
];

pub fn prepare_directory(out_dir: &Path, force: bool) -> Result<(), WorkerError> {
    fs::create_dir_all(out_dir).map_err(|source| WorkerError::CreateDir {
        path: out_dir.to_path_buf(),
        source,
    })?;

    if !force {
        for file in GENERATED_FILES {
            let path = out_dir.join(file);
            if path.exists() {
                return Err(WorkerError::RefuseOverwrite(path));
            }
        }
    }

    Ok(())
}

pub fn protect_private_files(out_dir: &Path) -> Result<(), WorkerError> {
    for file in private_files(out_dir) {
        set_private_permissions(&file)?;
    }
    Ok(())
}

fn private_files(out_dir: &Path) -> Vec<PathBuf> {
    [
        "private_signature.pem",
        "private_encryption.pem",
        "private_encryption.pk8",
        "private_signature.pk8",
    ]
    .iter()
    .map(|file| out_dir.join(file))
    .collect()
}

#[cfg(unix)]
fn set_private_permissions(path: &Path) -> Result<(), WorkerError> {
    use std::os::unix::fs::PermissionsExt;

    let permissions = fs::Permissions::from_mode(0o600);
    fs::set_permissions(path, permissions).map_err(|source| WorkerError::SetPermissions {
        path: path.to_path_buf(),
        source,
    })
}

#[cfg(not(unix))]
fn set_private_permissions(_path: &Path) -> Result<(), WorkerError> {
    Ok(())
}
