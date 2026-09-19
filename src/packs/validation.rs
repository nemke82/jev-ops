use semver::Version;
use std::collections::HashSet;

use crate::error::{JevOpsError, Result};
use crate::packs::manifest::{DecisionSpec, PackManifest, CURRENT_API_VERSION, CURRENT_KIND};
use crate::security::limits::{
    ABSOLUTE_MAX_INPUT_BYTES, MAX_CHOICE_VALUES_COUNT, MAX_DECISION_COUNT, MAX_INSTRUCTIONS_BYTES,
};
use crate::security::validation::validate_pack_name;

/// Validates a parsed pack manifest against all semantic and security rules.
pub fn validate_manifest(manifest: &PackManifest) -> Result<()> {
    let mut errors = Vec::new();

    // 1. API Version & Kind
    if manifest.api_version != CURRENT_API_VERSION {
        errors.push(format!(
            "unsupported api_version '{}'. Expected '{}'",
            manifest.api_version, CURRENT_API_VERSION
        ));
    }

    if manifest.kind != CURRENT_KIND {
        errors.push(format!(
            "unsupported kind '{}'. Expected '{}'",
            manifest.kind, CURRENT_KIND
        ));
    }

    // 2. Metadata
    if let Err(e) = validate_pack_name(&manifest.metadata.name) {
        errors.push(format!("metadata.name: {}", e));
    }

    if let Err(e) = Version::parse(&manifest.metadata.version) {
        errors.push(format!(
            "metadata.version: '{}' is not valid semantic version ({})",
            manifest.metadata.version, e
        ));
    }

    if manifest.metadata.description.trim().is_empty() {
        errors.push("metadata.description: description cannot be empty".to_string());
    }

    if manifest.metadata.author.trim().is_empty() {
        errors.push("metadata.author: author cannot be empty".to_string());
    }

    // 3. Spec: Input
    if manifest.spec.input.input_type != "text" {
        errors.push(format!(
            "spec.input.type: '{}' is unsupported. Only 'text' is supported in v0.1",
            manifest.spec.input.input_type
        ));
    }

    if let Some(max_b) = manifest.spec.input.max_bytes {
        if max_b == 0 {
            errors.push("spec.input.max_bytes: cannot be 0".to_string());
        } else if max_b > ABSOLUTE_MAX_INPUT_BYTES {
            errors.push(format!(
                "spec.input.max_bytes: {} exceeds absolute maximum of {} bytes (10 MiB)",
                max_b, ABSOLUTE_MAX_INPUT_BYTES
            ));
        }
    }

    // 4. Spec: Instructions
    if manifest.spec.instructions.trim().is_empty() {
        errors.push("spec.instructions: instructions cannot be empty".to_string());
    } else if manifest.spec.instructions.len() > MAX_INSTRUCTIONS_BYTES {
        errors.push(format!(
            "spec.instructions: size {} bytes exceeds maximum of {} bytes (32 KiB)",
            manifest.spec.instructions.len(),
            MAX_INSTRUCTIONS_BYTES
        ));
    }

    // 5. Spec: Decisions
    if manifest.spec.decisions.is_empty() {
        errors.push("spec.decisions: at least one decision must be defined".to_string());
    } else if manifest.spec.decisions.len() > MAX_DECISION_COUNT {
        errors.push(format!(
            "spec.decisions: count {} exceeds maximum allowed of {}",
            manifest.spec.decisions.len(),
            MAX_DECISION_COUNT
        ));
    }

    for (name, spec) in &manifest.spec.decisions {
        if name.trim().is_empty() {
            errors.push("spec.decisions: decision name cannot be empty".to_string());
            continue;
        }

        if !name
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || c == '_' || c == '-')
        {
            errors.push(format!(
                "spec.decisions.{}: decision name must contain only alphanumeric characters, '_', or '-'",
                name
            ));
        }

        match spec {
            DecisionSpec::Choice { values } => {
                if values.is_empty() {
                    errors.push(format!(
                        "spec.decisions.{}: choice decision must define at least one value",
                        name
                    ));
                } else if values.len() > MAX_CHOICE_VALUES_COUNT {
                    errors.push(format!(
                        "spec.decisions.{}: choice values count {} exceeds maximum of {}",
                        name,
                        values.len(),
                        MAX_CHOICE_VALUES_COUNT
                    ));
                }

                let mut seen = HashSet::new();
                for val in values {
                    if val.trim().is_empty() {
                        errors.push(format!(
                            "spec.decisions.{}: choice value cannot be empty",
                            name
                        ));
                    }
                    if !seen.insert(val) {
                        errors.push(format!(
                            "spec.decisions.{}: duplicate choice value '{}'",
                            name, val
                        ));
                    }
                }
            }
            DecisionSpec::Score { min, max } => {
                if min >= max {
                    errors.push(format!(
                        "spec.decisions.{}:\nscore minimum ({}) must be lower than maximum ({})",
                        name, min, max
                    ));
                }
            }
            DecisionSpec::Boolean => {
                // Boolean has no inner parameters
            }
        }
    }

    if !errors.is_empty() {
        return Err(JevOpsError::InvalidPack(errors.join("\n")));
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::packs::manifest::{InputSpec, PackMetadata, PackSpec};
    use std::collections::BTreeMap;

    fn make_valid_manifest() -> PackManifest {
        let mut decisions = BTreeMap::new();
        decisions.insert(
            "health".to_string(),
            DecisionSpec::Choice {
                values: vec!["healthy".to_string(), "unhealthy".to_string()],
            },
        );
        decisions.insert(
            "severity".to_string(),
            DecisionSpec::Score { min: 0, max: 5 },
        );
        decisions.insert("needs_attention".to_string(), DecisionSpec::Boolean);

        PackManifest {
            api_version: CURRENT_API_VERSION.to_string(),
            kind: CURRENT_KIND.to_string(),
            metadata: PackMetadata {
                name: "linux".to_string(),
                version: "0.1.0".to_string(),
                description: "Linux diagnostics".to_string(),
                author: "team".to_string(),
            },
            spec: PackSpec {
                input: InputSpec::default(),
                decisions,
                instructions: "Analyze Linux logs.".to_string(),
            },
        }
    }

    #[test]
    fn test_valid_manifest() {
        let m = make_valid_manifest();
        assert!(validate_manifest(&m).is_ok());
    }

    #[test]
    fn test_invalid_score_range() {
        let mut m = make_valid_manifest();
        m.spec.decisions.insert(
            "severity".to_string(),
            DecisionSpec::Score { min: 5, max: 3 },
        );
        let res = validate_manifest(&m);
        assert!(res.is_err());
        assert!(res
            .unwrap_err()
            .to_string()
            .contains("score minimum (5) must be lower than maximum (3)"));
    }

    #[test]
    fn test_duplicate_choice_values() {
        let mut m = make_valid_manifest();
        m.spec.decisions.insert(
            "health".to_string(),
            DecisionSpec::Choice {
                values: vec!["ok".to_string(), "ok".to_string()],
            },
        );
        let res = validate_manifest(&m);
        assert!(res.is_err());
        assert!(res
            .unwrap_err()
            .to_string()
            .contains("duplicate choice value"));
    }
}
