use std::io;

use crate::clipboard::copy_to_clipboard;
use crate::data::{IGNORED_DIRS, IGNORED_FILES};
use crate::file_processing::{get_directory_structure, process_files};
use crate::gitignore::build_gitignore;

#[cfg(test)]
mod tests;

mod cli;
mod clipboard;
mod config;
mod data;
mod file_processing;
mod gitignore;

fn main() -> io::Result<()> {
    let matches = crate::cli::create_commands().get_matches();

    // Handle init subcommand
    if let Some(sub_m) = matches.subcommand_matches("init") {
        let global = sub_m.get_flag("global");
        let force = sub_m.get_flag("force");

        let (path, display_path) = if global {
            let mut home = dirs::home_dir().expect("Could not determine home directory");
            home.push(".fyai");
            std::fs::create_dir_all(&home)?;
            home.push("config.yaml");
            (home.clone(), home.display().to_string())
        } else {
            let local = std::path::PathBuf::from("./fyai.yaml");
            (local.clone(), local.display().to_string())
        };

        if path.exists() && !force {
            eprintln!(
                "Config file already exists at {}. Use --force to overwrite.",
                display_path
            );
            std::process::exit(1);
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
        return Ok(());
    }

    // Normal flow: parse CLI args and config file
    let cli_config = crate::config::config_from_matches(matches)?;

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

    // Merge configs (CLI takes precedence)
    let config = crate::config::merge_config(file_config, cli_config);

    let gitignore = build_gitignore(&config.directory, IGNORED_FILES, IGNORED_DIRS, &config)?;

    let dir_structure =
        get_directory_structure(&config.directory, &gitignore, IGNORED_DIRS, &config)?;

    if config.tree_only {
        std::fs::write(&config.output, &dir_structure)?;
        println!("Project tree written to {}", config.output.display());
    } else {
        process_files(&config, &gitignore, &dir_structure, IGNORED_DIRS)?;
        copy_to_clipboard(&config.output)?;
        println!(
            "Files combined successfully into {}",
            config.output.display()
        );
        println!("Output copied to clipboard successfully!");
    }
    Ok(())
}
