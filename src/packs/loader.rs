use std::fs;
use std::path::{Path, PathBuf};

use crate::error::{JevOpsError, Result};
use crate::packs::manifest::PackManifest;
use crate::packs::validation::validate_manifest;
use crate::security::limits::MAX_MANIFEST_BYTES;
use crate::security::validation::validate_text_input;

/// Resolve manifest file path if a directory is provided.
pub fn resolve_manifest_path(path: &Path) -> Result<PathBuf> {
    if path.is_file() {
        return Ok(path.to_path_buf());
    }

    if path.is_dir() {
        let yaml = path.join("pack.yaml");
        if yaml.is_file() {
            return Ok(yaml);
        }
        let yml = path.join("pack.yml");
        if yml.is_file() {
            return Ok(yml);
        }
        return Err(JevOpsError::PackNotFound {
            name: path.display().to_string(),
            searched: format!("{}/pack.yaml, {}/pack.yml", path.display(), path.display()),
        });
    }

    Err(JevOpsError::InvalidPack(format!(
        "Path '{}' does not exist or is not accessible",
        path.display()
    )))
}

/// Loads and validates a pack manifest from a file or directory path.
pub fn load_manifest(path: &Path) -> Result<PackManifest> {
    let resolved_path = resolve_manifest_path(path)?;

    let metadata = fs::metadata(&resolved_path).map_err(|e| {
        JevOpsError::InvalidPack(format!("Cannot read '{}': {}", resolved_path.display(), e))
    })?;

    if metadata.len() > MAX_MANIFEST_BYTES as u64 {
        return Err(JevOpsError::InvalidPack(format!(
            "Pack manifest '{}' size ({} bytes) exceeds limit of {} bytes (256 KiB)",
            resolved_path.display(),
            metadata.len(),
            MAX_MANIFEST_BYTES
        )));
    }

    let bytes = fs::read(&resolved_path).map_err(|e| {
        JevOpsError::InvalidPack(format!(
            "Failed to read '{}': {}",
            resolved_path.display(),
            e
        ))
    })?;

    let content = validate_text_input(&bytes).map_err(|e| {
        JevOpsError::InvalidPack(format!(
            "Pack manifest '{}' is not valid UTF-8: {}",
            resolved_path.display(),
            e
        ))
    })?;

    let manifest: PackManifest =
        serde_yaml::from_str(content).map_err(|e| JevOpsError::PackValidation {
            path: resolved_path.clone(),
            details: format!("YAML parsing error: {}", e),
        })?;

    validate_manifest(&manifest).map_err(|e| JevOpsError::PackValidation {
        path: resolved_path,
        details: e.to_string(),
    })?;

    Ok(manifest)
}

/// Parses and validates a pack manifest from a YAML string in memory.
pub fn load_manifest_from_str(content: &str, virtual_name: &str) -> Result<PackManifest> {
    if content.len() > MAX_MANIFEST_BYTES {
        return Err(JevOpsError::InvalidPack(format!(
            "Pack manifest '{}' size exceeds limit of {} bytes",
            virtual_name, MAX_MANIFEST_BYTES
        )));
    }

    let manifest: PackManifest =
        serde_yaml::from_str(content).map_err(|e| JevOpsError::PackValidation {
            path: PathBuf::from(virtual_name),
            details: format!("YAML parsing error: {}", e),
        })?;

    validate_manifest(&manifest).map_err(|e| JevOpsError::PackValidation {
        path: PathBuf::from(virtual_name),
        details: e.to_string(),
    })?;

    Ok(manifest)
}
