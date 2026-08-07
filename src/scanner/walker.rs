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
/// appears in, unconditionally — unlike the knobs below, it's *not*
/// controlled by any `Config` field, since it's fyai's own dedicated exclude
/// mechanism rather than a git-ecosystem one.
///
/// Every other `ignore`-crate filter is controlled independently, rather
/// than as one all-or-nothing switch:
///
/// - `config.hidden` gates dot-files/dot-directories.
/// - `config.gitignore` gates `.gitignore`, `.git/info/exclude`, and
///   `.gitignore` files in parent directories, together (splitting these
///   three further isn't worth the surface area — they're all "git's own
///   ignore mechanism" and are essentially always toggled as a unit).
/// - `config.ignore_files` gates plain `.ignore` files, independently of
///   `.gitignore` (a repo without git can still use `.ignore`).
/// - `config.git_global` gates git's global excludes file.
/// - `config.follow_links` gates symlink traversal — a different axis
///   entirely (not an ignore rule at all), but exposed here too since it's
///   the same builder.
pub fn build_walker(config: &Config) -> io::Result<WalkParallel> {
    let mut builder = WalkBuilder::new(&config.directory);
    builder
        .hidden(config.hidden)
        .parents(config.gitignore)
        .ignore(config.ignore_files)
        .git_ignore(config.gitignore)
        .git_global(config.git_global)
        .git_exclude(config.gitignore)
        .follow_links(config.follow_links);
    builder.add_custom_ignore_filename(".fyaiignore");
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
