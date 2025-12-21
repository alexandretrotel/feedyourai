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
///
/// The merge accepts an `ExplicitFlags` argument which indicates which CLI
/// values were explicitly set by the user.
#[derive(Debug, Clone, Copy)]
pub struct ExplicitFlags {
    pub directory: bool,
    pub output: bool,
    pub respect_gitignore: bool,
    pub tree_only: bool,
}

pub fn merge_config_with_explicit(
    file: FileConfig,
    cli: Config,
    explicit: ExplicitFlags,
) -> Config {
    // For directory and output, prefer file value when the CLI did not explicitly set them.
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

    // For booleans, use file value when CLI did not explicitly set the flag.
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

/// Create Config from clap ArgMatches
///
/// Returns both the built `Config` and an `ExplicitFlags` struct that indicates
/// which CLI values were actually provided on the command line (as opposed to
/// being left as clap defaults).
pub fn config_from_matches_with_explicit(
    matches: clap::ArgMatches,
) -> std::io::Result<(Config, ExplicitFlags)> {
    let directory_set = match matches.try_get_one::<String>("directory") {
        Ok(Some(_)) => true,
        Ok(None) => false,
        Err(_) => false,
    };
    let output_set = match matches.try_get_one::<String>("output") {
        Ok(Some(_)) => true,
        Ok(None) => false,
        Err(_) => false,
    };
    let respect_gitignore_set = match matches.try_get_one::<String>("respect_gitignore") {
        Ok(Some(_)) => true,
        Ok(None) => false,
        Err(_) => false,
    };
    let tree_only_set = match matches.try_get_one::<bool>("tree_only") {
        Ok(Some(_)) => true,
        Ok(None) => false,
        Err(_) => false,
    };

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
                io::ErrorKind::InvalidInput,
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

    // For flags, use try_get_one to safely handle whether the arg is registered
    let tree_only = match matches.try_get_one::<bool>("tree_only") {
        Ok(Some(b)) => *b,
        Ok(None) => false,
        Err(_) => false,
    };

    Ok((
        Config {
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
        },
        ExplicitFlags {
            directory: directory_set,
            output: output_set,
            respect_gitignore: respect_gitignore_set,
            tree_only: tree_only_set,
        },
    ))
}
