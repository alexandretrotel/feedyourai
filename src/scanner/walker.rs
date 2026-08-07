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

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashSet;
    use std::fs;
    use std::path::PathBuf;
    use std::sync::{Arc, Mutex};

    /// A baseline [`Config`] with every filter at its "do nothing special"
    /// default. Individual tests override only the fields they care about.
    fn base_config(directory: &std::path::Path) -> Config {
        Config {
            directory: directory.to_path_buf(),
            output: PathBuf::from("fyai.txt"),
            include_dirs: None,
            exclude_dirs: None,
            include_ext: None,
            exclude_ext: None,
            include_files: None,
            exclude_files: None,
            min_size: None,
            max_size: None,
            hidden: true,
            gitignore: true,
            ignore_files: true,
            git_global: true,
            follow_links: false,
            tree_only: false,
            human: false,
        }
    }

    /// Runs `config`'s walker to completion and returns every visited path
    /// (including the root itself), unordered.
    fn run_walker(config: &Config) -> HashSet<PathBuf> {
        let walker = build_walker(config).expect("build_walker");
        let visited: Arc<Mutex<Vec<PathBuf>>> = Arc::new(Mutex::new(Vec::new()));

        walker.run(|| {
            let visited = Arc::clone(&visited);
            Box::new(move |result| {
                if let Ok(entry) = result {
                    visited.lock().unwrap().push(entry.path().to_path_buf());
                }
                ignore::WalkState::Continue
            })
        });

        Arc::try_unwrap(visited)
            .expect("no outstanding references")
            .into_inner()
            .unwrap()
            .into_iter()
            .collect()
    }

    #[test]
    fn build_walker_visits_all_files_and_dirs() {
        let dir = tempfile::tempdir().expect("tempdir");
        fs::write(dir.path().join("a.txt"), b"a").expect("write");
        fs::create_dir_all(dir.path().join("subdir")).expect("create_dir_all");
        fs::write(dir.path().join("subdir").join("b.txt"), b"b").expect("write");

        let config = base_config(dir.path());
        let visited = run_walker(&config);

        let expected: HashSet<PathBuf> = [
            dir.path().to_path_buf(),
            dir.path().join("a.txt"),
            dir.path().join("subdir"),
            dir.path().join("subdir").join("b.txt"),
        ]
        .into_iter()
        .collect();

        assert_eq!(visited, expected);
    }

    #[test]
    fn hidden_true_excludes_dot_files() {
        let dir = tempfile::tempdir().expect("tempdir");
        fs::write(dir.path().join(".hidden_file"), b"hidden").expect("write");
        fs::write(dir.path().join("normal_file"), b"normal").expect("write");

        let mut config = base_config(dir.path());
        config.hidden = true;
        let visited = run_walker(&config);

        assert!(!visited.contains(&dir.path().join(".hidden_file")));
        assert!(visited.contains(&dir.path().join("normal_file")));
    }

    #[test]
    fn hidden_false_includes_dot_files() {
        let dir = tempfile::tempdir().expect("tempdir");
        fs::write(dir.path().join(".hidden_file"), b"hidden").expect("write");
        fs::write(dir.path().join("normal_file"), b"normal").expect("write");

        let mut config = base_config(dir.path());
        config.hidden = false;
        let visited = run_walker(&config);

        assert!(visited.contains(&dir.path().join(".hidden_file")));
        assert!(visited.contains(&dir.path().join("normal_file")));
    }

    #[test]
    fn exclude_dirs_removes_matching_directory_and_its_contents() {
        let dir = tempfile::tempdir().expect("tempdir");
        fs::create_dir_all(dir.path().join("node_modules")).expect("create_dir_all");
        fs::write(dir.path().join("node_modules").join("foo.txt"), b"foo").expect("write");
        fs::create_dir_all(dir.path().join("src")).expect("create_dir_all");
        fs::write(dir.path().join("src").join("bar.txt"), b"bar").expect("write");

        let mut config = base_config(dir.path());
        config.exclude_dirs = Some(vec!["node_modules".to_string()]);
        let visited = run_walker(&config);

        assert!(!visited.contains(&dir.path().join("node_modules")));
        assert!(!visited.contains(&dir.path().join("node_modules").join("foo.txt")));
        assert!(visited.contains(&dir.path().join("src").join("bar.txt")));
    }

    #[test]
    fn exclude_dirs_is_case_insensitive() {
        let dir = tempfile::tempdir().expect("tempdir");
        fs::create_dir_all(dir.path().join("Node_Modules")).expect("create_dir_all");
        fs::write(dir.path().join("Node_Modules").join("foo.txt"), b"foo").expect("write");

        let mut config = base_config(dir.path());
        config.exclude_dirs = Some(vec!["node_modules".to_string()]);
        let visited = run_walker(&config);

        assert!(!visited.contains(&dir.path().join("Node_Modules")));
        assert!(!visited.contains(&dir.path().join("Node_Modules").join("foo.txt")));
    }

    #[test]
    fn fyaiignore_file_is_always_honored_regardless_of_gitignore_and_ignore_files_settings() {
        let dir = tempfile::tempdir().expect("tempdir");
        fs::write(dir.path().join(".fyaiignore"), "ignored.txt\n").expect("write");
        fs::write(dir.path().join("ignored.txt"), b"skip me").expect("write");
        fs::write(dir.path().join("kept.txt"), b"keep me").expect("write");

        let mut config = base_config(dir.path());
        config.gitignore = false;
        config.ignore_files = false;
        let visited = run_walker(&config);

        assert!(!visited.contains(&dir.path().join("ignored.txt")));
        assert!(visited.contains(&dir.path().join("kept.txt")));
    }

    #[cfg(unix)]
    #[test]
    fn follow_links_false_does_not_descend_into_symlinked_directories() {
        use std::os::unix::fs::symlink;

        let dir = tempfile::tempdir().expect("tempdir");
        fs::create_dir_all(dir.path().join("target")).expect("create_dir_all");
        fs::write(dir.path().join("target").join("file.txt"), b"content").expect("write");
        symlink(dir.path().join("target"), dir.path().join("link")).expect("symlink");

        let mut config = base_config(dir.path());
        config.follow_links = false;
        let visited = run_walker(&config);

        assert!(visited.contains(&dir.path().join("link")));
        assert!(!visited.contains(&dir.path().join("link").join("file.txt")));
        // The real directory is still walked normally.
        assert!(visited.contains(&dir.path().join("target").join("file.txt")));
    }

    #[cfg(unix)]
    #[test]
    fn follow_links_true_descends_into_symlinked_directories() {
        use std::os::unix::fs::symlink;

        let dir = tempfile::tempdir().expect("tempdir");
        fs::create_dir_all(dir.path().join("target")).expect("create_dir_all");
        fs::write(dir.path().join("target").join("file.txt"), b"content").expect("write");
        symlink(dir.path().join("target"), dir.path().join("link")).expect("symlink");

        let mut config = base_config(dir.path());
        config.follow_links = true;
        let visited = run_walker(&config);

        assert!(visited.contains(&dir.path().join("link").join("file.txt")));
    }

    // ---- build_overrides --------------------------------------------------

    #[test]
    fn build_overrides_with_no_exclude_dirs_builds_an_empty_override_set() {
        let dir = tempfile::tempdir().expect("tempdir");
        let config = base_config(dir.path());

        let overrides = build_overrides(&config).expect("build_overrides");
        // No patterns were added, so nothing is matched (an empty override
        // set matches everything, since there's nothing to exclude).
        assert!(overrides.is_empty());
    }

    #[test]
    fn build_overrides_with_exclude_dirs_builds_a_non_empty_override_set() {
        let dir = tempfile::tempdir().expect("tempdir");
        let mut config = base_config(dir.path());
        config.exclude_dirs = Some(vec!["node_modules".to_string(), "target".to_string()]);

        let overrides = build_overrides(&config).expect("build_overrides");
        assert!(!overrides.is_empty());
    }
}
