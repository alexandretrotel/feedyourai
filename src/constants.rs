//! Default directory names that are always skipped during a scan, on top of
//! anything the user configures via `include`/`exclude` options.

/// VCS and editor/tool config directories skipped regardless of
/// `.gitignore` state.
///
/// Build/dependency/cache output (`node_modules`, `target`, `dist`,
/// `.venv`, ...) is deliberately *not* listed here: those are almost
/// universally covered by a project's own `.gitignore`, which
/// `respect_gitignore` already honors. What's listed below is routinely
/// *committed* to version control (CI configs, IDE settings, VCS
/// internals), so `.gitignore` never catches it.
///
/// Editor/IDE entries mirror github/gitignore's `Global/` templates
/// (`VisualStudioCode.gitignore`, `JetBrains.gitignore`,
/// `Eclipse.gitignore`). Re-sync against
/// <https://github.com/github/gitignore/tree/main/Global> periodically to
/// pick up new tooling.
pub const IGNORED_DIRS: &[&str] = &[
    // VCS internals
    ".git",
    ".hg",
    ".svn",
    // Editor / IDE (github/gitignore Global/VisualStudioCode,
    // Global/JetBrains, Global/Eclipse)
    ".vscode",
    ".idea",
    ".classpath",
    ".project",
    ".settings",
    // CI / tooling config, routinely committed
    ".changeset",
    ".circleci",
    ".config",
    ".cursor",
    ".docker",
    ".github",
    ".husky",
];
