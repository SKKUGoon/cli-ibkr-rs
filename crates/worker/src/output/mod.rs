pub mod json;

pub use json::write_json;

use std::fs;
use std::io::{self, Write};
use std::path::Path;

use crate::error::WorkerError;

pub fn write_text(text: &str, output: Option<&Path>) -> Result<(), WorkerError> {
    write_bytes(text.as_bytes(), output, false)
}

pub(crate) fn write_bytes(
    bytes: &[u8],
    output: Option<&Path>,
    trailing_newline: bool,
) -> Result<(), WorkerError> {
    match output {
        Some(path) => fs::write(path, bytes).map_err(|source| WorkerError::WriteOutput {
            path: path.to_path_buf(),
            source,
        }),
        None => {
            let mut stdout = io::stdout().lock();
            stdout.write_all(bytes)?;
            if trailing_newline {
                stdout.write_all(b"\n")?;
            }
            Ok(())
        }
    }
}
