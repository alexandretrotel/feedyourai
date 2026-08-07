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

    /// The config file at `path` was read but is not valid TOML for
    /// [`crate::config::PartialConfig`].
    #[error("TOML parse error in {path}: {source}")]
    ParseConfig {
        /// Path of the config file that failed to parse.
        path: PathBuf,
        /// Underlying TOML parse error.
        source: toml::de::Error,
    },

    /// Spawning `git`, or the `git` command itself, failed.
    #[error("{0}")]
    Git(String),
}

/// A [`Result`](std::result::Result) whose error type is [`FyaiError`].
pub type Result<T> = std::result::Result<T, FyaiError>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn io_variant_from_conversion() {
        let io_err = io::Error::other("boom");
        let fyai_err: FyaiError = io_err.into();
        assert!(matches!(fyai_err, FyaiError::Io(_)));
    }

    #[test]
    fn io_variant_display_is_transparent() {
        let io_err = io::Error::other("disk on fire");
        let fyai_err: FyaiError = FyaiError::Io(io_err);
        assert_eq!(fyai_err.to_string(), "disk on fire");
    }

    #[test]
    fn io_variant_via_question_mark_operator() {
        fn fails() -> Result<()> {
            Err(io::Error::other("nope"))?;
            Ok(())
        }

        let err = fails().unwrap_err();
        assert!(matches!(err, FyaiError::Io(_)));
        assert_eq!(err.to_string(), "nope");
    }

    #[test]
    fn read_config_display_includes_path_and_source() {
        let path = PathBuf::from("/tmp/does-not-exist/fyai.toml");
        let source = io::Error::other("permission denied");
        let err = FyaiError::ReadConfig {
            path: path.clone(),
            source,
        };
        let msg = err.to_string();
        assert!(msg.contains("failed to read config file"));
        assert!(msg.contains(&path.to_string_lossy().to_string()));
        assert!(msg.contains("permission denied"));
    }

    #[test]
    fn parse_config_display_includes_path_and_source() {
        #[derive(Debug, serde::Deserialize)]
        struct DummyConfig {
            #[allow(dead_code)]
            field: String,
        }

        let toml_err = toml::from_str::<DummyConfig>("not valid toml =")
            .expect_err("expected a TOML parse error");
        let toml_err_string = toml_err.to_string();

        let path = PathBuf::from("/etc/fyai/fyai.toml");
        let err = FyaiError::ParseConfig {
            path: path.clone(),
            source: toml_err,
        };
        let msg = err.to_string();
        assert!(msg.contains("TOML parse error in"));
        assert!(msg.contains(&path.to_string_lossy().to_string()));
        assert!(msg.contains(&toml_err_string));
    }

    #[test]
    fn git_variant_display_prints_inner_string() {
        let err = FyaiError::Git("git executable not found".to_string());
        assert_eq!(err.to_string(), "git executable not found");
    }

    #[test]
    fn debug_impl_is_available() {
        let err = FyaiError::Git("boom".to_string());
        let debug_str = format!("{err:?}");
        assert!(debug_str.contains("Git"));
    }
}
