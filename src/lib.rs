//! Core library for `feedyourai`: walks a directory (or a temporary clone of
//! a git repository), filters files according to a [`config::Config`], and
//! combines the matching files into a single text file for feeding into an
//! LLM.
//!
//! This crate is intentionally silent and side-effect-free beyond writing
//! the requested output file: it never prints to stdout/stderr and never
//! touches the clipboard. Those concerns live in the `feedyourai`/`fyai`
//! binaries, which are thin CLI wrappers around [`run_local`] and
//! [`run_git`].

#![warn(missing_docs)]

/// Orchestrates a single combine run: builds the directory tree, then either
/// writes the tree only or writes the tree plus every matching file's
/// contents.
pub mod commands;
/// Configuration types (CLI-agnostic) and config-file discovery/merging.
pub mod config;
/// Default file/directory names that are always skipped during a scan.
pub mod constants;
/// The crate's error type.
pub mod error;
/// Cloning a remote git repository into a temporary directory before
/// scanning it.
pub mod git;
/// Directory walking, filtering, and file-combining logic.
pub mod scanner;

use config::Config;
use error::Result;

/// Combines files from a local directory as described by `config`.
///
/// Writes the result to `config.output` (either the directory tree only, or
/// the tree plus file contents, depending on `config.tree_only`).
pub fn run_local(config: Config) -> Result<()> {
    commands::run(config)
}

/// Clones `repo_url` into a temporary directory, then runs the same combine
/// logic as [`run_local`] against the clone.
///
/// The temporary clone is removed before this function returns, regardless
/// of the run's outcome.
///
/// # Arguments
///
/// * `branch` - an optional branch or tag to check out.
/// * `commit` - an optional commit SHA to check out after cloning. When set,
///   the clone is not shallow, since the target commit may not be reachable
///   from a depth-1 clone.
/// * `config` - the combine configuration; `config.directory` is overwritten
///   with the path to the cloned repository.
pub fn run_git(
    repo_url: &str,
    branch: Option<&str>,
    commit: Option<&str>,
    config: Config,
) -> Result<()> {
    git::run(repo_url, branch, commit, config)
}
