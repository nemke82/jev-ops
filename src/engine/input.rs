use std::fs::File;
use std::io::{self, Read};
use std::path::Path;

use crate::engine::context::InputContext;
use crate::error::{JevOpsError, Result};
use crate::security::validation::validate_text_input;

pub struct InputData {
    pub text: String,
    pub context: InputContext,
}

/// Reads input data from a file or from stdin, strictly enforcing byte limits and UTF-8 validity.
pub fn read_input(input_path: Option<&Path>, max_bytes: usize) -> Result<InputData> {
    let mut buffer = Vec::new();

    if let Some(path) = input_path {
        if !path.exists() {
            return Err(JevOpsError::Input(format!(
                "Input file '{}' does not exist",
                path.display()
            )));
        }
        let file = File::open(path).map_err(|e| {
            JevOpsError::Input(format!("Failed to open input file '{}': {}", path.display(), e))
        })?;
        let mut handle = file.take((max_bytes + 1) as u64);
        handle.read_to_end(&mut buffer)?;
    } else {
        let stdin = io::stdin();
        let mut handle = stdin.lock().take((max_bytes + 1) as u64);
        handle.read_to_end(&mut buffer)?;
    }

    if buffer.len() > max_bytes {
        return Err(JevOpsError::InputTooLarge {
            bytes: buffer.len(),
            max_bytes,
        });
    }

    if buffer.is_empty() {
        return Err(JevOpsError::EmptyInput);
    }

    let valid_str = validate_text_input(&buffer)?;

    let byte_count = buffer.len();
    Ok(InputData {
        text: valid_str.to_string(),
        context: InputContext {
            bytes: byte_count,
            truncated: false,
        },
    })
}
