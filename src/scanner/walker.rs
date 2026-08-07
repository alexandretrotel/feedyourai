//! Builds the [`ignore::Walk`] iterator used to traverse a directory.

use ignore::overrides::{Override, OverrideBuilder};
use ignore::{Walk, WalkBuilder};
use std::io;

use crate::config::Config;

/// Builds a directory walker rooted at `config.directory`.
///
/// `config.exclude_dirs` is always excluded via override patterns. A
/// `.fyaiignore` file (gitignore syntax) is honored in any directory it
/// appears in, unconditionally — unlike `.gitignore`/`.ignore`/global
/// gitignore/`.git/info/exclude`, it is *not* affected by
/// `config.respect_gitignore`, since it's fyai's own dedicated exclude
/// mechanism rather than a git-ecosystem one. When `config.respect_gitignore`
/// is false, only those git-ecosystem sources (plus parent-directory
/// lookups) are disabled.
pub fn build_walker(config: &Config) -> io::Result<Walk> {
    let mut builder = WalkBuilder::new(&config.directory);
    builder.standard_filters(true);
    builder.add_custom_ignore_filename(".fyaiignore");
    if !config.respect_gitignore {
        builder
            .ignore(false)
            .git_ignore(false)
            .git_global(false)
            .git_exclude(false)
            .parents(false);
    }
    builder.overrides(build_overrides(config)?);
    Ok(builder.build())
}

/// Builds the override glob set that excludes `config.exclude_dirs` from
/// the walk.
fn build_overrides(config: &Config) -> io::Result<Override> {
    let mut builder = OverrideBuilder::new(&config.directory);
    builder.case_insensitive(true).map_err(|err| {
        io::Error::new(
            io::ErrorKind::InvalidInput,
            format!("override case sensitivity: {err}"),
        )
    })?;

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
