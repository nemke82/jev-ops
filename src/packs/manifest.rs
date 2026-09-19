use std::collections::BTreeMap;
use serde::{Deserialize, Serialize};

pub const CURRENT_API_VERSION: &str = "jev-ops/v1";
pub const CURRENT_KIND: &str = "DiagnosticPack";

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PackManifest {
    pub api_version: String,
    pub kind: String,
    pub metadata: PackMetadata,
    pub spec: PackSpec,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PackMetadata {
    pub name: String,
    pub version: String,
    pub description: String,
    pub author: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PackSpec {
    #[serde(default)]
    pub input: InputSpec,
    pub decisions: BTreeMap<String, DecisionSpec>,
    pub instructions: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct InputSpec {
    #[serde(rename = "type", default = "default_input_type")]
    pub input_type: String,
    #[serde(default)]
    pub max_bytes: Option<usize>,
}

fn default_input_type() -> String {
    "text".to_string()
}

impl Default for InputSpec {
    fn default() -> Self {
        Self {
            input_type: default_input_type(),
            max_bytes: None,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "lowercase")]
pub enum DecisionSpec {
    Choice {
        values: Vec<String>,
    },
    Score {
        min: i64,
        max: i64,
    },
    Boolean,
}

impl DecisionSpec {
    pub fn type_name(&self) -> &'static str {
        match self {
            Self::Choice { .. } => "choice",
            Self::Score { .. } => "score",
            Self::Boolean => "boolean",
        }
    }
}
