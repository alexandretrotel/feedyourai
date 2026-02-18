use clap::{CommandFactory, FromArgMatches};

use crate::cli::Cli;
use crate::errors::AppResult;

mod cli;
mod commands;
mod config;
mod constants;
mod errors;
mod git;
mod scanner;
mod utils;

fn main() -> AppResult<()> {
    let matches = Cli::command().get_matches();
    let cli = Cli::from_arg_matches(&matches)?;

    if crate::commands::init::handle_init_subcommand(&cli)? {
        return Ok(());
    }

    let repo_url = cli.repo.clone();
    let repo_branch = cli.repo_branch.clone();
    let repo_commit = cli.repo_commit.clone();

    let (cli_config, explicit) = crate::config::config_from_matches(matches)?;

    let file_config = match crate::config::discover_config_file() {
        Some(path) => match crate::config::FileConfig::from_path(&path) {
            Ok(cfg) => {
                println!("Loaded config from: {}", path.display());
                cfg
            }
            Err(e) => {
                eprintln!(
                    "Warning: Failed to load config file ({}): {}",
                    path.display(),
                    e
                );
                crate::config::FileConfig::default()
            }
        },
        None => crate::config::FileConfig::default(),
    };

    let config = crate::config::merge_config(file_config, cli_config, explicit);

    if let Some(repo_url) = repo_url {
        return crate::git::run(
            &repo_url,
            repo_branch.as_deref(),
            repo_commit.as_deref(),
            config,
        );
    }

    crate::commands::run(config)
}
