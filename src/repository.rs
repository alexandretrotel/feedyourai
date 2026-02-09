use std::{path::PathBuf, process::Command};

use tempfile::TempDir;

use crate::config::Config;
use crate::error::{AppError, AppResult};

pub(crate) fn clone_repository(
    repo_url: &str,
    branch: Option<&str>,
) -> AppResult<(TempDir, PathBuf)> {
    let temp_dir = tempfile::tempdir()?;
    let clone_path = temp_dir.path().join("repo");

    let mut cmd = Command::new("git");
    cmd.arg("clone").arg("--depth").arg("1");
    if let Some(branch) = branch {
        cmd.args(["--branch", branch]);
    }
    cmd.arg(repo_url).arg(&clone_path);

    let output = cmd.output().map_err(AppError::GitCloneExec)?;
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        let stdout = String::from_utf8_lossy(&output.stdout);
        let details = if !stderr.trim().is_empty() {
            stderr.trim()
        } else {
            stdout.trim()
        };
        let details = if details.is_empty() {
            "unknown error"
        } else {
            details
        };
        return Err(AppError::GitCloneFailed(details.to_string()));
    }

    Ok((temp_dir, clone_path))
}

pub fn run_on_repository(repo_url: &str, branch: Option<&str>, config: Config) -> AppResult<()> {
    let (temp_dir, clone_path) = clone_repository(repo_url, branch)?;

    let mut config = config;
    config.directory = clone_path;

    let result = crate::run_with_config(config);

    drop(temp_dir);

    result
}
