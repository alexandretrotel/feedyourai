use std::io;
use std::path::PathBuf;

use thiserror::Error;

pub type AppResult<T> = Result<T, AppError>;

#[derive(Debug, Error)]
pub enum AppError {
    #[error(transparent)]
    Io(#[from] io::Error),

    #[error("Clipboard error: {0}")]
    Clipboard(String),

    #[error("YAML parse error in {path}: {source}")]
    YamlParse {
        path: PathBuf,
        #[source]
        source: serde_yaml::Error,
    },

    #[error("Missing directory")]
    MissingDirectory,

    #[error("Missing output")]
    MissingOutput,

    #[error("Invalid min-size")]
    InvalidMinSize,

    #[error("Invalid max-size")]
    InvalidMaxSize,

    #[error("Config file already exists at {path}. Use --force to overwrite.")]
    ConfigAlreadyExists { path: String },

    #[error("Failed to run git clone: {0}")]
    GitCloneExec(#[source] io::Error),

    #[error("git clone failed: {0}")]
    GitCloneFailed(String),

    #[error("Failed to run git checkout: {0}")]
    GitCheckoutExec(#[source] io::Error),

    #[error("git checkout failed: {0}")]
    GitCheckoutFailed(String),
}
