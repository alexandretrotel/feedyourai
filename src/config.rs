//! Configuration types (CLI-agnostic) and config-file discovery/merging.
//!
//! [`Config`] is what the scanning/combining logic actually runs on.
//! [`FileConfig`](crate::config::FileConfig) is its optional,
//! partially-specified counterpart loaded from a `fyai.yaml` file;
//! [`merge_config`](crate::config::merge_config) reconciles a `FileConfig`
//! with CLI-supplied values.

use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};

use crate::error::{FyaiError, Result};

/// Fully-resolved configuration for a single combine run.
#[derive(Debug, PartialEq, Clone)]
pub struct Config {
    /// Directory to scan.
    pub directory: PathBuf,
    /// File the combined output is written to.
    pub output: PathBuf,
    /// If set, only directories whose name matches one of these are walked.
    pub include_dirs: Option<Vec<String>>,
    /// Directory names to skip, in addition to [`crate::constants::IGNORED_DIRS`].
    pub exclude_dirs: Option<Vec<String>>,
    /// If set, only files with one of these extensions are included.
    pub include_ext: Option<Vec<String>>,
    /// File extensions to skip.
    pub exclude_ext: Option<Vec<String>>,
    /// If set, only files with one of these names are included.
    pub include_files: Option<Vec<String>>,
    /// File names to skip.
    pub exclude_files: Option<Vec<String>>,
    /// Files smaller than this many bytes are skipped.
    pub min_size: Option<u64>,
    /// Files larger than this many bytes are skipped.
    pub max_size: Option<u64>,
    /// Whether to honor `.gitignore` and friends while walking.
    pub respect_gitignore: bool,
    /// If true, only the directory tree is written; file contents are skipped.
    pub tree_only: bool,
    /// If true, renders the directory tree with `tree`-style connector
    /// glyphs (`├──`, `└──`, `│`) instead of the minimal two-space indent.
    pub human: bool,
}

/// Partially-specified configuration as loaded from a `fyai.yaml` file.
///
/// Every field is optional; unset fields fall back to the CLI-supplied
/// [`Config`] value when merged via [`merge_config`].
#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct FileConfig {
    /// See [`Config::directory`].
    pub directory: Option<String>,
    /// See [`Config::output`].
    pub output: Option<String>,
    /// See [`Config::include_dirs`].
    pub include_dirs: Option<Vec<String>>,
    /// See [`Config::exclude_dirs`].
    pub exclude_dirs: Option<Vec<String>>,
    /// See [`Config::include_ext`].
    pub include_ext: Option<Vec<String>>,
    /// See [`Config::exclude_ext`].
    pub exclude_ext: Option<Vec<String>>,
    /// See [`Config::include_files`].
    pub include_files: Option<Vec<String>>,
    /// See [`Config::exclude_files`].
    pub exclude_files: Option<Vec<String>>,
    /// See [`Config::min_size`].
    pub min_size: Option<u64>,
    /// See [`Config::max_size`].
    pub max_size: Option<u64>,
    /// See [`Config::respect_gitignore`].
    pub respect_gitignore: Option<bool>,
    /// See [`Config::tree_only`].
    pub tree_only: Option<bool>,
    /// See [`Config::human`].
    pub human: Option<bool>,
}

impl FileConfig {
    /// Reads and parses a `fyai.yaml`-style config file from `path`.
    ///
    /// # Errors
    ///
    /// Returns [`FyaiError::ReadConfig`] if the file can't be read, or
    /// [`FyaiError::ParseConfig`] if its contents aren't valid YAML for this
    /// type.
    pub fn from_path<P: AsRef<Path>>(path: P) -> Result<Self> {
        let path = path.as_ref();
        let content = fs::read_to_string(path).map_err(|source| FyaiError::ReadConfig {
            path: path.to_path_buf(),
            source,
        })?;
        let config: FileConfig =
            yaml_serde::from_str(&content).map_err(|source| FyaiError::ParseConfig {
                path: path.to_path_buf(),
                source,
            })?;
        Ok(config)
    }
}

/// Looks for a config file, preferring a local `./fyai.yaml` over the
/// system-wide one returned by [`system_config_dir`].
///
/// Returns `None` if neither exists.
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

/// Returns the platform's config directory (e.g. `~/.config` on Linux),
/// where the global `fyai.yaml` lives.
pub fn system_config_dir() -> Option<PathBuf> {
    dirs::config_dir()
}

/// Tracks which [`Config`] fields were explicitly set on the command line,
/// so [`merge_config`] knows whether a CLI default should lose to a config
/// file value.
#[derive(Debug, Clone, Copy)]
pub struct ExplicitFlags {
    /// Whether `--input`/`-i` was passed explicitly.
    pub directory: bool,
    /// Whether `--output`/`-o` was passed explicitly.
    pub output: bool,
    /// Whether `--no-gitignore` was passed explicitly.
    pub respect_gitignore: bool,
    /// Whether `--tree-only` was passed explicitly.
    pub tree_only: bool,
    /// Whether `--human` was passed explicitly.
    pub human: bool,
}

/// Merges a [`FileConfig`] with CLI-supplied values into a final [`Config`].
///
/// For fields tracked by `explicit`, an explicit CLI value always wins.
/// Otherwise, the config file's value is preferred, falling back to the CLI
/// default. Fields without an `explicit` flag (the `include_*`/`exclude_*`
/// and size filters) simply prefer the CLI value when present.
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

    let human = if explicit.human {
        cli.human
    } else {
        file.human.unwrap_or(cli.human)
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
        human,
    }
}
