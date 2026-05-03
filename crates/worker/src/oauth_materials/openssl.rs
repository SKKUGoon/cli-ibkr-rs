use std::path::Path;
use std::process::Command;

use crate::error::WorkerError;

pub fn ensure_available() -> Result<(), WorkerError> {
    let output = Command::new("openssl").arg("version").output()?;
    if !output.status.success() {
        return Err(WorkerError::OpenSsl(command_error(
            "openssl version",
            output.stderr,
        )));
    }
    Ok(())
}

pub fn run_all(out_dir: &Path) -> Result<(), WorkerError> {
    run(
        out_dir,
        &["dhparam", "-out", "dhparam.pem", "-outform", "PEM", "2048"],
    )?;
    run(
        out_dir,
        &["genrsa", "-out", "private_signature.pem", "2048"],
    )?;
    run(
        out_dir,
        &["genrsa", "-out", "private_encryption.pem", "2048"],
    )?;
    run(
        out_dir,
        &[
            "rsa",
            "-in",
            "private_signature.pem",
            "-outform",
            "PEM",
            "-pubout",
            "-out",
            "public_signature.pem",
        ],
    )?;
    run(
        out_dir,
        &[
            "rsa",
            "-in",
            "private_encryption.pem",
            "-outform",
            "PEM",
            "-pubout",
            "-out",
            "public_encryption.pem",
        ],
    )?;
    run(
        out_dir,
        &[
            "pkcs8",
            "-topk8",
            "-inform",
            "PEM",
            "-outform",
            "PEM",
            "-in",
            "private_encryption.pem",
            "-out",
            "private_encryption.pk8",
            "-nocrypt",
        ],
    )?;
    run(
        out_dir,
        &[
            "pkcs8",
            "-topk8",
            "-inform",
            "PEM",
            "-outform",
            "PEM",
            "-in",
            "private_signature.pem",
            "-out",
            "private_signature.pk8",
            "-nocrypt",
        ],
    )?;
    Ok(())
}

fn run(out_dir: &Path, args: &[&str]) -> Result<(), WorkerError> {
    let output = Command::new("openssl")
        .args(args)
        .current_dir(out_dir)
        .output()?;
    if output.status.success() {
        return Ok(());
    }
    Err(WorkerError::OpenSsl(command_error(
        &format!("openssl {}", args.join(" ")),
        output.stderr,
    )))
}

fn command_error(command: &str, stderr: Vec<u8>) -> String {
    let message = String::from_utf8_lossy(&stderr).trim().to_string();
    if message.is_empty() {
        format!("{command} failed without stderr")
    } else {
        format!("{command} failed: {message}")
    }
}
