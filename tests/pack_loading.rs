use std::fs;
use std::path::PathBuf;
use tempfile::tempdir;

use jev_ops::packs::discovery::{find_pack, list_available_packs, search_directories};
use jev_ops::packs::loader::load_manifest;

#[test]
fn test_load_standard_packs() {
    let linux = load_manifest(&PathBuf::from("packs/linux/pack.yaml")).unwrap();
    assert_eq!(linux.metadata.name, "linux");
    assert_eq!(linux.metadata.version, "0.1.0");

    let k8s = load_manifest(&PathBuf::from("packs/kubernetes/pack.yaml")).unwrap();
    assert_eq!(k8s.metadata.name, "kubernetes");

    let example = load_manifest(&PathBuf::from("packs/example/pack.yaml")).unwrap();
    assert_eq!(example.metadata.name, "example");
}

#[test]
fn test_load_from_directory() {
    let linux = load_manifest(&PathBuf::from("packs/linux")).unwrap();
    assert_eq!(linux.metadata.name, "linux");
}

#[test]
fn test_custom_pack_dir_precedence() {
    let dir = tempdir().unwrap();
    let custom_pack_yaml = r#"
api_version: "jev-ops/v1"
kind: "DiagnosticPack"
metadata:
  name: "linux"
  version: "9.9.9"
  description: "Overridden custom linux pack"
  author: "custom"
spec:
  input:
    type: "text"
  decisions:
    health:
      type: "choice"
      values: ["healthy", "unhealthy"]
  instructions: "Custom test instructions"
"#;
    let custom_linux_dir = dir.path().join("linux");
    fs::create_dir_all(&custom_linux_dir).unwrap();
    fs::write(custom_linux_dir.join("pack.yaml"), custom_pack_yaml).unwrap();

    // When custom dir is passed, it should take precedence over ./packs/linux
    let (_path, manifest) = find_pack("linux", Some(dir.path())).unwrap();
    assert_eq!(manifest.metadata.version, "9.9.9");
    assert_eq!(manifest.metadata.author, "custom");
}

#[test]
fn test_search_directories_precedence() {
    let custom = PathBuf::from("/custom/packs");
    let dirs = search_directories(Some(&custom));
    assert_eq!(dirs[0], custom);
    assert_eq!(dirs[1], PathBuf::from("./packs"));
}

#[test]
fn test_list_available_packs() {
    let packs = list_available_packs(None);
    let names: Vec<String> = packs.into_iter().map(|p| p.name).collect();
    assert!(names.contains(&"linux".to_string()));
    assert!(names.contains(&"kubernetes".to_string()));
    assert!(names.contains(&"example".to_string()));
}
