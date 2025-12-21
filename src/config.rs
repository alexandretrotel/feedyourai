use serde::{Deserialize, Serialize};
use std::fs;
use std::io;
use std::path::{Path, PathBuf};

/// Main config struct used throughout the app.
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
    if let Some(config_dir) = dirs::config_dir() {
        let global = config_dir.join("fyai.yaml");
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

/// Create Config from clap ArgMatches
pub fn config_from_matches(matches: clap::ArgMatches) -> std::io::Result<Config> {
    let directory = matches
        .try_get_one::<String>("directory")
        .map_err(|e| {
            std::io::Error::new(
                std::io::ErrorKind::InvalidInput,
                format!("Missing directory: {}", e),
            )
        })?
        .ok_or_else(|| std::io::Error::new(std::io::ErrorKind::InvalidInput, "Missing directory"))?
        .into();

    let output = matches
        .try_get_one::<String>("output")
        .map_err(|e| {
            std::io::Error::new(
                std::io::ErrorKind::InvalidInput,
                format!("Missing output: {}", e),
            )
        })?
        .ok_or_else(|| std::io::Error::new(std::io::ErrorKind::InvalidInput, "Missing output"))?
        .into();

    let include_dirs = match matches.try_get_one::<String>("include_dirs") {
        Ok(opt) => opt.map(|dirs| {
            dirs.split(',')
                .map(|s| s.trim().to_lowercase())
                .filter(|s| !s.is_empty())
                .collect::<Vec<_>>()
        }),
        Err(_) => None,
    };

    let exclude_dirs = match matches.try_get_one::<String>("exclude_dirs") {
        Ok(opt) => opt.map(|dirs| {
            dirs.split(',')
                .map(|s| s.trim().to_lowercase())
                .filter(|s| !s.is_empty())
                .collect::<Vec<_>>()
        }),
        Err(_) => None,
    };

    let include_ext = match matches.try_get_one::<String>("include_ext") {
        Ok(opt) => opt.map(|ext| {
            ext.split(',')
                .map(|s| s.trim().to_lowercase())
                .filter(|s| !s.is_empty())
                .collect::<Vec<_>>()
        }),
        Err(_) => None,
    };

    let exclude_ext = match matches.try_get_one::<String>("exclude_ext") {
        Ok(opt) => opt.map(|ext| {
            ext.split(',')
                .map(|s| s.trim().to_lowercase())
                .filter(|s| !s.is_empty())
                .collect::<Vec<_>>()
        }),
        Err(_) => None,
    };

    let include_files = match matches.try_get_one::<String>("include_files") {
        Ok(opt) => opt.map(|files| {
            files
                .split(',')
                .map(|s| s.trim().to_lowercase())
                .filter(|s| !s.is_empty())
                .collect::<Vec<_>>()
        }),
        Err(_) => None,
    };

    let exclude_files = match matches.try_get_one::<String>("exclude_files") {
        Ok(opt) => opt.map(|files| {
            files
                .split(',')
                .map(|s| s.trim().to_lowercase())
                .filter(|s| !s.is_empty())
                .collect::<Vec<_>>()
        }),
        Err(_) => None,
    };

    let min_size = match matches.try_get_one::<String>("min_size") {
        Ok(Some(s)) => Some(s.parse::<u64>().map_err(|_| {
            std::io::Error::new(std::io::ErrorKind::InvalidInput, "Invalid min-size")
        })?),
        Ok(None) => None,
        Err(_) => None,
    };

    let max_size = match matches.try_get_one::<String>("max_size") {
        Ok(Some(s)) => Some(s.parse::<u64>().map_err(|_| {
            std::io::Error::new(std::io::ErrorKind::InvalidInput, "Invalid max-size")
        })?),
        Ok(None) => None,
        Err(_) => None,
    };

    let respect_gitignore = match matches.try_get_one::<String>("respect_gitignore") {
        Ok(Some(s)) => s == "true" || s == "1",
        Ok(None) => true,
        Err(_) => true,
    };

    // For flags, keep using contains_id + get_flag which is safe when checked first
    let tree_only = if matches.contains_id("tree_only") {
        matches.get_flag("tree_only")
    } else {
        false
    };

    Ok(Config {
        directory,
        output,
        include_dirs,
        exclude_dirs,
        include_ext,
        exclude_ext,
        include_files,
        exclude_files,
        min_size,
        max_size,
        respect_gitignore,
        tree_only,
    })
}
