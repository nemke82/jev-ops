use std::collections::BTreeMap;

use jev_ops::engine::pipeline::{run_pipeline, validate_provider_response};
use jev_ops::inference::types::ProviderDecision;
use jev_ops::packs::manifest::DecisionSpec;

#[test]
fn test_decision_linux_oom() {
    let fixture = "tests/fixtures/linux/oom.txt";
    let res = run_pipeline(
        "linux",
        Some(std::path::Path::new(fixture)),
        None,
        None,
        Some("mock"),
        None,
    )
    .unwrap();

    let cat = match &res.decisions["category"] {
        ProviderDecision::Choice { value, .. } => value.as_str(),
        _ => panic!("Expected choice for category"),
    };
    assert_eq!(cat, "memory");

    let health = match &res.decisions["health"] {
        ProviderDecision::Choice { value, .. } => value.as_str(),
        _ => panic!("Expected choice for health"),
    };
    assert_eq!(health, "unhealthy");

    let sev = match &res.decisions["severity"] {
        ProviderDecision::Score { value, .. } => *value,
        _ => panic!("Expected score for severity"),
    };
    assert!(sev >= 4);

    let attn = match &res.decisions["needs_attention"] {
        ProviderDecision::Boolean { value, .. } => *value,
        _ => panic!("Expected boolean for needs_attention"),
    };
    assert!(attn);
}

#[test]
fn test_decision_linux_ext4() {
    let fixture = "tests/fixtures/linux/ext4-error.txt";
    let res = run_pipeline(
        "linux",
        Some(std::path::Path::new(fixture)),
        None,
        None,
        Some("mock"),
        None,
    )
    .unwrap();

    let cat = match &res.decisions["category"] {
        ProviderDecision::Choice { value, .. } => value.as_str(),
        _ => panic!("Expected choice for category"),
    };
    assert_eq!(cat, "filesystem");

    let sev = match &res.decisions["severity"] {
        ProviderDecision::Score { value, .. } => *value,
        _ => panic!("Expected score for severity"),
    };
    assert_eq!(sev, 5);
}

#[test]
fn test_decision_linux_ssh_bruteforce() {
    let fixture = "tests/fixtures/linux/ssh-bruteforce.txt";
    let res = run_pipeline(
        "linux",
        Some(std::path::Path::new(fixture)),
        None,
        None,
        Some("mock"),
        None,
    )
    .unwrap();

    let cat = match &res.decisions["category"] {
        ProviderDecision::Choice { value, .. } => value.as_str(),
        _ => panic!("Expected choice for category"),
    };
    assert_eq!(cat, "security");
}

#[test]
fn test_decision_kubernetes_crashloop() {
    let fixture = "tests/fixtures/kubernetes/crashloop.txt";
    let res = run_pipeline(
        "kubernetes",
        Some(std::path::Path::new(fixture)),
        None,
        None,
        Some("mock"),
        None,
    )
    .unwrap();

    let cause = match &res.decisions["root_cause"] {
        ProviderDecision::Choice { value, .. } => value.as_str(),
        _ => panic!("Expected choice for root_cause"),
    };
    assert_eq!(cause, "crashloop");
}

#[test]
fn test_decision_kubernetes_oomkilled() {
    let fixture = "tests/fixtures/kubernetes/oomkilled.txt";
    let res = run_pipeline(
        "kubernetes",
        Some(std::path::Path::new(fixture)),
        None,
        None,
        Some("mock"),
        None,
    )
    .unwrap();

    let cause = match &res.decisions["root_cause"] {
        ProviderDecision::Choice { value, .. } => value.as_str(),
        _ => panic!("Expected choice for root_cause"),
    };
    assert_eq!(cause, "resources");
}

#[test]
fn test_decision_healthy() {
    let fixture = "tests/fixtures/linux/healthy.txt";
    let res = run_pipeline(
        "linux",
        Some(std::path::Path::new(fixture)),
        None,
        None,
        Some("mock"),
        None,
    )
    .unwrap();

    let health = match &res.decisions["health"] {
        ProviderDecision::Choice { value, .. } => value.as_str(),
        _ => panic!("Expected choice for health"),
    };
    assert_eq!(health, "healthy");

    let sev = match &res.decisions["severity"] {
        ProviderDecision::Score { value, .. } => *value,
        _ => panic!("Expected score for severity"),
    };
    assert_eq!(sev, 0);

    let attn = match &res.decisions["needs_attention"] {
        ProviderDecision::Boolean { value, .. } => *value,
        _ => panic!("Expected boolean for needs_attention"),
    };
    assert!(!attn);
}

#[test]
fn test_provider_schema_enforcement_bad_choice() {
    let mut expected = BTreeMap::new();
    expected.insert(
        "health".to_string(),
        DecisionSpec::Choice {
            values: vec!["healthy".to_string(), "unhealthy".to_string()],
        },
    );

    let mut actual = BTreeMap::new();
    actual.insert(
        "health".to_string(),
        ProviderDecision::Choice {
            value: "exploded".to_string(),
            confidence: 0.99,
            probabilities: None,
        },
    );

    let err = validate_provider_response(&expected, &actual).unwrap_err();
    assert!(err.to_string().contains("is not in allowed pack values"));
}

#[test]
fn test_provider_schema_enforcement_bad_score() {
    let mut expected = BTreeMap::new();
    expected.insert(
        "severity".to_string(),
        DecisionSpec::Score {
            min: 0,
            max: 5,
            levels: vec![],
        },
    );

    let mut actual = BTreeMap::new();
    actual.insert(
        "severity".to_string(),
        ProviderDecision::Score {
            value: 99,
            confidence: 0.99,
            probabilities: None,
        },
    );

    let err = validate_provider_response(&expected, &actual).unwrap_err();
    assert!(err.to_string().contains("is outside allowed range"));
}

#[test]
fn test_provider_schema_enforcement_bad_confidence() {
    let mut expected = BTreeMap::new();
    expected.insert("needs_attention".to_string(), DecisionSpec::Boolean);

    let mut actual = BTreeMap::new();
    actual.insert(
        "needs_attention".to_string(),
        ProviderDecision::Boolean {
            value: true,
            confidence: 1.5, // invalid confidence > 1.0
        },
    );

    let err = validate_provider_response(&expected, &actual).unwrap_err();
    assert!(err
        .to_string()
        .contains("outside the valid [0.0, 1.0] range"));
}
