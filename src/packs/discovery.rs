use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use crate::error::{JevOpsError, Result};
use crate::packs::builtin::{get_builtin_pack, BUILTIN_PACKS};
use crate::packs::loader::{load_manifest, load_manifest_from_str};
use crate::packs::manifest::PackManifest;
use crate::security::validation::validate_pack_name;

#[derive(Debug, Clone)]
pub struct DiscoveredPack {
    pub name: String,
    pub path: PathBuf,
    pub manifest: PackManifest,
}

/// Returns the list of standard pack search directories in precedence order.
pub fn search_directories(custom_dir: Option<&Path>) -> Vec<PathBuf> {
    let mut dirs = Vec::new();

    // 1. Explicit CLI --pack-dir takes highest precedence
    if let Some(custom) = custom_dir {
        dirs.push(custom.to_path_buf());
    }

    // 2. Current working directory ./packs/
    dirs.push(PathBuf::from("./packs"));

    // 3. User config ~/.config/jev-ops/packs/
    if let Some(config_dir) = dirs::config_dir() {
        dirs.push(config_dir.join("jev-ops").join("packs"));
    }

    // 4. User data ~/.local/share/jev-ops/packs/
    if let Some(data_dir) = dirs::data_dir() {
        dirs.push(data_dir.join("jev-ops").join("packs"));
    }

    // 5. System wide /usr/share/jev-ops/packs/
    dirs.push(PathBuf::from("/usr/share/jev-ops/packs"));

    dirs
}

/// Locates and loads a pack by name according to search precedence.
/// Checks filesystem search directories first, then falls back to compiled-in standard packs.
pub fn find_pack(name: &str, custom_dir: Option<&Path>) -> Result<(PathBuf, PackManifest)> {
    // Reject names like "../x" or "/etc/x" before they are joined onto search paths.
    validate_pack_name(name)?;

    let dirs = search_directories(custom_dir);

    for dir in &dirs {
        if !dir.is_dir() {
            continue;
        }

        // Candidate 1: dir/<name>/pack.yaml
        let sub_yaml = dir.join(name).join("pack.yaml");
        if sub_yaml.is_file() {
            let manifest = load_manifest(&sub_yaml)?;
            if manifest.metadata.name == name {
                return Ok((sub_yaml, manifest));
            }
        }

        // Candidate 2: dir/<name>/pack.yml
        let sub_yml = dir.join(name).join("pack.yml");
        if sub_yml.is_file() {
            let manifest = load_manifest(&sub_yml)?;
            if manifest.metadata.name == name {
                return Ok((sub_yml, manifest));
            }
        }

        // Candidate 3: dir/<name>.yaml
        let file_yaml = dir.join(format!("{}.yaml", name));
        if file_yaml.is_file() {
            let manifest = load_manifest(&file_yaml)?;
            if manifest.metadata.name == name {
                return Ok((file_yaml, manifest));
            }
        }

        // Candidate 4: dir/<name>.yml
        let file_yml = dir.join(format!("{}.yml", name));
        if file_yml.is_file() {
            let manifest = load_manifest(&file_yml)?;
            if manifest.metadata.name == name {
                return Ok((file_yml, manifest));
            }
        }
    }

    // Fallback: Check compiled-in standard diagnostic packs
    if let Some(res) = get_builtin_pack(name) {
        let manifest = res?;
        return Ok((PathBuf::from(format!("<embedded:{}>", name)), manifest));
    }

    Err(JevOpsError::PackNotFound {
        name: name.to_string(),
        searched: dirs
            .iter()
            .map(|p| p.display().to_string())
            .collect::<Vec<_>>()
            .join(", "),
    })
}

/// Discovers all available packs across all search directories and compiled-in packs.
/// Higher precedence directories override lower precedence packs and compiled-in packs with the same name.
pub fn list_available_packs(custom_dir: Option<&Path>) -> Vec<DiscoveredPack> {
    let dirs = search_directories(custom_dir);
    let mut packs_by_name: BTreeMap<String, DiscoveredPack> = BTreeMap::new();

    // 0. Seed with compiled-in builtin packs (lowest precedence)
    for builtin in BUILTIN_PACKS {
        if let Ok(manifest) =
            load_manifest_from_str(builtin.yaml_content, &format!("<embedded:{}>", builtin.name))
        {
            packs_by_name.insert(
                manifest.metadata.name.clone(),
                DiscoveredPack {
                    name: manifest.metadata.name.clone(),
                    path: PathBuf::from(format!("<embedded:{}>", builtin.name)),
                    manifest,
                },
            );
        }
    }

    // Iterate in reverse order so higher-precedence directories overwrite lower ones
    for dir in dirs.iter().rev() {
        if !dir.is_dir() {
            continue;
        }

        if let Ok(entries) = std::fs::read_dir(dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                let manifest_path = if path.is_dir() {
                    let y1 = path.join("pack.yaml");
                    let y2 = path.join("pack.yml");
                    if y1.is_file() {
                        Some(y1)
                    } else if y2.is_file() {
                        Some(y2)
                    } else {
                        None
                    }
                } else if path.is_file() {
                    let ext = path.extension().and_then(|s| s.to_str()).unwrap_or("");
                    if ext == "yaml" || ext == "yml" {
                        Some(path.clone())
                    } else {
                        None
                    }
                } else {
                    None
                };

                if let Some(mp) = manifest_path {
                    match load_manifest(&mp) {
                        Ok(manifest) => {
                            packs_by_name.insert(
                                manifest.metadata.name.clone(),
                                DiscoveredPack {
                                    name: manifest.metadata.name.clone(),
                                    path: mp,
                                    manifest,
                                },
                            );
                        }
                        Err(e) => {
                            tracing::warn!("Skipping invalid pack '{}': {}", mp.display(), e);
                        }
                    }
                }
            }
        }
    }

    packs_by_name.into_values().collect()
}
