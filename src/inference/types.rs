use std::collections::BTreeMap;
use serde::{Deserialize, Serialize};

use crate::packs::manifest::DecisionSpec;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct InferenceRequest {
    pub pack_name: String,
    pub pack_version: String,
    pub instructions: String,
    pub decisions: BTreeMap<String, DecisionSpec>,
    pub input_text: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct InferenceResponse {
    pub provider: String,
    pub decisions: BTreeMap<String, ProviderDecision>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "lowercase")]
pub enum ProviderDecision {
    Choice {
        value: String,
        confidence: f64,
        #[serde(skip_serializing_if = "Option::is_none")]
        probabilities: Option<BTreeMap<String, f64>>,
    },
    Score {
        value: i64,
        confidence: f64,
        #[serde(skip_serializing_if = "Option::is_none")]
        probabilities: Option<BTreeMap<String, f64>>,
    },
    Boolean {
        value: bool,
        confidence: f64,
    },
}

impl ProviderDecision {
    pub fn confidence(&self) -> f64 {
        match self {
            Self::Choice { confidence, .. } => *confidence,
            Self::Score { confidence, .. } => *confidence,
            Self::Boolean { confidence, .. } => *confidence,
        }
    }

    pub fn type_name(&self) -> &'static str {
        match self {
            Self::Choice { .. } => "choice",
            Self::Score { .. } => "score",
            Self::Boolean { .. } => "boolean",
        }
    }

    pub fn value_display(&self) -> String {
        match self {
            Self::Choice { value, .. } => value.clone(),
            Self::Score { value, .. } => value.to_string(),
            Self::Boolean { value, .. } => {
                if *value {
                    "yes".to_string()
                } else {
                    "no".to_string()
                }
            }
        }
    }
}
