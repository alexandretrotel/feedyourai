use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};

use crate::errors::{AppError, AppResult};
use clap::parser::ValueSource;
use directories_next::BaseDirs;

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
    pub fn from_path<P: AsRef<Path>>(path: P) -> AppResult<Self> {
        let content = fs::read_to_string(path.as_ref())?;
        let config: FileConfig =
            serde_yaml::from_str(&content).map_err(|e| AppError::YamlParse {
                path: path.as_ref().to_path_buf(),
                source: e,
            })?;
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
    BaseDirs::new().map(|dirs| dirs.config_dir().to_path_buf())
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

pub fn config_from_matches(matches: clap::ArgMatches) -> AppResult<(Config, ExplicitFlags)> {
    let directory_set = matches.value_source("directory") == Some(ValueSource::CommandLine);
    let output_set = matches.value_source("output") == Some(ValueSource::CommandLine);
    let respect_gitignore_set =
        matches.value_source("respect_gitignore") == Some(ValueSource::CommandLine);
    let tree_only_set = matches.value_source("tree_only") == Some(ValueSource::CommandLine);

    let directory = matches
        .try_get_one::<String>("directory")
        .map_err(|_| AppError::MissingDirectory)?
        .ok_or(AppError::MissingDirectory)?
        .into();

    let output = matches
        .try_get_one::<String>("output")
        .map_err(|_| AppError::MissingOutput)?
        .ok_or(AppError::MissingOutput)?
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

    let min_size = match matches.try_get_one::<u64>("min_size") {
        Ok(Some(value)) => Some(*value),
        Ok(None) | Err(_) => match matches.try_get_one::<String>("min_size") {
            Ok(Some(s)) => Some(s.parse::<u64>().map_err(|_| AppError::InvalidMinSize)?),
            Ok(None) | Err(_) => None,
        },
    };

    let max_size = match matches.try_get_one::<u64>("max_size") {
        Ok(Some(value)) => Some(*value),
        Ok(None) | Err(_) => match matches.try_get_one::<String>("max_size") {
            Ok(Some(s)) => Some(s.parse::<u64>().map_err(|_| AppError::InvalidMaxSize)?),
            Ok(None) | Err(_) => None,
        },
    };

    let respect_gitignore = match matches.try_get_one::<bool>("respect_gitignore") {
        Ok(Some(flag)) => *flag,
        Ok(None) | Err(_) => match matches.try_get_one::<String>("respect_gitignore") {
            Ok(Some(s)) => s == "true" || s == "1",
            Ok(None) | Err(_) => true,
        },
    };

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
