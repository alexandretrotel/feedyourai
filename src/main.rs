use std::io;

use crate::cli::parse_args;
use crate::clipboard::copy_to_clipboard;
use crate::file_processing::{get_directory_structure, process_files};
use crate::gitignore::build_gitignore;

#[cfg(test)]
mod tests;

mod cli;
mod clipboard;
mod file_processing;
mod gitignore;

/// Main entry point for FeedYourAI.
/// Orchestrates CLI parsing, file processing, and clipboard operations.
///
/// # Returns
/// - `Ok(())`: On successful execution.
/// - `Err(io::Error)`: If an error occurs during execution
fn main() -> io::Result<()> {
    let config = parse_args()?;
    let gitignore = build_gitignore(&config.directory)?;
    let ignored_dirs = [
        "node_modules",
        ".git",
        ".svn",
        ".hg",
        ".idea",
        ".vscode",
        "build",
        "dist",
        "src-tauri",
        ".venv",
        "__pycache__",
        ".pytest_cache",
        ".next",
        ".turbo",
        "out",
        "target",
    ];

    let dir_structure = get_directory_structure(&config.directory, &gitignore, &ignored_dirs)?;
    process_files(&config, &gitignore, &ignored_dirs, &dir_structure)?;
    copy_to_clipboard(&config.output)?;

    println!(
        "Files combined successfully into {}",
        config.output.display()
    );
    println!("Output copied to clipboard successfully!");
    Ok(())
}
