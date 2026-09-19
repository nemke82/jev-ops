use std::fs;
use std::path::{Path, PathBuf};

use crate::error::{JevOpsError, Result};
use crate::security::validation::validate_pack_name;

/// Scaffolds a new diagnostic pack template into the specified directory.
pub fn scaffold_pack(name: &str, target_dir: Option<&Path>) -> Result<PathBuf> {
    validate_pack_name(name)?;

    let base_dir = target_dir
        .map(|p| p.to_path_buf())
        .unwrap_or_else(|| PathBuf::from("./packs"));

    let pack_dir = base_dir.join(name);
    if pack_dir.exists() {
        return Err(JevOpsError::InvalidPack(format!(
            "Target directory '{}' already exists",
            pack_dir.display()
        )));
    }

    fs::create_dir_all(&pack_dir).map_err(|e| {
        JevOpsError::InvalidPack(format!(
            "Failed to create directory '{}': {}",
            pack_dir.display(),
            e
        ))
    })?;

    let pack_yaml_content = format!(
        r#"api_version: "jev-ops/v1"
kind: "DiagnosticPack"

metadata:
  name: "{name}"
  version: "0.1.0"
  description: "Diagnostic pack for {name}"
  author: "Community"

spec:
  input:
    type: "text"
    max_bytes: 1048576

  # TypeSafe System One atomic decisions
  decisions:
    health:
      type: "choice"
      values:
        - healthy
        - degraded
        - unhealthy
        - unknown

    severity:
      type: "score"
      min: 0
      max: 5
      # One description per level, from min to max (optional but recommended)
      levels:
        - "Normal: no errors or anomalies in the signals"
        - "Informational: notable events with no impact on service"
        - "Minor: isolated errors or warnings with no user-visible impact"
        - "Degraded: partial impairment affecting some users or requests"
        - "Serious: a major component is failing; urgent action needed"
        - "Critical: outage, data loss risk, or security compromise"

    needs_attention:
      type: "boolean"

  instructions: |
    Analyze the supplied diagnostic signals for {name}.
    Classify the operational health, severity level, and whether immediate
    operator intervention is needed.
"#,
        name = name
    );

    let manifest_path = pack_dir.join("pack.yaml");
    fs::write(&manifest_path, pack_yaml_content).map_err(|e| {
        JevOpsError::InvalidPack(format!(
            "Failed to write template manifest '{}': {}",
            manifest_path.display(),
            e
        ))
    })?;

    Ok(manifest_path)
}
