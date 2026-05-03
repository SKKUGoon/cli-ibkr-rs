use std::fs;
use std::io::{self, Write};
use std::path::Path;

use serde_json::Value;

use crate::error::WorkerError;

pub fn write_json(value: &Value, output: Option<&Path>, pretty: bool) -> Result<(), WorkerError> {
    let bytes = if pretty {
        serde_json::to_vec_pretty(value)?
    } else {
        serde_json::to_vec(value)?
    };

    match output {
        Some(path) => fs::write(path, bytes).map_err(|source| WorkerError::WriteOutput {
            path: path.to_path_buf(),
            source,
        }),
        None => write_stdout(&bytes),
    }
}

fn write_stdout(bytes: &[u8]) -> Result<(), WorkerError> {
    let mut stdout = io::stdout().lock();
    stdout.write_all(bytes)?;
    stdout.write_all(b"\n")?;
    Ok(())
}
