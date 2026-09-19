use std::collections::BTreeMap;
use std::thread;
use std::time::Duration;
use serde::{Deserialize, Serialize};

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
        let model = std::env::var("TYPESAFE_MODEL")
            .unwrap_or_else(|_| DEFAULT_TYPESAFE_MODEL.to_string());

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

#[derive(Serialize)]
struct SystemOneApiRequest<'a> {
    state: &'a str,
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

impl InferenceProvider for TypeSafeJevProvider {
    fn name(&self) -> &'static str {
        "typesafe-jev"
    }

    fn infer(&self, request: &InferenceRequest) -> Result<InferenceResponse> {
        let mut questions = BTreeMap::new();

        for (name, spec) in &request.decisions {
            let question = match spec {
                DecisionSpec::Choice { values } => {
                    let mut criteria = BTreeMap::new();
                    for v in values {
                        criteria.insert(v.clone(), None);
                    }
                    ApiQuestion::Choice {
                        instructions: format!(
                            "Determine the appropriate '{}' classification from the operational state.",
                            name
                        ),
                        criteria,
                    }
                }
                DecisionSpec::Score { min, max } => {
                    let mut levels = Vec::new();
                    for i in *min..=*max {
                        levels.push(format!("Level {}", i));
                    }
                    ApiQuestion::Score {
                        instructions: format!(
                            "Rate the operational '{}' severity level from {} to {}.",
                            name, min, max
                        ),
                        criteria: levels,
                    }
                }
                DecisionSpec::Boolean => ApiQuestion::Noul {
                    instructions: format!(
                        "Based on the diagnostic output, is the statement '{}' true?",
                        name.replace('_', " ")
                    ),
                },
            };
            questions.insert(name.clone(), question);
        }

        let api_req = SystemOneApiRequest {
            state: &request.input_text,
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
                    let parsed: SystemOneApiResponse = resp.into_json().map_err(|e| {
                        JevOpsError::ProviderError {
                            provider: self.name().to_string(),
                            details: format!("Failed to parse TypeSafe API response JSON: {}", e),
                        }
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
                            transport_err, attempts, self.max_retries
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

        // Convert ApiAnswer to ProviderDecision
        let mut decisions = BTreeMap::new();
        for (name, spec) in &request.decisions {
            let answer = api_response.answers.get(name).ok_or_else(|| {
                JevOpsError::InvalidProviderResponse(format!(
                    "TypeSafe Jev API response did not contain answer for '{}'",
                    name
                ))
            })?;

            let decision = match (spec, answer) {
                (DecisionSpec::Choice { .. }, ApiAnswer::Choice { choice, confidence, probabilities }) => {
                    ProviderDecision::Choice {
                        value: choice.clone(),
                        confidence: *confidence,
                        probabilities: probabilities.clone(),
                    }
                }
                (DecisionSpec::Score { min, max }, ApiAnswer::Score { score, confidence, probabilities }) => {
                    let rounded = score.round() as i64;
                    let clamped = rounded.max(*min).min(*max);
                    ProviderDecision::Score {
                        value: clamped,
                        confidence: *confidence,
                        probabilities: probabilities.clone(),
                    }
                }
                (DecisionSpec::Boolean, ApiAnswer::Noul { noul }) => {
                    let is_true = *noul >= 0.5;
                    let conf = if is_true { *noul } else { 1.0 - *noul };
                    ProviderDecision::Boolean {
                        value: is_true,
                        confidence: conf,
                    }
                }
                _ => {
                    return Err(JevOpsError::InvalidProviderResponse(format!(
                        "Decision '{}': TypeSafe API answer type does not match pack specification",
                        name
                    )));
                }
            };

            decisions.insert(name.clone(), decision);
        }

        Ok(InferenceResponse {
            provider: format!("typesafe-jev ({})", self.model),
            decisions,
        })
    }
}
