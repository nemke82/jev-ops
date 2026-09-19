use std::collections::BTreeMap;
use std::path::Path;

use crate::engine::context::{InputContext, PackIdentity};
use crate::engine::input::read_input;
use crate::error::{JevOpsError, Result};
use crate::inference::resolve_provider;
use crate::inference::types::{InferenceRequest, ProviderDecision};
use crate::packs::discovery::find_pack;
use crate::packs::manifest::DecisionSpec;
use crate::security::limits::{ABSOLUTE_MAX_INPUT_BYTES, DEFAULT_MAX_INPUT_BYTES};

pub struct PipelineResult {
    pub pack: PackIdentity,
    pub provider: String,
    pub input: InputContext,
    pub decisions: BTreeMap<String, ProviderDecision>,
    pub min_confidence: Option<f64>,
}

/// Executes the full end-to-end diagnostic pipeline.
pub fn run_pipeline(
    pack_name: &str,
    input_path: Option<&Path>,
    custom_pack_dir: Option<&Path>,
    min_confidence: Option<f64>,
    provider_name: Option<&str>,
    api_key: Option<&str>,
) -> Result<PipelineResult> {
    // 1. Resolve & validate pack manifest
    let (_manifest_path, manifest) = find_pack(pack_name, custom_pack_dir)?;

    // 2. Determine input byte limits
    let max_bytes = manifest
        .spec
        .input
        .max_bytes
        .unwrap_or(DEFAULT_MAX_INPUT_BYTES)
        .min(ABSOLUTE_MAX_INPUT_BYTES);

    // 3. Read input
    let input_data = read_input(input_path, max_bytes)?;

    // 4. Construct InferenceRequest
    let request = InferenceRequest {
        pack_name: manifest.metadata.name.clone(),
        pack_version: manifest.metadata.version.clone(),
        instructions: manifest.spec.instructions.clone(),
        decisions: manifest.spec.decisions.clone(),
        input_text: input_data.text,
    };

    // 5. Query resolved provider (mock or TypeSafe Jev)
    let provider = resolve_provider(provider_name, api_key)?;
    let response = provider.infer(&request)?;

    // 6. Strictly validate provider response against pack schema
    validate_provider_response(&manifest.spec.decisions, &response.decisions)?;

    Ok(PipelineResult {
        pack: PackIdentity {
            name: manifest.metadata.name,
            version: manifest.metadata.version,
        },
        provider: response.provider,
        input: input_data.context,
        decisions: response.decisions,
        min_confidence,
    })
}

/// Validates that an untrusted provider response strictly satisfies the pack's decision schema.
pub fn validate_provider_response(
    expected_decisions: &BTreeMap<String, DecisionSpec>,
    actual_decisions: &BTreeMap<String, ProviderDecision>,
) -> Result<()> {
    for (name, spec) in expected_decisions {
        let decision = actual_decisions.get(name).ok_or_else(|| {
            JevOpsError::InvalidProviderResponse(format!(
                "Provider failed to return declared decision '{}'",
                name
            ))
        })?;

        let conf = decision.confidence();
        if !(0.0..=1.0).contains(&conf) {
            return Err(JevOpsError::InvalidProviderResponse(format!(
                "Decision '{}' confidence {} is outside the valid [0.0, 1.0] range",
                name, conf
            )));
        }

        match (spec, decision) {
            (DecisionSpec::Choice { values }, ProviderDecision::Choice { value, .. }) => {
                if !values.contains(value) {
                    return Err(JevOpsError::InvalidProviderResponse(format!(
                        "Decision '{}': choice value '{}' is not in allowed pack values: {:?}",
                        name, value, values
                    )));
                }
            }
            (DecisionSpec::Score { min, max }, ProviderDecision::Score { value, .. }) => {
                if value < min || value > max {
                    return Err(JevOpsError::InvalidProviderResponse(format!(
                        "Decision '{}': score {} is outside allowed range [{}, {}]",
                        name, value, min, max
                    )));
                }
            }
            (DecisionSpec::Boolean, ProviderDecision::Boolean { .. }) => {
                // Valid boolean
            }
            _ => {
                return Err(JevOpsError::InvalidProviderResponse(format!(
                    "Decision '{}': type mismatch. Pack expects '{}', but provider returned '{}'",
                    name,
                    spec.type_name(),
                    decision.type_name()
                )));
            }
        }
    }

    // Check for undeclared decisions returned by provider
    for name in actual_decisions.keys() {
        if !expected_decisions.contains_key(name) {
            return Err(JevOpsError::InvalidProviderResponse(format!(
                "Provider returned undeclared decision '{}' not defined in pack schema",
                name
            )));
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_response_mismatch_choice() {
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
                value: "catastrophic".to_string(),
                confidence: 0.95,
                probabilities: None,
            },
        );

        let res = validate_provider_response(&expected, &actual);
        assert!(res.is_err());
        assert!(res
            .unwrap_err()
            .to_string()
            .contains("not in allowed pack values"));
    }

    #[test]
    fn test_validate_response_score_out_of_range() {
        let mut expected = BTreeMap::new();
        expected.insert(
            "severity".to_string(),
            DecisionSpec::Score { min: 0, max: 5 },
        );

        let mut actual = BTreeMap::new();
        actual.insert(
            "severity".to_string(),
            ProviderDecision::Score {
                value: 8,
                confidence: 0.95,
                probabilities: None,
            },
        );

        let res = validate_provider_response(&expected, &actual);
        assert!(res.is_err());
        assert!(res
            .unwrap_err()
            .to_string()
            .contains("score 8 is outside allowed range"));
    }
}
