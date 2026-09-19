pub mod mock;
pub mod provider;
pub mod types;
pub mod typesafe;

use crate::error::{JevOpsError, Result};
pub use mock::*;
pub use provider::*;
pub use types::*;
pub use typesafe::*;

/// Resolves the appropriate inference provider based on CLI flags and environment variables.
pub fn resolve_provider(
    requested: Option<&str>,
    api_key: Option<&str>,
) -> Result<Box<dyn InferenceProvider>> {
    match requested {
        Some("mock") => Ok(Box::new(MockInferenceProvider::new())),
        Some("typesafe") | Some("typesafe-jev") => {
            let key = api_key
                .map(|s| s.to_string())
                .or_else(|| std::env::var("TYPESAFE_API_KEY").ok())
                .filter(|s| !s.trim().is_empty())
                .ok_or_else(|| JevOpsError::ProviderError {
                    provider: "typesafe-jev".to_string(),
                    details: "Missing API key. Provide --api-key or set the TYPESAFE_API_KEY environment variable.".to_string(),
                })?;
            Ok(Box::new(TypeSafeJevProvider::new(key)))
        }
        Some(other) => Err(JevOpsError::Cli(format!(
            "Unknown inference provider '{}'. Supported providers: 'mock', 'typesafe'",
            other
        ))),
        None => {
            // Auto-detect the live provider from the API key. Never fall back to the mock
            // silently: automation acting on keyword heuristics must be an explicit choice.
            let key = api_key
                .map(|s| s.to_string())
                .or_else(|| std::env::var("TYPESAFE_API_KEY").ok())
                .filter(|s| !s.trim().is_empty())
                .ok_or_else(|| JevOpsError::ProviderError {
                    provider: "typesafe-jev".to_string(),
                    details: "No API key found. Provide --api-key or set TYPESAFE_API_KEY, or pass --provider mock for offline heuristic output.".to_string(),
                })?;
            Ok(Box::new(TypeSafeJevProvider::new(key)))
        }
    }
}
