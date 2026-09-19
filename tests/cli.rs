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

#[test]
fn test_cli_version() {
    let mut cmd = jev_ops_cmd();
    cmd.arg("version");
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("jev-ops v2026.09.19"));
}

#[test]
fn test_cli_completion() {
    let mut cmd = jev_ops_cmd();
    cmd.args(["completion", "bash"]);
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("complete -F _jev__ops"));
}

#[test]
fn test_cli_packs_list() {
    let mut cmd = jev_ops_cmd();
    cmd.args(["packs", "list"]);
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("linux"))
        .stdout(predicate::str::contains("kubernetes"));
}

#[test]
fn test_cli_packs_list_json() {
    let mut cmd = jev_ops_cmd();
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
    let mut cmd = jev_ops_cmd();
    cmd.args(["packs", "show", "linux"]);
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("Diagnostic Pack: linux"))
        .stdout(predicate::str::contains("- health: choice"))
        .stdout(predicate::str::contains("Decisions"));
}

#[test]
fn test_cli_packs_show_json() {
    let mut cmd = jev_ops_cmd();
    cmd.args(["packs", "show", "linux", "--json"]);
    let assert = cmd.assert().success();
    let stdout = String::from_utf8(assert.get_output().stdout.clone()).unwrap();
    let json: serde_json::Value = serde_json::from_str(&stdout).unwrap();
    assert_eq!(json["metadata"]["name"], "linux");
}

#[test]
fn test_cli_packs_validate() {
    let mut cmd = jev_ops_cmd();
    cmd.args(["packs", "validate", "./packs/linux/pack.yaml"]);
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("is valid"));
}

#[test]
fn test_cli_packs_new() {
    let dir = tempdir().unwrap();
    let mut cmd = jev_ops_cmd();
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
    let mut val_cmd = jev_ops_cmd();
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
    let mut cmd = jev_ops_cmd();
    cmd.args([
        "analyze",
        "linux",
        "--provider",
        "mock",
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
    let mut cmd = jev_ops_cmd();
    cmd.args(["analyze", "linux", "--provider", "mock", "--json"]);
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
    let mut cmd = jev_ops_cmd();
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
    let mut cmd = jev_ops_cmd();
    cmd.arg("--unknown-flag-xyz");
    cmd.assert().failure().code(2);
}

#[test]
fn test_cli_help_and_version_flags_exit_zero_on_stdout() {
    jev_ops_cmd()
        .arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains("Usage"));
    jev_ops_cmd()
        .arg("--version")
        .assert()
        .success()
        .stdout(predicate::str::contains("v2026.09.19"));
}

#[test]
fn test_cli_min_confidence_out_of_range_rejected() {
    jev_ops_cmd()
        .args([
            "analyze",
            "linux",
            "--provider",
            "mock",
            "--min-confidence",
            "42",
        ])
        .write_stdin("x")
        .assert()
        .failure()
        .code(2)
        .stderr(predicate::str::contains("outside the valid range"));
}

#[test]
fn test_cli_min_confidence_reported_in_json() {
    let assert = jev_ops_cmd()
        .args([
            "analyze",
            "linux",
            "--provider",
            "mock",
            "--json",
            "--min-confidence",
            "0.97",
            "--input",
            "tests/fixtures/linux/oom.txt",
        ])
        .assert()
        .success();
    let json: serde_json::Value = serde_json::from_slice(&assert.get_output().stdout).unwrap();
    assert_eq!(json["min_confidence"], 0.97);
    let low: Vec<&str> = json["low_confidence"]
        .as_array()
        .unwrap()
        .iter()
        .map(|v| v.as_str().unwrap())
        .collect();
    // Mock OOM scenario: health 0.96 and severity 0.94 fall below 0.97; category 0.97 does not.
    assert!(low.contains(&"health"));
    assert!(low.contains(&"severity"));
    assert!(!low.contains(&"category"));
}

/// Writes a pack named `name` into `dir` with the given description and decisions YAML.
fn write_pack(dir: &std::path::Path, name: &str, description: &str, decisions: &str) {
    let pack_dir = dir.join(name);
    fs::create_dir_all(&pack_dir).unwrap();
    fs::write(
        pack_dir.join("pack.yaml"),
        format!(
            "api_version: \"jev-ops/v1\"\nkind: \"DiagnosticPack\"\nmetadata:\n  name: \"{name}\"\n  version: \"0.1.0\"\n  description: \"{description}\"\n  author: \"t\"\nspec:\n  decisions:\n{decisions}  instructions: \"x\"\n"
        ),
    )
    .unwrap();
}

#[test]
fn test_cli_packs_list_multibyte_description_does_not_panic() {
    let dir = tempdir().unwrap();
    // The em dash starts at byte 34, so a byte slice at 35 would split it.
    write_pack(
        dir.path(),
        "utf",
        "PostgreSQL replication lag monito — primary and replica",
        "    ok:\n      type: boolean\n",
    );
    jev_ops_cmd()
        .args(["packs", "list", "--pack-dir", dir.path().to_str().unwrap()])
        .assert()
        .success()
        .stdout(predicate::str::contains(
            "PostgreSQL replication lag monito —...",
        ));
}

#[test]
fn test_cli_human_score_uses_pack_max() {
    let dir = tempdir().unwrap();
    write_pack(
        dir.path(),
        "tiny",
        "small scale",
        "    severity:\n      type: score\n      min: 0\n      max: 3\n",
    );
    jev_ops_cmd()
        .args([
            "analyze",
            "tiny",
            "--provider",
            "mock",
            "--pack-dir",
            dir.path().to_str().unwrap(),
        ])
        .write_stdin("Out of memory: Killed process 42")
        .assert()
        .success()
        .stdout(predicate::str::contains("3/3"))
        .stdout(predicate::str::contains("/5").not());
}

#[test]
fn test_cli_unknown_pack_field_rejected() {
    let dir = tempdir().unwrap();
    write_pack(
        dir.path(),
        "typo",
        "typo pack",
        "    severity:\n      type: score\n      min: 0\n      max: 2\n      levles: [a, b, c]\n",
    );
    jev_ops_cmd()
        .args([
            "packs",
            "validate",
            dir.path().join("typo").to_str().unwrap(),
        ])
        .assert()
        .failure()
        .code(3)
        .stderr(predicate::str::contains("unknown field `levles`"));
}

#[test]
fn test_cli_mock_does_not_flag_healthy_log_with_502_in_pid() {
    let assert = jev_ops_cmd()
        .args(["analyze", "linux", "--provider", "mock", "--json"])
        .write_stdin(
            "Sep 19 12:00:01 web01 systemd[1502]: Started Service nginx. Active: active (running)",
        )
        .assert()
        .success();
    let json: serde_json::Value = serde_json::from_slice(&assert.get_output().stdout).unwrap();
    assert_eq!(json["decisions"]["health"]["value"], "healthy");
    assert_eq!(json["decisions"]["needs_attention"]["value"], false);
}

#[test]
fn test_cli_pack_name_path_traversal_rejected() {
    jev_ops_cmd()
        .args(["analyze", "../packs/linux", "--provider", "mock"])
        .write_stdin("x")
        .assert()
        .failure()
        .code(3)
        .stderr(predicate::str::contains("invalid characters"));
}
