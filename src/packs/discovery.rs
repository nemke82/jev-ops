use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use crate::error::{JevOpsError, Result};
use crate::packs::loader::load_manifest;
use crate::packs::manifest::PackManifest;

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
pub fn find_pack(name: &str, custom_dir: Option<&Path>) -> Result<(PathBuf, PackManifest)> {
    let dirs = search_directories(custom_dir);
    let mut searched_locations = Vec::new();

    for dir in &dirs {
        if !dir.is_dir() {
            continue;
        }

        // Candidate 1: dir/<name>/pack.yaml
        let sub_yaml = dir.join(name).join("pack.yaml");
        searched_locations.push(sub_yaml.display().to_string());
        if sub_yaml.is_file() {
            let manifest = load_manifest(&sub_yaml)?;
            if manifest.metadata.name == name {
                return Ok((sub_yaml, manifest));
            }
        }

        // Candidate 2: dir/<name>/pack.yml
        let sub_yml = dir.join(name).join("pack.yml");
        searched_locations.push(sub_yml.display().to_string());
        if sub_yml.is_file() {
            let manifest = load_manifest(&sub_yml)?;
            if manifest.metadata.name == name {
                return Ok((sub_yml, manifest));
            }
        }

        // Candidate 3: dir/<name>.yaml
        let file_yaml = dir.join(format!("{}.yaml", name));
        searched_locations.push(file_yaml.display().to_string());
        if file_yaml.is_file() {
            let manifest = load_manifest(&file_yaml)?;
            if manifest.metadata.name == name {
                return Ok((file_yaml, manifest));
            }
        }

        // Candidate 4: dir/<name>.yml
        let file_yml = dir.join(format!("{}.yml", name));
        searched_locations.push(file_yml.display().to_string());
        if file_yml.is_file() {
            let manifest = load_manifest(&file_yml)?;
            if manifest.metadata.name == name {
                return Ok((file_yml, manifest));
            }
        }
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

/// Discovers all available packs across all search directories.
/// Higher precedence directories override lower precedence packs with the same name.
pub fn list_available_packs(custom_dir: Option<&Path>) -> Vec<DiscoveredPack> {
    let dirs = search_directories(custom_dir);
    let mut packs_by_name: BTreeMap<String, DiscoveredPack> = BTreeMap::new();

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
                    if let Ok(manifest) = load_manifest(&mp) {
                        packs_by_name.insert(
                            manifest.metadata.name.clone(),
                            DiscoveredPack {
                                name: manifest.metadata.name.clone(),
                                path: mp,
                                manifest,
                            },
                        );
                    }
                }
            }
        }
    }

    packs_by_name.into_values().collect()
}
