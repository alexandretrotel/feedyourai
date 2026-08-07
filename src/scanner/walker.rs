//! Builds the [`ignore::WalkParallel`] iterator used to traverse a
//! directory.

use ignore::overrides::{Override, OverrideBuilder};
use ignore::{WalkBuilder, WalkParallel};
use std::io;

use crate::config::Config;

/// Builds a multi-threaded directory walker rooted at `config.directory`.
///
/// Runs across `ignore`'s default thread pool (sized to the available
/// parallelism) instead of walking single-threaded, since directory
/// traversal is largely I/O-bound and benefits from overlapping `stat`
/// calls across threads. Entries arrive in no particular order; callers
/// that need a deterministic, preorder sequence (e.g. for tree rendering)
/// must sort the collected entries by path afterward — see
/// [`super::collect::collect_entries`].
///
/// `config.exclude_dirs` is always excluded via override patterns. A
/// `.fyaiignore` file (gitignore syntax) is honored in any directory it
/// appears in, unconditionally — unlike `.gitignore`/`.ignore`/global
/// gitignore/`.git/info/exclude`, it is *not* affected by
/// `config.respect_gitignore`, since it's fyai's own dedicated exclude
/// mechanism rather than a git-ecosystem one. When `config.respect_gitignore`
/// is false, all of those git-ecosystem sources (plus parent-directory
/// lookups) are disabled, *and* dot-files/dot-directories (hidden entries)
/// are walked too, since `ignore`'s hidden-file filter is otherwise on by
/// default alongside them.
pub fn build_walker(config: &Config) -> io::Result<WalkParallel> {
    let mut builder = WalkBuilder::new(&config.directory);
    builder.standard_filters(true);
    builder.add_custom_ignore_filename(".fyaiignore");
    if !config.respect_gitignore {
        builder
            .hidden(false)
            .ignore(false)
            .git_ignore(false)
            .git_global(false)
            .git_exclude(false)
            .parents(false);
    }
    builder.overrides(build_overrides(config)?);
    Ok(builder.build_parallel())
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
