//! Orchestrates a single combine run: builds the directory tree, then either
//! writes the tree only or writes the tree plus every matching file's
//! contents. Also handles cloning a remote git repository into a temporary
//! directory before running the same combine logic against it.

use std::fs;
use std::path::PathBuf;
use std::process::{self, Command};

use tempfile::TempDir;

use crate::config::Config;
use crate::error::{FyaiError, Result};
use crate::scanner::{get_directory_tree, process_files};

/// Combines files from a local directory as described by `config`.
///
/// Writes the result to `config.output` (either the directory tree only, or
/// the tree plus file contents, depending on `config.tree_only`).
pub fn run_local(config: Config) -> Result<()> {
    let dir_structure = get_directory_tree(&config)?;

    if config.tree_only {
        fs::write(&config.output, &dir_structure)?;
    } else {
        process_files(&config, &dir_structure)?;
    }

    Ok(())
}

/// Clones `repo_url` into a temporary directory, then runs the same combine
/// logic as [`run_local`] against the clone.
///
/// The temporary clone is removed before this function returns, regardless
/// of the run's outcome.
///
/// # Arguments
///
/// * `branch` - an optional branch or tag to check out.
/// * `commit` - an optional commit SHA to check out after cloning. When set,
///   the clone is not shallow, since the target commit may not be reachable
///   from a depth-1 clone.
/// * `config` - the combine configuration; `config.directory` is overwritten
///   with the path to the cloned repository.
pub fn run_git(
    repo_url: &str,
    branch: Option<&str>,
    commit: Option<&str>,
    config: Config,
) -> Result<()> {
    let (temp_dir, clone_path) = clone_repository(repo_url, branch, commit)?;

    let mut config = config;
    config.directory = clone_path;

    let result = run_local(config);
    drop(temp_dir);
    result
}

/// Clones `repo_url` into a fresh temporary directory, optionally checking
/// out `branch` and/or `commit`.
///
/// The clone is shallow (`--depth 1`) unless `commit` is set, since the
/// target commit may not be reachable from a depth-1 history.
///
/// Returns the [`TempDir`] guard (drop it to delete the clone) alongside the
/// path to the checked-out repository.
fn clone_repository(
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
