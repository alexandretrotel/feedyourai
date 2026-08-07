//! Core library for `feedyourai`: walks a directory (or a temporary clone of
//! a git repository), filters files according to a [`Config`](config::Config),
//! and combines the matching files into a single text file for feeding into
//! an LLM.
//!
//! This crate is intentionally silent and side-effect-free beyond writing
//! the requested output file: it never prints to stdout/stderr and never
//! touches the clipboard. Those concerns live in the `feedyourai`/`fyai`
//! binaries, which are thin CLI wrappers around [`run_local`] and
//! [`run_git`].

#![warn(missing_docs)]

/// Configuration types (CLI-agnostic) and config-file discovery/merging.
pub mod config;
/// The crate's error type.
pub mod error;
/// Orchestrates a single combine run against a local directory or a
/// temporary clone of a git repository.
pub mod runner;
/// Directory walking, filtering, and file-combining logic.
pub mod scanner;

pub use runner::{run_git, run_local};
