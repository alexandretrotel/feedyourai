//! Shared CLI implementation for the `feedyourai` binary and its `fyai`
//! alias. Included by both `main.rs` files via `#[path]`, so this same
//! source compiles twice, once per binary target.

use clap::{CommandFactory, FromArgMatches};
use color_eyre::eyre::{Result, WrapErr};

use self::commands::Cli;
use feedyourai::{config, run_git, run_local};

/// System-clipboard access for copying the combined output.
mod clipboard;
/// Argument parsing and the `init` subcommand.
mod commands;

/// Runs the CLI end to end: parses arguments, resolves configuration
/// (merging any `fyai.toml` with CLI flags), runs the combine, and reports
/// the result to stdout/stderr, including a best-effort clipboard copy.
pub(crate) fn run() -> Result<()> {
    color_eyre::install()?;

    let matches = Cli::command().get_matches();
    let cli = Cli::from_arg_matches(&matches).wrap_err("failed to parse arguments")?;

    if commands::init::handle_init_subcommand(&cli)? {
        return Ok(());
    }

    let repo_url = cli.repo.clone();
    let repo_branch = cli.repo_branch.clone();
    let repo_commit = cli.repo_commit.clone();

    let cli_config = commands::config_from_matches(matches)?;

    let file_config = match config::discover_config_file() {
        Some(path) => match config::PartialConfig::from_path(&path) {
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
                config::PartialConfig::default()
            }
        },
        None => config::PartialConfig::default(),
    };

    let config = config::merge_config(file_config, cli_config);
    let output_path = config.output.clone();
    let tree_only = config.tree_only;

    if let Some(repo_url) = repo_url {
        run_git(
            &repo_url,
            repo_branch.as_deref(),
            repo_commit.as_deref(),
            config,
        )
        .wrap_err("failed to process git repository")?;
    } else {
        run_local(config).wrap_err("failed to process local directory")?;
    }

    if tree_only {
        println!("Project tree written to {}", output_path.display());
        return Ok(());
    }

    let output_contents = std::fs::read_to_string(&output_path)
        .wrap_err_with(|| format!("failed to read output file {}", output_path.display()))?;

    println!("Files combined successfully into {}", output_path.display());

    match clipboard::copy_to_clipboard(&output_contents) {
        Ok(()) => println!("Output copied to clipboard successfully!"),
        Err(err) if clipboard::should_ignore_clipboard_error() => {
            eprintln!("Warning: clipboard unavailable; skipping copy. {}", err);
        }
        Err(err) => return Err(err),
    }

    Ok(())
}
