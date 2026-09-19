use jev_ops::packs::manifest::{
    DecisionSpec, InputSpec, PackManifest, PackMetadata, PackSpec, CURRENT_API_VERSION,
    CURRENT_KIND,
};
use jev_ops::packs::validation::validate_manifest;
use std::collections::BTreeMap;

fn valid_manifest() -> PackManifest {
    let mut decisions = BTreeMap::new();
    decisions.insert(
        "health".to_string(),
        DecisionSpec::Choice {
            values: vec!["healthy".to_string(), "unhealthy".to_string()],
        },
    );
    decisions.insert(
        "severity".to_string(),
        DecisionSpec::Score {
            min: 0,
            max: 5,
            levels: vec![],
        },
    );
    decisions.insert("needs_attention".to_string(), DecisionSpec::Boolean);

    PackManifest {
        api_version: CURRENT_API_VERSION.to_string(),
        kind: CURRENT_KIND.to_string(),
        metadata: PackMetadata {
            name: "test-pack".to_string(),
            version: "1.0.0".to_string(),
            description: "A valid test pack".to_string(),
            author: "Validator".to_string(),
        },
        spec: PackSpec {
            input: InputSpec::default(),
            decisions,
            instructions: "Valid test instructions".to_string(),
        },
    }
}

#[test]
fn test_valid_pack_passes() {
    assert!(validate_manifest(&valid_manifest()).is_ok());
}

#[test]
fn test_invalid_api_version() {
    let mut m = valid_manifest();
    m.api_version = "jev-ops/v2-invalid".to_string();
    let err = validate_manifest(&m).unwrap_err().to_string();
    assert!(err.contains("unsupported api_version"));
}

#[test]
fn test_invalid_kind() {
    let mut m = valid_manifest();
    m.kind = "BadKind".to_string();
    let err = validate_manifest(&m).unwrap_err().to_string();
    assert!(err.contains("unsupported kind"));
}

#[test]
fn test_invalid_semver() {
    let mut m = valid_manifest();
    m.metadata.version = "1.0-bad".to_string();
    let err = validate_manifest(&m).unwrap_err().to_string();
    assert!(err.contains("not valid semantic version"));
}

#[test]
fn test_invalid_pack_name() {
    let mut m = valid_manifest();
    m.metadata.name = "My Invalid Name!".to_string();
    let err = validate_manifest(&m).unwrap_err().to_string();
    assert!(err.contains("metadata.name:"));
}

#[test]
fn test_invalid_score_range() {
    let mut m = valid_manifest();
    m.spec.decisions.insert(
        "severity".to_string(),
        DecisionSpec::Score {
            min: 5,
            max: 2,
            levels: vec![],
        },
    );
    let err = validate_manifest(&m).unwrap_err().to_string();
    assert!(err.contains("score minimum (5) must be lower than maximum (2)"));
}

#[test]
fn test_duplicate_choice_values() {
    let mut m = valid_manifest();
    m.spec.decisions.insert(
        "health".to_string(),
        DecisionSpec::Choice {
            values: vec!["ok".to_string(), "degraded".to_string(), "ok".to_string()],
        },
    );
    let err = validate_manifest(&m).unwrap_err().to_string();
    assert!(err.contains("duplicate choice value 'ok'"));
}

#[test]
fn test_empty_choice_values() {
    let mut m = valid_manifest();
    m.spec.decisions.insert(
        "health".to_string(),
        DecisionSpec::Choice { values: vec![] },
    );
    let err = validate_manifest(&m).unwrap_err().to_string();
    assert!(err.contains("must define at least one value"));
}

#[test]
fn test_empty_instructions() {
    let mut m = valid_manifest();
    m.spec.instructions = "   ".to_string();
    let err = validate_manifest(&m).unwrap_err().to_string();
    assert!(err.contains("instructions cannot be empty"));
}
