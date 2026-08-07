//! The crate's error type.

use std::io;
use std::path::PathBuf;

use thiserror::Error;

/// Errors that can occur while combining files or resolving configuration.
#[derive(Debug, Error)]
pub enum FyaiError {
    /// A filesystem operation (read, write, walk, ...) failed.
    #[error(transparent)]
    Io(#[from] io::Error),

    /// The config file at `path` could not be read.
    #[error("failed to read config file {path}: {source}")]
    ReadConfig {
        /// Path of the config file that could not be read.
        path: PathBuf,
        /// Underlying I/O error.
        source: io::Error,
    },

    /// The config file at `path` was read but is not valid YAML for
    /// [`crate::config::FileConfig`].
    #[error("YAML parse error in {path}: {source}")]
    ParseConfig {
        /// Path of the config file that failed to parse.
        path: PathBuf,
        /// Underlying YAML parse error.
        source: yaml_serde::Error,
    },

    /// Spawning `git`, or the `git` command itself, failed.
    #[error("{0}")]
    Git(String),
}

/// A [`Result`](std::result::Result) whose error type is [`FyaiError`].
pub type Result<T> = std::result::Result<T, FyaiError>;
