use std::{path::PathBuf, process::Command};

use eyre::{Result, WrapErr, eyre};
use tempfile::TempDir;

use crate::config::Config;
use crate::git::utils::command_error_details;

mod utils;

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

    let output = cmd.output().wrap_err("failed to run git clone")?;
    if !output.status.success() {
        return Err(eyre!("git clone failed: {}", command_error_details(&output)));
    }

    if let Some(commit) = commit {
        let output = Command::new("git")
            .arg("-C")
            .arg(&clone_path)
            .args(["checkout", commit])
            .output()
            .wrap_err("failed to run git checkout")?;
        if !output.status.success() {
            return Err(eyre!(
                "git checkout failed: {}",
                command_error_details(&output)
            ));
        }
    }

    Ok((temp_dir, clone_path))
}

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
