//! Default directory names that are always skipped during a scan, on top of
//! anything the user configures via `include`/`exclude` options.

use std::sync::LazyLock;

/// Raw contents of the bundled default-ignore list. See
/// `default_ignore_dirs.txt` for what's in it and why.
const RAW_IGNORED_DIRS: &str = include_str!("default_ignore_dirs.txt");

/// VCS and editor/tool config directories skipped regardless of
/// `.gitignore` state, parsed from [`RAW_IGNORED_DIRS`] (`#`-comments and
/// blank lines dropped).
pub static IGNORED_DIRS: LazyLock<Vec<&'static str>> = LazyLock::new(|| {
    RAW_IGNORED_DIRS
        .lines()
        .map(str::trim)
        .filter(|line| !line.is_empty() && !line.starts_with('#'))
        .collect()
});
