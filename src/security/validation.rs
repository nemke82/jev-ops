use crate::error::{JevOpsError, Result};
use crate::security::limits::MAX_PACK_NAME_LEN;

/// Validates that raw bytes are valid UTF-8 and do not contain binary garbage.
pub fn validate_text_input(bytes: &[u8]) -> Result<&str> {
    let s = std::str::from_utf8(bytes).map_err(|e| JevOpsError::InvalidUtf8(e.to_string()))?;

    if detect_binary(bytes) {
        return Err(JevOpsError::BinaryContentDetected);
    }

    Ok(s)
}

/// Detects binary/non-text garbage.
/// Looks for null bytes (\0) or a high concentration of control characters (excluding newline, tab, carriage return).
pub fn detect_binary(bytes: &[u8]) -> bool {
    if bytes.is_empty() {
        return false;
    }

    // Direct null byte presence is an immediate indicator of binary data
    if bytes.contains(&0) {
        return true;
    }

    // Inspect first 1024 bytes for non-printable control characters
    let sample_size = bytes.len().min(1024);
    let sample = &bytes[..sample_size];

    let mut control_count = 0;
    for &b in sample {
        // Allow tab (9), newline (10), carriage return (13)
        if b < 32 && b != 9 && b != 10 && b != 13 {
            control_count += 1;
        }
    }

    // If more than 5% control characters, treat as binary
    (control_count as f64 / sample_size as f64) > 0.05
}

/// Validates that a pack name adheres to naming rules:
/// - non-empty
/// - length <= MAX_PACK_NAME_LEN
/// - only lowercase ASCII alphanumeric and hyphens/underscores
/// - does not start or end with a hyphen/underscore
pub fn validate_pack_name(name: &str) -> Result<()> {
    if name.is_empty() {
        return Err(JevOpsError::InvalidPack(
            "Pack name cannot be empty".to_string(),
        ));
    }

    if name.len() > MAX_PACK_NAME_LEN {
        return Err(JevOpsError::InvalidPack(format!(
            "Pack name '{}' exceeds maximum allowed length of {} characters",
            name, MAX_PACK_NAME_LEN
        )));
    }

    if !name
        .chars()
        .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-' || c == '_')
    {
        return Err(JevOpsError::InvalidPack(format!(
            "Pack name '{}' contains invalid characters. Use only lowercase alphanumeric and '-' or '_'",
            name
        )));
    }

    if name.starts_with('-') || name.ends_with('-') || name.starts_with('_') || name.ends_with('_')
    {
        return Err(JevOpsError::InvalidPack(format!(
            "Pack name '{}' cannot start or end with a hyphen or underscore",
            name
        )));
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_text() {
        let text = b"Sep 19 12:00:00 server systemd: Started Service.";
        assert!(validate_text_input(text).is_ok());
    }

    #[test]
    fn test_null_byte_detected() {
        let text = b"hello\x00world";
        assert!(validate_text_input(text).is_err());
    }

    #[test]
    fn test_pack_name_rules() {
        assert!(validate_pack_name("linux").is_ok());
        assert!(validate_pack_name("k8s-core").is_ok());
        assert!(validate_pack_name("my_custom_pack").is_ok());
        assert!(validate_pack_name("Invalid-Case").is_err());
        assert!(validate_pack_name("-invalid").is_err());
        assert!(validate_pack_name("").is_err());
    }
}
