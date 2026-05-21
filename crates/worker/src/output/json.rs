use std::path::Path;

use serde_json::Value;

use crate::error::WorkerError;

use super::write_bytes;

pub fn write_json(value: &Value, output: Option<&Path>, pretty: bool) -> Result<(), WorkerError> {
    let bytes = if pretty {
        serde_json::to_vec_pretty(value)?
    } else {
        serde_json::to_vec(value)?
    };
    write_bytes(&bytes, output, output.is_none())
}
