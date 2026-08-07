//! Builds the [`ignore::Walk`] iterator used to traverse a directory.

use ignore::overrides::{Override, OverrideBuilder};
use ignore::{Walk, WalkBuilder};
use std::io;
use std::path::Path;

use crate::config::Config;

/// Builds a directory walker rooted at `root`.
///
/// `ignored_dirs` (see [`crate::constants`]) is always excluded via override
/// patterns, on top of `config.exclude_dirs`. When `config.respect_gitignore`
/// is false, all of the walker's standard ignore sources (`.gitignore`,
/// `.git/info/exclude`, global gitignore, `.ignore`, parent directories) are
/// disabled.
pub fn build_walker(root: &Path, ignored_dirs: &[&str], config: &Config) -> io::Result<Walk> {
    let mut builder = WalkBuilder::new(root);
    builder.standard_filters(true);
    if !config.respect_gitignore {
        builder
            .ignore(false)
            .git_ignore(false)
            .git_global(false)
            .git_exclude(false)
            .parents(false);
    }
    builder.overrides(build_overrides(root, ignored_dirs, config)?);
    Ok(builder.build())
}

/// Builds the override glob set that excludes `ignored_dirs` and
/// `config.exclude_dirs` from the walk.
fn build_overrides(root: &Path, ignored_dirs: &[&str], config: &Config) -> io::Result<Override> {
    let mut builder = OverrideBuilder::new(root);
    builder.case_insensitive(true).map_err(|err| {
        io::Error::new(
            io::ErrorKind::InvalidInput,
            format!("override case sensitivity: {err}"),
        )
    })?;

    for dir in ignored_dirs {
        let pattern = format!("!{dir}/");
        builder.add(&pattern).map_err(|err| {
            io::Error::new(
                io::ErrorKind::InvalidInput,
                format!("override {pattern}: {err}"),
            )
        })?;
    }

    if let Some(exclude_dirs) = &config.exclude_dirs {
        for dir in exclude_dirs {
            let pattern = format!("!{dir}/");
            builder.add(&pattern).map_err(|err| {
                io::Error::new(
                    io::ErrorKind::InvalidInput,
                    format!("override {pattern}: {err}"),
                )
            })?;
        }
    }

    builder.build().map_err(|err| {
        io::Error::new(
            io::ErrorKind::InvalidInput,
            format!("override build: {err}"),
        )
    })
}
