use crate::engine::pipeline::PipelineResult;
use crate::error::Result;
use crate::output::types::{AnalysisOutput, OUTPUT_SCHEMA_VERSION};

/// Renders the pipeline result as a clean, machine-consumable JSON string.
pub fn render_json(result: &PipelineResult) -> Result<String> {
    let output = AnalysisOutput {
        schema_version: OUTPUT_SCHEMA_VERSION.to_string(),
        pack: result.pack.clone(),
        provider: result.provider.clone(),
        input: result.input.clone(),
        decisions: result.decisions.clone(),
        min_confidence: result.min_confidence,
        low_confidence: result.low_confidence_decisions(),
    };

    let json = serde_json::to_string_pretty(&output)?;
    Ok(json)
}
