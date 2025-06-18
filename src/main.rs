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

const IGNORED_FILES: &[&str] = &[
    "bun.lock",
    "package-lock.json",
    "yarn.lock",
    "pnpm-lock.yaml",
    "Cargo.lock",
    ".DS_Store",
    "uv.lock",
];

const IGNORED_DIRS: &[&str] = &[
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
    ".meteor",
    ".local",
    ".cache",
    ".config",
    ".trash",
    "cargo-target",
    ".mypy_cache",
    ".pylint.d",
    ".ropeproject",
    ".ipynb_checkpoints",
    ".parcel-cache",
    "coverage",
    "storybook-static",
    "bin",
    "pkg",
    ".gradle",
    ".settings",
    ".classpath",
    ".project",
    ".docker",
    ".husky",
    ".circleci",
    ".github",
    ".vercel",
    "k8s",
    "helm",
];

/// Main entry point for FeedYourAI.
/// Orchestrates CLI parsing, file processing, and clipboard operations.
///
/// # Returns
/// - `Ok(())`: On successful execution.
/// - `Err(io::Error)`: If an error occurs during execution
fn main() -> io::Result<()> {
    let config = parse_args()?;
    let gitignore = build_gitignore(
        &config.directory,
        IGNORED_FILES,
        IGNORED_DIRS,
        &config.exclude_dirs,
    )?;

    let dir_structure = get_directory_structure(
        &config.directory,
        &gitignore,
        &IGNORED_DIRS,
        &config.exclude_dirs,
    )?;
    process_files(&config, &gitignore, &dir_structure, IGNORED_DIRS)?;
    copy_to_clipboard(&config.output)?;

    println!(
        "Files combined successfully into {}",
        config.output.display()
    );
    println!("Output copied to clipboard successfully!");
    Ok(())
}
