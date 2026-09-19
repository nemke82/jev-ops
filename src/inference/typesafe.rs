use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::thread;
use std::time::Duration;

use crate::error::{JevOpsError, Result};
use crate::inference::provider::InferenceProvider;
use crate::inference::types::{InferenceRequest, InferenceResponse, ProviderDecision};
use crate::packs::manifest::DecisionSpec;

pub const DEFAULT_TYPESAFE_API_URL: &str = "https://api.typesafe.ai/v1";
pub const DEFAULT_TYPESAFE_MODEL: &str = "jev-latest";

#[derive(Debug, Clone)]
pub struct TypeSafeJevProvider {
    api_key: String,
    base_url: String,
    model: String,
    max_retries: usize,
}

impl TypeSafeJevProvider {
    pub fn new(api_key: String) -> Self {
        let base_url = std::env::var("TYPESAFE_API_URL")
            .unwrap_or_else(|_| DEFAULT_TYPESAFE_API_URL.to_string());
        let model =
            std::env::var("TYPESAFE_MODEL").unwrap_or_else(|_| DEFAULT_TYPESAFE_MODEL.to_string());

        Self {
            api_key,
            base_url,
            model,
            max_retries: 3,
        }
    }

    pub fn with_custom(api_key: String, base_url: Option<String>, model: Option<String>) -> Self {
        Self {
            api_key,
            base_url: base_url.unwrap_or_else(|| DEFAULT_TYPESAFE_API_URL.to_string()),
            model: model.unwrap_or_else(|| DEFAULT_TYPESAFE_MODEL.to_string()),
            max_retries: 3,
        }
    }
}

/// Structured state: the pack's analysis guidance travels with the input it applies to.
#[derive(Serialize)]
struct DiagnosticState<'a> {
    analysis_guidance: &'a str,
    diagnostic_input: &'a str,
}

#[derive(Serialize)]
struct SystemOneApiRequest<'a> {
    state: DiagnosticState<'a>,
    model: &'a str,
    questions: BTreeMap<String, ApiQuestion>,
}

#[derive(Serialize)]
#[serde(tag = "type", rename_all = "lowercase")]
enum ApiQuestion {
    Choice {
        instructions: String,
        criteria: BTreeMap<String, Option<String>>,
    },
    Score {
        instructions: String,
        criteria: Vec<String>,
    },
    Noul {
        instructions: String,
    },
}

#[derive(Deserialize)]
struct SystemOneApiResponse {
    answers: BTreeMap<String, ApiAnswer>,
}

#[derive(Deserialize)]
#[serde(tag = "type", rename_all = "lowercase")]
enum ApiAnswer {
    Choice {
        choice: String,
        confidence: f64,
        #[serde(default)]
        probabilities: Option<BTreeMap<String, f64>>,
    },
    Score {
        score: f64,
        confidence: f64,
        #[serde(default)]
        probabilities: Option<BTreeMap<String, f64>>,
    },
    Noul {
        noul: f64,
    },
}

/// Tolerance for floating-point error in a probability-weighted score.
const SCORE_EPSILON: f64 = 1e-6;

fn build_question(name: &str, spec: &DecisionSpec) -> ApiQuestion {
    let label = name.replace(['_', '-'], " ");
    match spec {
        DecisionSpec::Choice { values } => ApiQuestion::Choice {
            instructions: format!(
                "Following `analysis_guidance`, which '{}' classification best describes `diagnostic_input`?",
                label
            ),
            criteria: values.iter().map(|v| (v.clone(), None)).collect(),
        },
        DecisionSpec::Score { min, max, levels } => {
            // TypeSafe numbers levels by array position from 0; `levels[i]` is pack value `min + i`.
            let criteria = if levels.is_empty() {
                (*min..=*max)
                    .map(|v| {
                        format!(
                            "'{}' is {} on the {} to {} scale defined in `analysis_guidance`",
                            label, v, min, max
                        )
                    })
                    .collect()
            } else {
                levels.clone()
            };
            ApiQuestion::Score {
                instructions: format!(
                    "Following `analysis_guidance`, rate the '{}' of `diagnostic_input`.",
                    label
                ),
                criteria,
            }
        }
        DecisionSpec::Boolean => ApiQuestion::Noul {
            instructions: format!(
                "Following `analysis_guidance`, does `diagnostic_input` indicate '{}'?",
                label
            ),
        },
    }
}

fn invalid(name: &str, details: String) -> JevOpsError {
    JevOpsError::InvalidProviderResponse(format!("Decision '{}': {}", name, details))
}

/// Converts one TypeSafe answer into a pack decision, mapping score level indices back to pack values.
fn convert_answer(name: &str, spec: &DecisionSpec, answer: &ApiAnswer) -> Result<ProviderDecision> {
    match (spec, answer) {
        (
            DecisionSpec::Choice { .. },
            ApiAnswer::Choice {
                choice,
                confidence,
                probabilities,
            },
        ) => Ok(ProviderDecision::Choice {
            value: choice.clone(),
            confidence: *confidence,
            probabilities: probabilities.clone(),
        }),
        (
            DecisionSpec::Score { min, max, .. },
            ApiAnswer::Score {
                score,
                confidence,
                probabilities,
            },
        ) => {
            let top_index = (max - min) as f64;
            if !score.is_finite() || *score < -SCORE_EPSILON || *score > top_index + SCORE_EPSILON {
                return Err(invalid(
                    name,
                    format!(
                        "score level {} is outside the requested levels [0, {}]",
                        score, top_index
                    ),
                ));
            }
            let index = score.round().clamp(0.0, top_index) as i64;

            let probabilities = probabilities
                .as_ref()
                .map(|probs| {
                    probs
                        .iter()
                        .map(|(level, p)| {
                            let idx: i64 = level
                                .parse()
                                .ok()
                                .filter(|i| (0..=max - min).contains(i))
                                .ok_or_else(|| {
                                    invalid(name, format!("unknown score level key '{}'", level))
                                })?;
                            Ok(((min + idx).to_string(), *p))
                        })
                        .collect::<Result<BTreeMap<_, _>>>()
                })
                .transpose()?;

            Ok(ProviderDecision::Score {
                value: min + index,
                confidence: *confidence,
                probabilities,
            })
        }
        (DecisionSpec::Boolean, ApiAnswer::Noul { noul }) => {
            if !(0.0..=1.0).contains(noul) {
                return Err(invalid(
                    name,
                    format!("noul {} is outside the valid [0.0, 1.0] range", noul),
                ));
            }
            let is_true = *noul >= 0.5;
            Ok(ProviderDecision::Boolean {
                value: is_true,
                confidence: if is_true { *noul } else { 1.0 - *noul },
            })
        }
        _ => Err(invalid(
            name,
            "TypeSafe API answer type does not match pack specification".to_string(),
        )),
    }
}

impl InferenceProvider for TypeSafeJevProvider {
    fn name(&self) -> &'static str {
        "typesafe-jev"
    }

    fn infer(&self, request: &InferenceRequest) -> Result<InferenceResponse> {
        let questions = request
            .decisions
            .iter()
            .map(|(name, spec)| (name.clone(), build_question(name, spec)))
            .collect();

        let api_req = SystemOneApiRequest {
            state: DiagnosticState {
                analysis_guidance: &request.instructions,
                diagnostic_input: &request.input_text,
            },
            model: &self.model,
            questions,
        };

        let endpoint = format!("{}/systemone", self.base_url.trim_end_matches('/'));
        let agent = ureq::AgentBuilder::new()
            .timeout(Duration::from_secs(30))
            .build();

        let mut attempts = 0;
        let api_response: SystemOneApiResponse = loop {
            attempts += 1;
            let response = agent
                .post(&endpoint)
                .set("Authorization", &format!("Bearer {}", self.api_key))
                .set("Content-Type", "application/json")
                .send_json(&api_req);

            match response {
                Ok(resp) => {
                    let parsed: SystemOneApiResponse =
                        resp.into_json().map_err(|e| JevOpsError::ProviderError {
                            provider: self.name().to_string(),
                            details: format!("Failed to parse TypeSafe API response JSON: {}", e),
                        })?;
                    break parsed;
                }
                Err(ureq::Error::Status(status, resp)) => {
                    let err_body = resp.into_string().unwrap_or_default();
                    if (status == 429 || status == 529) && attempts <= self.max_retries {
                        tracing::warn!(
                            "TypeSafe API returned HTTP {} (overload/rate-limit). Retrying attempt {}/{}...",
                            status, attempts, self.max_retries
                        );
                        let backoff_secs = 2u64.pow(attempts as u32);
                        thread::sleep(Duration::from_secs(backoff_secs));
                        continue;
                    }

                    return Err(JevOpsError::ProviderError {
                        provider: self.name().to_string(),
                        details: format!("HTTP {} from TypeSafe API: {}", status, err_body),
                    });
                }
                Err(ureq::Error::Transport(transport_err)) => {
                    if attempts <= self.max_retries {
                        tracing::warn!(
                            "Network transport error: {}. Retrying attempt {}/{}...",
                            transport_err,
                            attempts,
                            self.max_retries
                        );
                        thread::sleep(Duration::from_secs(1));
                        continue;
                    }

                    return Err(JevOpsError::ProviderError {
                        provider: self.name().to_string(),
                        details: format!("Connection failed: {}", transport_err),
                    });
                }
            }
        };

        let mut decisions = BTreeMap::new();
        for (name, spec) in &request.decisions {
            let answer = api_response.answers.get(name).ok_or_else(|| {
                JevOpsError::InvalidProviderResponse(format!(
                    "TypeSafe Jev API response did not contain answer for '{}'",
                    name
                ))
            })?;
            decisions.insert(name.clone(), convert_answer(name, spec, answer)?);
        }

        Ok(InferenceResponse {
            provider: format!("typesafe-jev ({})", self.model),
            decisions,
        })
    }
}
