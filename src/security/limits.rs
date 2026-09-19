//! Security limit constants for input, manifests, and pack specifications.

/// Default maximum input size for analysis (1 MiB).
pub const DEFAULT_MAX_INPUT_BYTES: usize = 1024 * 1024;

/// Absolute maximum input size any pack can configure (10 MiB).
pub const ABSOLUTE_MAX_INPUT_BYTES: usize = 10 * 1024 * 1024;

/// Maximum allowable size for a pack manifest file (256 KiB).
pub const MAX_MANIFEST_BYTES: usize = 256 * 1024;

/// Maximum allowable size for pack instructions (32 KiB).
pub const MAX_INSTRUCTIONS_BYTES: usize = 32 * 1024;

/// Maximum number of decisions in a single pack (64).
pub const MAX_DECISION_COUNT: usize = 64;

/// Maximum number of choice values in a Choice decision (128).
pub const MAX_CHOICE_VALUES_COUNT: usize = 128;

/// Maximum number of levels in a Score decision (`max - min + 1`), as accepted by TypeSafe (10).
pub const MAX_SCORE_LEVELS: i64 = 10;

/// Maximum length of a pack name (64 chars).
pub const MAX_PACK_NAME_LEN: usize = 64;
