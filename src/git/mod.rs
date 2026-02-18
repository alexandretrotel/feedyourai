use std::{path::PathBuf, process::Command};

use tempfile::TempDir;

use crate::config::Config;
use crate::errors::{AppError, AppResult};
use crate::git::utils::command_error_details;

mod utils;

pub(crate) fn clone_repository(
    repo_url: &str,
    branch: Option<&str>,
    commit: Option<&str>,
) -> AppResult<(TempDir, PathBuf)> {
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

    let output = cmd.output().map_err(AppError::GitCloneExec)?;
    if !output.status.success() {
        return Err(AppError::GitCloneFailed(command_error_details(&output)));
    }

    if let Some(commit) = commit {
        let output = Command::new("git")
            .arg("-C")
            .arg(&clone_path)
            .args(["checkout", commit])
            .output()
            .map_err(AppError::GitCheckoutExec)?;
        if !output.status.success() {
            return Err(AppError::GitCheckoutFailed(command_error_details(&output)));
        }
    }

    Ok((temp_dir, clone_path))
}

pub fn run(
    repo_url: &str,
    branch: Option<&str>,
    commit: Option<&str>,
    config: Config,
) -> AppResult<()> {
    let (temp_dir, clone_path) = clone_repository(repo_url, branch, commit)?;

    let mut config = config;
    config.directory = clone_path;

    let result = crate::run_with_config(config);
    drop(temp_dir);
    result
}
