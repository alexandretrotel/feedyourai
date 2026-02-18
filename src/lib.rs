pub mod commands;
pub mod config;
pub mod constants;
pub mod errors;
pub mod git;
pub mod scanner;
pub mod utils;

use config::Config;
use errors::AppResult;

pub fn run_local(config: Config) -> AppResult<()> {
    commands::run(config)
}

pub fn run_git(
    repo_url: &str,
    branch: Option<&str>,
    commit: Option<&str>,
    config: Config,
) -> AppResult<()> {
    git::run(repo_url, branch, commit, config)
}
