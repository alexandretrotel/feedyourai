//! Orchestrates a single combine run: builds the directory tree, then either
//! writes the tree only or writes the tree plus every matching file's
//! contents.

use std::fs;

use crate::config::Config;
use crate::constants::IGNORED_DIRS;
use crate::error::Result;
use crate::scanner::{get_directory_structure, process_files};

/// Runs a combine pass against `config.directory`, writing the result to
/// `config.output`.
///
/// When `config.tree_only` is set, only the directory tree is written;
/// otherwise the tree is followed by the contents of every file that passes
/// the configured filters.
pub fn run(config: Config) -> Result<()> {
    let ignored_dirs = IGNORED_DIRS.as_slice();
    let dir_structure = get_directory_structure(&config, ignored_dirs)?;

    if config.tree_only {
        fs::write(&config.output, &dir_structure)?;
    } else {
        process_files(&config, &dir_structure, ignored_dirs)?;
    }

    Ok(())
}
