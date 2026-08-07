//! Cloning a remote git repository into a temporary directory before
//! scanning it.

use std::{path::PathBuf, process, process::Command};

use tempfile::TempDir;

use crate::config::Config;
use crate::error::{FyaiError, Result};

/// Extracts a human-readable error message from a failed command's output,
/// preferring stderr over stdout, and falling back to a generic message if
/// both are empty.
fn command_error_details(output: &process::Output) -> String {
    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);

    let details = if !stderr.trim().is_empty() {
        stderr.trim()
    } else {
        stdout.trim()
    };

    if details.is_empty() {
        "unknown error".to_string()
    } else {
        details.to_string()
    }
}

/// Clones `repo_url` into a fresh temporary directory, optionally checking
/// out `branch` and/or `commit`.
///
/// The clone is shallow (`--depth 1`) unless `commit` is set, since the
/// target commit may not be reachable from a depth-1 history.
///
/// Returns the [`TempDir`] guard (drop it to delete the clone) alongside the
/// path to the checked-out repository.
pub(crate) fn clone_repository(
    repo_url: &str,
    branch: Option<&str>,
    commit: Option<&str>,
) -> Result<(TempDir, PathBuf)> {
    let temp_dir = tempfile::tempdir()?;
    let clone_path = temp_dir.path().join("repo");

    let mut cmd = Command::new("git");
    cmd.arg("clone");
    if commit.is_none() {
        cmd.arg("--depth").arg("1");
    }
    if let Some(branch) = branch {
        cmd.args(["--branch", branch]);
    }
    cmd.arg(repo_url).arg(&clone_path);

    let output = cmd
        .output()
        .map_err(|e| FyaiError::Git(format!("failed to run git clone: {e}")))?;
    if !output.status.success() {
        return Err(FyaiError::Git(format!(
            "git clone failed: {}",
            command_error_details(&output)
        )));
    }

    if let Some(commit) = commit {
        let output = Command::new("git")
            .arg("-C")
            .arg(&clone_path)
            .args(["checkout", commit])
            .output()
            .map_err(|e| FyaiError::Git(format!("failed to run git checkout: {e}")))?;
        if !output.status.success() {
            return Err(FyaiError::Git(format!(
                "git checkout failed: {}",
                command_error_details(&output)
            )));
        }
    }

    Ok((temp_dir, clone_path))
}

/// Clones `repo_url` and runs [`crate::commands::run`] against the clone.
///
/// The temporary clone is deleted before this function returns, whether the
/// combine run succeeded or failed.
pub fn run(
    repo_url: &str,
    branch: Option<&str>,
    commit: Option<&str>,
    config: Config,
) -> Result<()> {
    let (temp_dir, clone_path) = clone_repository(repo_url, branch, commit)?;

    let mut config = config;
    config.directory = clone_path;

    let result = crate::commands::run(config);
    drop(temp_dir);
    result
}
