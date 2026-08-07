//! Configuration types (CLI-agnostic) and config-file discovery/merging.
//!
//! [`Config`](crate::config::Config) is what the scanning/combining logic
//! actually runs on.
//! [`PartialConfig`](crate::config::PartialConfig) is its optional,
//! partially-specified counterpart: one instance loaded from a `fyai.toml`
//! file, another built from CLI flags (`None` for anything not explicitly
//! passed, so an unset CLI flag can't shadow a config-file value).
//! [`merge_config`](crate::config::merge_config) reconciles the two, CLI
//! winning over file, file winning over the built-in default.

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
    /// Directory names to skip.
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

/// Partially-specified configuration, either loaded from a `fyai.toml` file
/// or built from CLI flags. Every field is optional; unset fields fall back
/// to the other source, then to a built-in default, when merged via
/// [`merge_config`].
#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct PartialConfig {
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

impl PartialConfig {
    /// Reads and parses a `fyai.toml`-style config file from `path`.
    ///
    /// # Errors
    ///
    /// Returns [`FyaiError::ReadConfig`] if the file can't be read, or
    /// [`FyaiError::ParseConfig`] if its contents aren't valid TOML for this
    /// type.
    pub fn from_path<P: AsRef<Path>>(path: P) -> Result<Self> {
        let path = path.as_ref();
        let content = fs::read_to_string(path).map_err(|source| FyaiError::ReadConfig {
            path: path.to_path_buf(),
            source,
        })?;
        let config: PartialConfig =
            toml::from_str(&content).map_err(|source| FyaiError::ParseConfig {
                path: path.to_path_buf(),
                source,
            })?;
        Ok(config)
    }
}

/// Looks for a config file, preferring a local `./fyai.toml` over the
/// system-wide one returned by [`system_config_dir`].
///
/// Returns `None` if neither exists.
pub fn discover_config_file() -> Option<PathBuf> {
    let local = PathBuf::from("./fyai.toml");
    if local.exists() {
        return Some(local);
    }
    if let Some(config_dir) = system_config_dir() {
        let global = config_dir.join("fyai.toml");
        if global.exists() {
            return Some(global);
        }
    }
    None
}

/// Returns the platform's config directory, where the global `fyai.toml`
/// lives: `$XDG_CONFIG_HOME` if set to an absolute path (honored on every
/// platform, not just Linux, matching the XDG Base Directory spec), else
/// the platform default (e.g. `~/.config` on Linux, `~/Library/Application
/// Support` on macOS).
pub fn system_config_dir() -> Option<PathBuf> {
    if let Some(xdg_config_home) = std::env::var_os("XDG_CONFIG_HOME") {
        let path = PathBuf::from(xdg_config_home);
        if path.is_absolute() {
            return Some(path);
        }
    }
    dirs::config_dir()
}

/// Merges two [`PartialConfig`]s into a final [`Config`]: `cli`'s value wins
/// wherever set, otherwise `file`'s, otherwise the built-in default.
pub fn merge_config(file: PartialConfig, cli: PartialConfig) -> Config {
    let directory = cli
        .directory
        .or(file.directory)
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("."));

    let output = cli
        .output
        .or(file.output)
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("fyai.txt"));

    let respect_gitignore = cli
        .respect_gitignore
        .or(file.respect_gitignore)
        .unwrap_or(true);
    let tree_only = cli.tree_only.or(file.tree_only).unwrap_or(false);
    let human = cli.human.or(file.human).unwrap_or(false);

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
