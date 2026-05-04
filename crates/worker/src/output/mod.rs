pub mod json;

pub use json::write_json;

use std::fs;
use std::io::{self, Write};
use std::path::Path;

use crate::error::WorkerError;

pub fn write_text(text: &str, output: Option<&Path>) -> Result<(), WorkerError> {
    match output {
        Some(path) => fs::write(path, text).map_err(|source| WorkerError::WriteOutput {
            path: path.to_path_buf(),
            source,
        }),
        None => {
            let mut stdout = io::stdout().lock();
            stdout.write_all(text.as_bytes())?;
            Ok(())
        }
    }
}
