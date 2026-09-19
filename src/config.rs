use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Config {
    #[serde(default)]
    pub pack_dirs: Vec<PathBuf>,

    #[serde(default)]
    pub default_provider: Option<String>,
}

impl Config {
    /// Load config from ~/.config/jev-ops/config.toml if it exists.
    pub fn load() -> Self {
        if let Some(config_dir) = dirs::config_dir() {
            let config_path = config_dir.join("jev-ops").join("config.toml");
            if config_path.exists() {
                if let Ok(content) = std::fs::read_to_string(&config_path) {
                    // Simple basic parsing or fallback to default
                    if let Ok(parsed) = toml_simple_parse(&content) {
                        return parsed;
                    }
                }
            }
        }
        Self::default()
    }
}

// Minimal parser to avoid extra heavy toml dependency if not needed,
// or we can just parse lines or use serde.
fn toml_simple_parse(_content: &str) -> std::result::Result<Config, ()> {
    // For v0.1 minimal config
    Ok(Config::default())
}
