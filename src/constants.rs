//! Default directory names that are always skipped during a scan, on top of
//! anything the user configures via `include`/`exclude` options.

/// VCS and editor/tool config directories skipped regardless of
/// `.gitignore` state. Build/dependency/cache output (`node_modules`,
/// `target`, `dist`, `.venv`, ...) is deliberately *not* listed here: those
/// are almost universally covered by the project's own `.gitignore`, which
/// `respect_gitignore` already honors, so hardcoding them would just be
/// duplicate maintenance. What remains below is stuff that's routinely
/// *committed* (CI configs, IDE settings, VCS internals) and so wouldn't be
/// caught by `.gitignore` at all.
pub const IGNORED_DIRS: &[&str] = &[
    ".changeset",
    ".circleci",
    ".classpath",
    ".config",
    ".cursor",
    ".docker",
    ".git",
    ".github",
    ".hg",
    ".husky",
    ".idea",
    ".project",
    ".settings",
    ".svn",
    ".vscode",
];
