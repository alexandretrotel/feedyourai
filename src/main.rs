use std::io;

use crate::cli::parse_args;
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
    // Parse CLI args first
    let cli_config = parse_args()?;

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
