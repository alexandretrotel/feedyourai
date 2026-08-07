use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};

use color_eyre::eyre::{Result, WrapErr};

#[derive(Debug, PartialEq, Clone)]
pub struct Config {
    pub directory: PathBuf,
    pub output: PathBuf,
    pub include_dirs: Option<Vec<String>>,
    pub exclude_dirs: Option<Vec<String>>,
    pub include_ext: Option<Vec<String>>,
    pub exclude_ext: Option<Vec<String>>,
    pub include_files: Option<Vec<String>>,
    pub exclude_files: Option<Vec<String>>,
    pub min_size: Option<u64>,
    pub max_size: Option<u64>,
    pub respect_gitignore: bool,
    pub tree_only: bool,
}

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
    pub fn from_path<P: AsRef<Path>>(path: P) -> Result<Self> {
        let path = path.as_ref();
        let content = fs::read_to_string(path)
            .wrap_err_with(|| format!("failed to read config file {}", path.display()))?;
        let config: FileConfig = yaml_serde::from_str(&content)
            .wrap_err_with(|| format!("YAML parse error in {}", path.display()))?;
        Ok(config)
    }
}

pub fn discover_config_file() -> Option<PathBuf> {
    let local = PathBuf::from("./fyai.yaml");
    if local.exists() {
        return Some(local);
    }
    if let Some(config_dir) = system_config_dir() {
        let global = config_dir.join("fyai.yaml");
        if global.exists() {
            return Some(global);
        }
    }
    None
}

pub fn system_config_dir() -> Option<PathBuf> {
    dirs::config_dir()
}

#[derive(Debug, Clone, Copy)]
pub struct ExplicitFlags {
    pub directory: bool,
    pub output: bool,
    pub respect_gitignore: bool,
    pub tree_only: bool,
}

pub fn merge_config(file: FileConfig, cli: Config, explicit: ExplicitFlags) -> Config {
    let directory = if explicit.directory {
        cli.directory
    } else {
        file.directory.map(PathBuf::from).unwrap_or(cli.directory)
    };

    let output = if explicit.output {
        cli.output
    } else {
        file.output.map(PathBuf::from).unwrap_or(cli.output)
    };

    let respect_gitignore = if explicit.respect_gitignore {
        cli.respect_gitignore
    } else {
        file.respect_gitignore.unwrap_or(cli.respect_gitignore)
    };

    let tree_only = if explicit.tree_only {
        cli.tree_only
    } else {
        file.tree_only.unwrap_or(cli.tree_only)
    };

    Config {
        directory,
        output,
        include_dirs: cli.include_dirs.or(file.include_dirs),
        exclude_dirs: cli.exclude_dirs.or(file.exclude_dirs),
        include_ext: cli.include_ext.or(file.include_ext),
        exclude_ext: cli.exclude_ext.or(file.exclude_ext),
        include_files: cli.include_files.or(file.include_files),
        exclude_files: cli.exclude_files.or(file.exclude_files),
        min_size: cli.min_size.or(file.min_size),
        max_size: cli.max_size.or(file.max_size),
        respect_gitignore,
        tree_only,
    }
}
