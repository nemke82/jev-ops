use std::collections::BTreeMap;
use serde::{Deserialize, Serialize};

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
}
