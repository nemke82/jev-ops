use assert_cmd::Command;
use predicates::prelude::*;
use std::fs;
use tempfile::tempdir;

/// Builds a jev-ops command isolated from any TYPESAFE_API_KEY in the developer's environment.
fn jev_ops_cmd() -> Command {
    let mut cmd = Command::cargo_bin("jev-ops").unwrap();
    cmd.env_remove("TYPESAFE_API_KEY");
    cmd
}

use jev_ops::packs::loader::load_manifest;
use jev_ops::security::limits::MAX_MANIFEST_BYTES;
use jev_ops::security::validation::{detect_binary, validate_text_input};

#[test]
fn test_security_huge_input_rejected() {
    let mut cmd = jev_ops_cmd();
    cmd.args([
        "analyze",
        "linux",
        "--input",
        "tests/fixtures/malformed/huge-input.txt",
    ]);
    cmd.assert()
        .failure()
        .code(4)
        .stderr(predicate::str::contains("Input too large"));
}

#[test]
fn test_security_binary_input_rejected() {
    let mut cmd = jev_ops_cmd();
    cmd.args([
        "analyze",
        "linux",
        "--input",
        "tests/fixtures/malformed/binary-input.bin",
    ]);
    cmd.assert()
        .failure()
        .code(4)
        .stderr(predicate::str::contains("Binary or non-text data"));
}

#[test]
fn test_security_empty_input_rejected() {
    let mut cmd = jev_ops_cmd();
    cmd.args(["analyze", "linux"]);
    cmd.write_stdin("");
    cmd.assert()
        .failure()
        .code(4)
        .stderr(predicate::str::contains("Input is empty"));
}

#[test]
fn test_security_invalid_utf8_detected() {
    let bad_bytes = [0xFF, 0xFE, 0xFD];
    assert!(validate_text_input(&bad_bytes).is_err());
}

#[test]
fn test_security_binary_detection_null_byte() {
    let data = b"Valid looking log line with embedded \x00 null byte";
    assert!(detect_binary(data));
}

#[test]
fn test_security_oversized_manifest_rejected() {
    let dir = tempdir().unwrap();
    let huge_manifest_path = dir.path().join("pack.yaml");
    let huge_content = format!(
        "api_version: \"jev-ops/v1\"\nkind: \"DiagnosticPack\"\n# {}\n",
        "A".repeat(MAX_MANIFEST_BYTES + 1024)
    );
    fs::write(&huge_manifest_path, huge_content).unwrap();

    let res = load_manifest(&huge_manifest_path);
    assert!(res.is_err());
    assert!(res.unwrap_err().to_string().contains("exceeds limit of"));
}
