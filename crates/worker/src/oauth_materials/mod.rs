pub mod files;
pub mod openssl;

use std::path::Path;

use crate::error::WorkerError;

pub fn generate(out_dir: &Path, force: bool) -> Result<(), WorkerError> {
    openssl::ensure_available()?;
    files::prepare_directory(out_dir, force)?;
    openssl::run_all(out_dir)?;
    files::protect_private_files(out_dir)?;
    print_public_file_notice(out_dir);
    Ok(())
}

fn print_public_file_notice(out_dir: &Path) {
    eprintln!("generated OAuth materials in {}", out_dir.display());
    eprintln!("send public_signature.pem, public_encryption.pem, and dhparam.pem to IBKR");
    eprintln!("do not share private_signature.pem, private_encryption.pem, or *.pk8 files");
}
