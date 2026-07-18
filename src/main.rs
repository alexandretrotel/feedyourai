use clap::{CommandFactory, FromArgMatches};
use eyre::{Result, WrapErr};

use crate::cli::Cli;
use feedyourai::{config, run_git, run_local};

mod cli;

fn main() -> Result<()> {
    color_eyre::install()?;

    let matches = Cli::command().get_matches();
    let cli = Cli::from_arg_matches(&matches).wrap_err("failed to parse arguments")?;

    if cli::init::handle_init_subcommand(&cli)? {
        return Ok(());
    }

    let repo_url = cli.repo.clone();
    let repo_branch = cli.repo_branch.clone();
    let repo_commit = cli.repo_commit.clone();

    let (cli_config, explicit) = cli::config_from_matches(matches)?;

    let file_config = match config::discover_config_file() {
        Some(path) => match config::FileConfig::from_path(&path) {
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
                config::FileConfig::default()
            }
        },
        None => config::FileConfig::default(),
    };

    let config = config::merge_config(file_config, cli_config, explicit);

    if let Some(repo_url) = repo_url {
        return run_git(
            &repo_url,
            repo_branch.as_deref(),
            repo_commit.as_deref(),
            config,
        )
        .wrap_err("failed to process git repository");
    }

    run_local(config).wrap_err("failed to process local directory")
}
