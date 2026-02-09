use crate::clipboard::copy_to_clipboard;
use crate::data::{IGNORED_DIRS, IGNORED_FILES};
use crate::error::{AppError, AppResult};
use crate::file_processing::{get_directory_structure, process_files};
use crate::gitignore::build_gitignore;

#[cfg(test)]
mod tests;

mod cli;
mod clipboard;
mod config;
mod data;
mod error;
mod file_processing;
mod gitignore;
mod repository;

/// Run the core application logic using a fully-resolved `Config`.
///
/// This function is extracted from `main` and made public so tests can call it
/// directly with a controlled `Config`.
pub fn run_with_config(config: crate::config::Config) -> AppResult<()> {
    let gitignore = build_gitignore(&config.directory, IGNORED_FILES, IGNORED_DIRS, &config)?;

    let dir_structure =
        get_directory_structure(&config.directory, &gitignore, IGNORED_DIRS, &config)?;

    if config.tree_only {
        std::fs::write(&config.output, &dir_structure)?;
        println!("Project tree written to {}", config.output.display());
    } else {
        process_files(&config, &gitignore, &dir_structure, IGNORED_DIRS)?;
        let mut copied = true;
        if let Err(err) = copy_to_clipboard(&config.output) {
            if matches!(err, AppError::Clipboard(_)) && should_ignore_clipboard_error() {
                copied = false;
                eprintln!("Warning: clipboard unavailable; skipping copy. {}", err);
            } else {
                return Err(err);
            }
        }
        println!(
            "Files combined successfully into {}",
            config.output.display()
        );
        if copied {
            println!("Output copied to clipboard successfully!");
        }
    }
    Ok(())
}

fn should_ignore_clipboard_error() -> bool {
    if std::env::var_os("CI").is_some() {
        return true;
    }
    if cfg!(target_os = "linux") {
        let has_display = std::env::var_os("DISPLAY").is_some()
            || std::env::var_os("WAYLAND_DISPLAY").is_some()
            || std::env::var_os("SWAYSOCK").is_some();
        return !has_display;
    }
    false
}

pub fn handle_init_subcommand(matches: &clap::ArgMatches) -> AppResult<bool> {
    if let Some(sub_m) = matches.subcommand_matches("init") {
        let global = sub_m.get_flag("global");
        let force = sub_m.get_flag("force");

        let (path, display_path) = if global {
            let cfg_dir = dirs::config_dir()
                .or_else(|| dirs::home_dir().map(|h| h.join(".config")))
                .expect("Could not determine config directory");
            std::fs::create_dir_all(&cfg_dir)?;
            let mut cfg_path = cfg_dir.clone();
            cfg_path.push("fyai.yaml");
            (cfg_path.clone(), cfg_path.display().to_string())
        } else {
            let local = std::path::PathBuf::from("./fyai.yaml");
            (local.clone(), local.display().to_string())
        };

        if path.exists() && !force {
            return Err(AppError::ConfigAlreadyExists { path: display_path });
        }

        let template = r#"# fyai.yaml - Configuration file for fyai
# All options are optional. CLI flags override config values.
# See README.md for details.

directory: .                # Input directory
output: fyai.txt            # Output file
include_dirs:               # Directories to include (list)
  - src
  - docs
exclude_dirs:               # Directories to exclude (list)
  - node_modules
  - dist
include_ext:                # File extensions to include (list)
  - md
  - txt
exclude_ext:                # File extensions to exclude (list)
  - log
  - tmp
include_files:              # File names to include (list)
  - README.md
  - main.rs
exclude_files:              # File names to exclude (list)
  - LICENSE
  - config.json
min_size: 10240             # Minimum file size in bytes
max_size: 512000            # Maximum file size in bytes
respect_gitignore: true     # Respect .gitignore rules
tree_only: false            # Only output directory tree, no file contents
"#;

        std::fs::write(&path, template)?;
        println!("Template config file written to {}", display_path);
        return Ok(true);
    }
    Ok(false)
}

fn main() -> AppResult<()> {
    let matches = crate::cli::create_commands().get_matches();

    // Handle init subcommand via helper so tests can call it directly.
    if handle_init_subcommand(&matches)? {
        return Ok(());
    }

    let repo_url = matches.get_one::<String>("repo").cloned();
    let repo_branch = matches.get_one::<String>("repo_branch").cloned();

    // Normal flow: parse CLI args and config file
    // `config_from_matches_with_explicit` returns both the parsed CLI `Config` and an
    // `ExplicitFlags` struct indicating which CLI options were explicitly set.
    let (cli_config, explicit) = crate::config::config_from_matches(matches)?;

    // Discover and load config file if present
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

    // Merge configs (CLI takes precedence, but allow file to provide values when CLI didn't explicitly set them)
    let config = crate::config::merge_config(file_config, cli_config, explicit);

    if let Some(repo_url) = repo_url {
        return crate::repository::run_on_repository(&repo_url, repo_branch.as_deref(), config);
    }

    // Delegate to the extracted function so it can be tested in isolation.
    run_with_config(config)
}
