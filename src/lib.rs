pub mod commands;
pub mod config;
pub mod constants;
pub mod git;
pub mod scanner;
pub mod utils;

use config::Config;
use color_eyre::eyre::Result;

pub fn run_local(config: Config) -> Result<()> {
    commands::run(config)
}

pub fn run_git(
    repo_url: &str,
    branch: Option<&str>,
    commit: Option<&str>,
    config: Config,
) -> Result<()> {
    git::run(repo_url, branch, commit, config)
}
