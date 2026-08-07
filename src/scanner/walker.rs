//! Builds the [`ignore::Walk`] iterator used to traverse a directory.

use ignore::overrides::{Override, OverrideBuilder};
use ignore::{Walk, WalkBuilder};
use std::io;

use crate::config::Config;

/// Builds a directory walker rooted at `config.directory`.
///
/// `ignored_dirs` (see [`crate::constants`]) is always excluded via override
/// patterns, on top of `config.exclude_dirs`. When `config.respect_gitignore`
/// is false, all of the walker's standard ignore sources (`.gitignore`,
/// `.git/info/exclude`, global gitignore, `.ignore`, parent directories) are
/// disabled.
pub fn build_walker(config: &Config, ignored_dirs: &[&str]) -> io::Result<Walk> {
    let mut builder = WalkBuilder::new(&config.directory);
    builder.standard_filters(true);
    if !config.respect_gitignore {
        builder
            .ignore(false)
            .git_ignore(false)
            .git_global(false)
            .git_exclude(false)
            .parents(false);
    }
    builder.overrides(build_overrides(config, ignored_dirs)?);
    Ok(builder.build())
}

/// Builds the override glob set that excludes `ignored_dirs` and
/// `config.exclude_dirs` from the walk.
fn build_overrides(config: &Config, ignored_dirs: &[&str]) -> io::Result<Override> {
    let mut builder = OverrideBuilder::new(&config.directory);
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
