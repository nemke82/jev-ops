use assert_cmd::Command;
use predicates::prelude::*;

use jev_ops::inference::resolve_provider;

#[test]
fn test_resolve_mock_provider() {
    let provider = resolve_provider(Some("mock"), None).unwrap();
    assert_eq!(provider.name(), "mock");
}

#[test]
fn test_resolve_typesafe_provider_with_explicit_key() {
    let provider = resolve_provider(Some("typesafe"), Some("ts_test_12345")).unwrap();
    assert_eq!(provider.name(), "typesafe-jev");
}

#[test]
fn test_resolve_typesafe_provider_missing_key() {
    // Ensure environment does not leak a key during this test
    std::env::remove_var("TYPESAFE_API_KEY");
    let res = resolve_provider(Some("typesafe"), None);
    assert!(res.is_err());
    let err = res.err().unwrap().to_string();
    assert!(err.contains("Missing API key"));
    assert!(err.contains("TYPESAFE_API_KEY"));
}

#[test]
fn test_resolve_unknown_provider() {
    let res = resolve_provider(Some("non-existent-provider"), None);
    assert!(res.is_err());
    let err = res.err().unwrap().to_string();
    assert!(err.contains("Unknown inference provider"));
}

#[test]
fn test_cli_typesafe_missing_api_key_exit_code() {
    let mut cmd = Command::cargo_bin("jev-ops").unwrap();
    cmd.env_remove("TYPESAFE_API_KEY");
    cmd.args([
        "analyze",
        "linux",
        "--provider",
        "typesafe",
        "--input",
        "tests/fixtures/linux/healthy.txt",
    ]);
    cmd.assert()
        .failure()
        .code(5)
        .stderr(predicate::str::contains("Missing API key"));
}

#[test]
fn test_cli_explicit_mock_provider() {
    let mut cmd = Command::cargo_bin("jev-ops").unwrap();
    cmd.args([
        "analyze",
        "linux",
        "--provider",
        "mock",
        "--input",
        "tests/fixtures/linux/ext4-error.txt",
        "--json",
    ]);
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("\"provider\": \"mock\""))
        .stdout(predicate::str::contains("\"filesystem\""));
}
