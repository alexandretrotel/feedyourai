use serde::{Deserialize, Serialize};
use std::fs;
use std::io;
use std::path::{Path, PathBuf};

use crate::cli::Config;

/// Struct for deserializing YAML config file.
#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct FileConfig {
    pub directory: Option<String>,
    pub output: Option<String>,
    pub include_dirs: Option<Vec<String>>,
    pub exclude_dirs: Option<Vec<String>>,
    pub include_ext: Option<Vec<String>>,
    pub exclude_ext: Option<Vec<String>>,
    pub include_files: Option<Vec<String>>,
    pub exclude_files: Option<Vec<String>>,
    pub min_size: Option<u64>,
    pub max_size: Option<u64>,
    pub respect_gitignore: Option<bool>,
    pub tree_only: Option<bool>,
}

impl FileConfig {
    /// Load config from a YAML file path.
    pub fn from_path<P: AsRef<Path>>(path: P) -> io::Result<Self> {
        let content = fs::read_to_string(path)?;
        let config: FileConfig = serde_yaml::from_str(&content).map_err(|e| {
            io::Error::new(
                io::ErrorKind::InvalidData,
                format!("YAML parse error: {}", e),
            )
        })?;
        Ok(config)
    }
}

/// Discover config file location based on precedence.
/// Returns Some(path) if found, None otherwise.
pub fn discover_config_file() -> Option<PathBuf> {
    let local = PathBuf::from("./fyai.yaml");
    if local.exists() {
        return Some(local);
    }
    if let Some(home) = dirs::home_dir() {
        let global = home.join(".fyai").join("config.yaml");
        if global.exists() {
            return Some(global);
        }
    }
    None
}

/// Merge FileConfig with CLI Config.
/// CLI config takes precedence over file config.
pub fn merge_config(file: FileConfig, cli: Config) -> Config {
    Config {
        directory: cli.directory,
        output: cli.output,
        include_dirs: cli.include_dirs.or(file.include_dirs),
        exclude_dirs: cli.exclude_dirs.or(file.exclude_dirs),
        include_ext: cli.include_ext.or(file.include_ext),
        exclude_ext: cli.exclude_ext.or(file.exclude_ext),
        include_files: cli.include_files.or(file.include_files),
        exclude_files: cli.exclude_files.or(file.exclude_files),
        min_size: cli.min_size.or(file.min_size),
        max_size: cli.max_size.or(file.max_size),
        respect_gitignore: cli.respect_gitignore,
        tree_only: cli.tree_only,
    }
}
