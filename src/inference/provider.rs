use crate::error::Result;
use crate::inference::types::{InferenceRequest, InferenceResponse};

/// Trait implemented by inference engines (Mock provider, and in future versions real TypeSafe Jev API).
pub trait InferenceProvider: Send + Sync {
    /// Identifier name of this inference provider (e.g. "mock", "typesafe-jev").
    fn name(&self) -> &'static str;

    /// Runs inference on the request and returns structured typed decisions.
    fn infer(&self, request: &InferenceRequest) -> Result<InferenceResponse>;
}
