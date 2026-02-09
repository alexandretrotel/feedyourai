use std::{io, path::PathBuf, process::Command};

use tempfile::TempDir;

use crate::config::Config;

pub(crate) fn clone_repository(
    repo_url: &str,
    branch: Option<&str>,
) -> io::Result<(TempDir, PathBuf)> {
    let temp_dir = tempfile::tempdir()?;
    let clone_path = temp_dir.path().join("repo");

    let mut cmd = Command::new("git");
    cmd.arg("clone").arg("--depth").arg("1");
    if let Some(branch) = branch {
        cmd.args(["--branch", branch]);
    }
    cmd.arg(repo_url).arg(&clone_path);

    let output = cmd
        .output()
        .map_err(|e| io::Error::other(format!("Failed to run git clone: {e}")))?;
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        let stdout = String::from_utf8_lossy(&output.stdout);
        let mut msg = String::from("git clone failed");
        let details = if !stderr.trim().is_empty() {
            stderr.trim()
        } else {
            stdout.trim()
        };
        if !details.is_empty() {
            msg.push_str(": ");
            msg.push_str(details);
        }
        return Err(io::Error::other(msg));
    }

    Ok((temp_dir, clone_path))
}

pub fn run_on_repository(
    repo_url: &str,
    branch: Option<&str>,
    config: Config,
) -> io::Result<()> {
    let (temp_dir, clone_path) = clone_repository(repo_url, branch)?;

    let mut config = config;
    config.directory = clone_path;

    let result = crate::run_with_config(config);

    drop(temp_dir);

    result
}
