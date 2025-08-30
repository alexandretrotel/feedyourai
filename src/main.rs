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

/// Files to ignore during directory scanning and processing.
///
/// These are typically lock files, system files, or files that don't need indexing.
const IGNORED_FILES: &[&str] = &[
    "bun.lock",
    "package-lock.json",
    "yarn.lock",
    "pnpm-lock.yaml",
    "Cargo.lock",
    ".DS_Store",
    "uv.lock",
];

/// Directories to ignore during directory scanning and processing.
///
/// Common build directories, VCS folders, caches, and IDE config folders.
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
    ".changeset",
    ".cursor",
    ".vite",
];

/// Main entry point for fyai application.
///
/// Orchestrates the workflow by:
/// 1. Parsing CLI arguments to get user configuration.
/// 2. Building `.gitignore` rules incorporating default and user-excluded files/dirs.
/// 3. Recursively scanning the target directory respecting ignore rules.
/// 4. Processing matched files according to config.
/// 5. Copying the final combined output to the system clipboard.
/// 6. Printing success messages.
///
/// # Errors
/// Returns an [`io::Error`] if any filesystem or IO operation fails during the process.
///
/// # Examples
/// ```no_run
/// fn main() -> std::io::Result<()> {
///     // Your application logic here
///     Ok(())
/// }
/// ```
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

    if config.tree_only {
        // Only output the directory tree to the output file
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
