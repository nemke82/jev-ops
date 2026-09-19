use assert_cmd::Command;
use predicates::prelude::*;
use std::fs;
use tempfile::tempdir;

#[test]
fn test_cli_version() {
    let mut cmd = Command::cargo_bin("jev-ops").unwrap();
    cmd.arg("version");
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("jev-ops v2026.09.19"));
}

#[test]
fn test_cli_completion() {
    let mut cmd = Command::cargo_bin("jev-ops").unwrap();
    cmd.args(["completion", "bash"]);
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("complete -F _jev__ops"));
}

#[test]
fn test_cli_packs_list() {
    let mut cmd = Command::cargo_bin("jev-ops").unwrap();
    cmd.args(["packs", "list"]);
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("linux"))
        .stdout(predicate::str::contains("kubernetes"));
}

#[test]
fn test_cli_packs_list_json() {
    let mut cmd = Command::cargo_bin("jev-ops").unwrap();
    cmd.args(["packs", "list", "--json"]);
    let assert = cmd.assert().success();
    let stdout = String::from_utf8(assert.get_output().stdout.clone()).unwrap();
    let json: serde_json::Value = serde_json::from_str(&stdout).unwrap();
    assert!(json.is_array());
    let names: Vec<&str> = json
        .as_array()
        .unwrap()
        .iter()
        .filter_map(|v| v.get("name").and_then(|n| n.as_str()))
        .collect();
    assert!(names.contains(&"linux"));
}

#[test]
fn test_cli_packs_show() {
    let mut cmd = Command::cargo_bin("jev-ops").unwrap();
    cmd.args(["packs", "show", "linux"]);
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("Diagnostic Pack: linux"))
        .stdout(predicate::str::contains("- health: choice"))
        .stdout(predicate::str::contains("Decisions"));
}

#[test]
fn test_cli_packs_show_json() {
    let mut cmd = Command::cargo_bin("jev-ops").unwrap();
    cmd.args(["packs", "show", "linux", "--json"]);
    let assert = cmd.assert().success();
    let stdout = String::from_utf8(assert.get_output().stdout.clone()).unwrap();
    let json: serde_json::Value = serde_json::from_str(&stdout).unwrap();
    assert_eq!(json["metadata"]["name"], "linux");
}

#[test]
fn test_cli_packs_validate() {
    let mut cmd = Command::cargo_bin("jev-ops").unwrap();
    cmd.args(["packs", "validate", "./packs/linux/pack.yaml"]);
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("is valid"));
}

#[test]
fn test_cli_packs_new() {
    let dir = tempdir().unwrap();
    let mut cmd = Command::cargo_bin("jev-ops").unwrap();
    cmd.args([
        "packs",
        "new",
        "my-service",
        "--dir",
        dir.path().to_str().unwrap(),
    ]);
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("Scaffolded diagnostic pack"));

    assert!(dir.path().join("my-service").join("pack.yaml").exists());

    // Validate the newly scaffolded pack
    let mut val_cmd = Command::cargo_bin("jev-ops").unwrap();
    val_cmd.args([
        "packs",
        "validate",
        dir.path()
            .join("my-service")
            .join("pack.yaml")
            .to_str()
            .unwrap(),
    ]);
    val_cmd.assert().success();
}

#[test]
fn test_cli_analyze_file_input_ext4() {
    let mut cmd = Command::cargo_bin("jev-ops").unwrap();
    cmd.args([
        "analyze",
        "linux",
        "--input",
        "tests/fixtures/linux/ext4-error.txt",
    ]);
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("Pack:       linux"))
        .stdout(predicate::str::contains("filesystem"))
        .stdout(predicate::str::contains("unhealthy"));
}

#[test]
fn test_cli_analyze_stdin_json() {
    let fixture_content = fs::read_to_string("tests/fixtures/linux/ext4-error.txt").unwrap();
    let mut cmd = Command::cargo_bin("jev-ops").unwrap();
    cmd.args(["analyze", "linux", "--json"]);
    cmd.write_stdin(fixture_content);

    let assert = cmd.assert().success();
    let stdout = String::from_utf8(assert.get_output().stdout.clone()).unwrap();
    let json: serde_json::Value = serde_json::from_str(&stdout).unwrap();

    assert_eq!(json["schema_version"], "1");
    assert_eq!(json["pack"]["name"], "linux");
    assert_eq!(json["provider"], "mock");
    assert_eq!(json["decisions"]["category"]["value"], "filesystem");
    assert_eq!(json["decisions"]["health"]["value"], "unhealthy");
    assert_eq!(json["decisions"]["severity"]["value"], 5);
    assert_eq!(json["decisions"]["needs_attention"]["value"], true);
}

#[test]
fn test_cli_invalid_pack_exit_code() {
    let mut cmd = Command::cargo_bin("jev-ops").unwrap();
    cmd.args([
        "analyze",
        "non-existent-pack-name-12345",
        "--input",
        "tests/fixtures/linux/healthy.txt",
    ]);
    cmd.assert()
        .failure()
        .code(3)
        .stderr(predicate::str::contains("not found"));
}

#[test]
fn test_cli_invalid_usage_exit_code() {
    let mut cmd = Command::cargo_bin("jev-ops").unwrap();
    cmd.arg("--unknown-flag-xyz");
    cmd.assert().failure().code(2);
}
