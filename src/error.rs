use std::path::PathBuf;
use thiserror::Error;

/// Exit codes for jev-ops CLI.
pub mod exit_codes {
    pub const SUCCESS: i32 = 0;
    pub const GENERAL_ERROR: i32 = 1;
    pub const INVALID_CLI_USAGE: i32 = 2;
    pub const INVALID_PACK: i32 = 3;
    pub const INVALID_INPUT: i32 = 4;
    pub const PROVIDER_FAILURE: i32 = 5;
    pub const INVALID_PROVIDER_RESPONSE: i32 = 6;
}

#[derive(Debug, Error)]
pub enum JevOpsError {
    #[error("Invalid CLI argument or option: {0}")]
    Cli(String),

    #[error("Diagnostic pack '{name}' not found. Searched in: {searched}")]
    PackNotFound {
        name: String,
        searched: String,
    },

    #[error("Invalid pack '{path}':\n{details}")]
    PackValidation {
        path: PathBuf,
        details: String,
    },

    #[error("Pack error: {0}")]
    InvalidPack(String),

    #[error("Input error: {0}")]
    Input(String),

    #[error("Input too large ({bytes} bytes exceeds limit of {max_bytes} bytes)")]
    InputTooLarge {
        bytes: usize,
        max_bytes: usize,
    },

    #[error("Input is empty. Provide diagnostic data via stdin or --input")]
    EmptyInput,

    #[error("Invalid UTF-8 in input: {0}")]
    InvalidUtf8(String),

    #[error("Binary or non-text data detected in input")]
    BinaryContentDetected,

    #[error("Inference provider '{provider}' failure: {details}")]
    ProviderError {
        provider: String,
        details: String,
    },

    #[error("Invalid provider response against pack schema: {0}")]
    InvalidProviderResponse(String),

    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    #[error("YAML error: {0}")]
    Yaml(#[from] serde_yaml::Error),

    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),

    #[error("{0}")]
    General(String),
}

impl JevOpsError {
    pub fn exit_code(&self) -> i32 {
        match self {
            Self::Cli(_) => exit_codes::INVALID_CLI_USAGE,
            Self::PackNotFound { .. }
            | Self::PackValidation { .. }
            | Self::InvalidPack(_)
            | Self::Yaml(_) => exit_codes::INVALID_PACK,
            Self::Input(_)
            | Self::InputTooLarge { .. }
            | Self::EmptyInput
            | Self::InvalidUtf8(_)
            | Self::BinaryContentDetected => exit_codes::INVALID_INPUT,
            Self::ProviderError { .. } => exit_codes::PROVIDER_FAILURE,
            Self::InvalidProviderResponse(_) => exit_codes::INVALID_PROVIDER_RESPONSE,
            Self::Io(_) | Self::Json(_) | Self::General(_) => exit_codes::GENERAL_ERROR,
        }
    }
}

pub type Result<T> = std::result::Result<T, JevOpsError>;
