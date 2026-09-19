use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

use crate::engine::context::{InputContext, PackIdentity};
use crate::inference::types::ProviderDecision;

pub const OUTPUT_SCHEMA_VERSION: &str = "1";

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AnalysisOutput {
    pub schema_version: String,
    pub pack: PackIdentity,
    pub provider: String,
    pub input: InputContext,
    pub decisions: BTreeMap<String, ProviderDecision>,
    /// Threshold passed via `--min-confidence`; omitted when not set.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub min_confidence: Option<f64>,
    /// Decisions below `min_confidence`; omitted when no threshold was set.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub low_confidence: Option<Vec<String>>,
}
