use anyhow::{Context, Result};
use glob::glob;
use serde::Deserialize;
use std::collections::HashSet;
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Deserialize)]
pub struct Config {
    pub projects: Vec<Project>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Project {
    pub input_paths: Vec<String>,
}

impl Config {
    pub fn load(config_path: &Path) -> Result<Self> {
        let content = fs::read_to_string(config_path)
            .with_context(|| format!("Failed to read config file at: {}", config_path.display()))?;

        let config: Config = serde_json::from_str(&content)
            .with_context(|| "Failed to parse grimoire.config.json")?;

        Ok(config)
    }

    pub fn get_files_to_watch(&self, config_path: &Path) -> Vec<PathBuf> {
        let mut files = HashSet::new();

        for project in &self.projects {
            for pattern in &project.input_paths {
                match glob(pattern) {
                    Ok(paths) => {
                        for path_result in paths {
                            match path_result {
                                Ok(path) if path.is_file() => {
                                    files.insert(path);
                                }
                                Ok(_) => continue, // Skip non-file paths
                                Err(e) => {
                                    log::error!(
                                        "Failed to process path for pattern {}: {}",
                                        pattern,
                                        e
                                    );
                                }
                            }
                        }
                    }
                    Err(e) => {
                        log::error!("Invalid glob pattern {}: {}", pattern, e);
                    }
                }
            }
        }

        files.insert(config_path.to_path_buf());

        // Convert HashSet to Vec for return
        files.into_iter().collect()
    }
}
