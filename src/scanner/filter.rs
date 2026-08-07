//! Per-entry include/exclude decisions, layered on top of the walker's own
//! ignore rules.

use std::collections::HashSet;
use std::ffi::OsString;
use std::path::Path;

use crate::config::Config;

/// Decides whether a walked path should be included in the scan, based on
/// the output file's own path and the configured include/exclude filters.
///
/// Built once per run and shared (by reference, across threads) for every
/// entry the walker yields.
pub struct PathFilter<'a> {
    /// The run's full configuration, consulted for the output path itself
    /// and for anything [`NormalizedFilterConfig`] doesn't already cover.
    config: &'a Config,
    /// `config.output`'s file name, checked before falling back to
    /// [`same_file::is_same_file`] in [`PathFilter::is_output_path`].
    output_file_name: Option<OsString>,
    /// Lower-cased, set-based view of `config`'s include/exclude lists.
    normalized_filters: NormalizedFilterConfig,
}

impl<'a> PathFilter<'a> {
    /// Creates a filter for `config`.
    pub fn new(config: &'a Config) -> Self {
        let output_file_name = config.output.file_name().map(|name| name.to_os_string());
        let normalized_filters = NormalizedFilterConfig::new(config);

        Self {
            config,
            output_file_name,
            normalized_filters,
        }
    }

    /// Returns whether `path` should be walked into (if a directory) or
    /// included in the output (if a file).
    pub fn allows_entry(&self, path: &Path, is_dir: bool) -> bool {
        if self.is_output_path(path) {
            return false;
        }
        if !self.is_dir_allowed(path) {
            return false;
        }
        if is_dir {
            return true;
        }
        self.is_file_allowed(path)
    }

    /// Returns true if `path` is the run's own output file, which must never
    /// be read back into itself.
    ///
    /// Almost every walked entry has a different file name than the output
    /// file, so that's checked first with no syscalls; only a name match
    /// pays for [`same_file::is_same_file`]'s two `stat` calls, which is far
    /// cheaper than canonicalizing (resolving every symlink in) both full
    /// paths on every single entry.
    fn is_output_path(&self, path: &Path) -> bool {
        if path.file_name() != self.output_file_name.as_deref() {
            return false;
        }
        same_file::is_same_file(path, &self.config.output)
            .unwrap_or_else(|_| path == self.config.output)
    }

    fn is_dir_allowed(&self, path: &Path) -> bool {
        if !self.matches_included_dir(path) {
            return false;
        }
        if self.matches_ignored_dir(path) {
            return false;
        }
        true
    }

    fn matches_included_dir(&self, path: &Path) -> bool {
        match &self.normalized_filters.include_dirs {
            Some(includes) => any_component_in_set(path, includes),
            None => true,
        }
    }

    fn matches_ignored_dir(&self, path: &Path) -> bool {
        match &self.normalized_filters.exclude_dirs {
            Some(excludes) => any_component_in_set(path, excludes),
            None => false,
        }
    }

    fn is_file_allowed(&self, path: &Path) -> bool {
        if !self.file_name_allowed(path) {
            return false;
        }
        if !self.extension_allowed(path) {
            return false;
        }
        true
    }

    fn file_name_allowed(&self, path: &Path) -> bool {
        let excludes = &self.normalized_filters.exclude_files;
        let includes = &self.normalized_filters.include_files;
        if excludes.is_none() && includes.is_none() {
            return true;
        }

        let file_name = path
            .file_name()
            .and_then(|f| f.to_str())
            .unwrap_or_default()
            .to_lowercase();

        if let Some(excludes) = excludes
            && excludes.contains(&file_name)
        {
            return false;
        }

        match includes {
            Some(includes) => includes.contains(&file_name),
            None => true,
        }
    }

    fn extension_allowed(&self, path: &Path) -> bool {
        let excludes = &self.normalized_filters.exclude_ext;
        let includes = &self.normalized_filters.include_ext;
        if excludes.is_none() && includes.is_none() {
            return true;
        }

        let ext = path
            .extension()
            .and_then(|e| e.to_str())
            .unwrap_or("")
            .to_lowercase();

        if let Some(excludes) = excludes
            && excludes.contains(&ext)
        {
            return false;
        }

        match includes {
            Some(includes) => includes.contains(&ext),
            None => true,
        }
    }
}

/// Lower-cased, set-based view of a [`Config`]'s include/exclude lists, built
/// once so per-entry checks are O(1) lookups instead of repeated scans.
struct NormalizedFilterConfig {
    /// Lower-cased [`Config::include_dirs`](crate::config::Config::include_dirs).
    include_dirs: Option<HashSet<String>>,
    /// Lower-cased [`Config::exclude_dirs`](crate::config::Config::exclude_dirs).
    exclude_dirs: Option<HashSet<String>>,
    /// Lower-cased [`Config::include_files`](crate::config::Config::include_files).
    include_files: Option<HashSet<String>>,
    /// Lower-cased [`Config::exclude_files`](crate::config::Config::exclude_files).
    exclude_files: Option<HashSet<String>>,
    /// Lower-cased [`Config::include_ext`](crate::config::Config::include_ext).
    include_ext: Option<HashSet<String>>,
    /// Lower-cased [`Config::exclude_ext`](crate::config::Config::exclude_ext).
    exclude_ext: Option<HashSet<String>>,
}

impl NormalizedFilterConfig {
    fn new(config: &Config) -> Self {
        Self {
            include_dirs: normalize_list(&config.include_dirs),
            exclude_dirs: normalize_list(&config.exclude_dirs),
            include_files: normalize_list(&config.include_files),
            exclude_files: normalize_list(&config.exclude_files),
            include_ext: normalize_list(&config.include_ext),
            exclude_ext: normalize_list(&config.exclude_ext),
        }
    }
}

/// Lower-cases every item of `list` into a [`HashSet`], or returns `None` if
/// `list` is `None`.
fn normalize_list(list: &Option<Vec<String>>) -> Option<HashSet<String>> {
    list.as_ref()
        .map(|items| items.iter().map(|item| item.to_lowercase()).collect())
}

/// Returns true if any path component of `path`, lower-cased, is in `set`.
fn any_component_in_set(path: &Path, set: &HashSet<String>) -> bool {
    path.components().any(|component| {
        component
            .as_os_str()
            .to_str()
            .map(|name| set.contains(&name.to_lowercase()))
            .unwrap_or(false)
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::path::PathBuf;

    /// A baseline [`Config`] with every filter disabled, matching the task's
    /// suggested defaults. Individual tests override only the fields they
    /// care about.
    fn base_config() -> Config {
        Config {
            directory: PathBuf::from("."),
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

    // ---- is_output_path ------------------------------------------------

    #[test]
    fn is_output_path_true_for_the_same_real_file() {
        let dir = tempfile::tempdir().unwrap();
        let output_path = dir.path().join("fyai.txt");
        fs::write(&output_path, b"hello").unwrap();

        let mut config = base_config();
        config.output = output_path.clone();
        let filter = PathFilter::new(&config);

        assert!(filter.is_output_path(&output_path));
        assert!(!filter.allows_entry(&output_path, false));
    }

    #[test]
    fn is_output_path_false_for_a_differently_named_file() {
        let dir = tempfile::tempdir().unwrap();
        let output_path = dir.path().join("fyai.txt");
        let other_path = dir.path().join("other.rs");
        fs::write(&output_path, b"hello").unwrap();
        fs::write(&other_path, b"fn main() {}").unwrap();

        let mut config = base_config();
        config.output = output_path;
        let filter = PathFilter::new(&config);

        assert!(!filter.is_output_path(&other_path));
    }

    #[test]
    fn is_output_path_false_for_distinct_files_sharing_a_name() {
        let dir_a = tempfile::tempdir().unwrap();
        let dir_b = tempfile::tempdir().unwrap();
        let output_path = dir_a.path().join("fyai.txt");
        let lookalike_path = dir_b.path().join("fyai.txt");
        fs::write(&output_path, b"real output").unwrap();
        fs::write(&lookalike_path, b"a different file, same name").unwrap();

        let mut config = base_config();
        config.output = output_path;
        let filter = PathFilter::new(&config);

        // Same file name, but `same_file::is_same_file` should determine
        // they're genuinely different files on disk.
        assert!(!filter.is_output_path(&lookalike_path));
    }

    // ---- is_dir_allowed / matches_included_dir / matches_ignored_dir ----

    #[test]
    fn matches_included_dir_allows_paths_containing_the_component() {
        let mut config = base_config();
        config.include_dirs = Some(vec!["src".into()]);
        let filter = PathFilter::new(&config);

        assert!(filter.is_dir_allowed(Path::new("project/src/scanner")));
        assert!(!filter.is_dir_allowed(Path::new("project/tests")));
    }

    #[test]
    fn matches_included_dir_is_case_insensitive() {
        let mut config = base_config();
        config.include_dirs = Some(vec!["src".into()]);
        let filter = PathFilter::new(&config);

        assert!(filter.is_dir_allowed(Path::new("project/SRC/scanner")));
    }

    #[test]
    fn matches_ignored_dir_excludes_paths_containing_the_component() {
        let mut config = base_config();
        config.exclude_dirs = Some(vec!["node_modules".into()]);
        let filter = PathFilter::new(&config);

        assert!(!filter.is_dir_allowed(Path::new("project/node_modules/pkg")));
        assert!(filter.is_dir_allowed(Path::new("project/src")));
    }

    #[test]
    fn exclude_dirs_wins_over_include_dirs_when_both_match() {
        let mut config = base_config();
        config.include_dirs = Some(vec!["src".into()]);
        config.exclude_dirs = Some(vec!["node_modules".into()]);
        let filter = PathFilter::new(&config);

        // Matches the include list (`src`) AND the exclude list
        // (`node_modules`): exclude must win.
        assert!(!filter.is_dir_allowed(Path::new("project/src/node_modules")));
        // Matches include only: allowed.
        assert!(filter.is_dir_allowed(Path::new("project/src/scanner")));
        // Matches neither: not allowed (include list is set and doesn't
        // match).
        assert!(!filter.is_dir_allowed(Path::new("project/docs")));
    }

    #[test]
    fn dir_allowed_by_default_when_no_dir_filters_set() {
        let config = base_config();
        let filter = PathFilter::new(&config);

        assert!(filter.is_dir_allowed(Path::new("anything/goes")));
    }

    // ---- is_file_allowed / file_name_allowed / extension_allowed --------

    #[test]
    fn file_name_allowed_matches_include_list_case_insensitively() {
        let mut config = base_config();
        config.include_files = Some(vec!["readme.md".into()]);
        let filter = PathFilter::new(&config);

        assert!(filter.file_name_allowed(Path::new("README.md")));
        assert!(!filter.file_name_allowed(Path::new("other.md")));
    }

    #[test]
    fn file_name_allowed_matches_exclude_list_case_insensitively() {
        let mut config = base_config();
        config.exclude_files = Some(vec!["secret.txt".into()]);
        let filter = PathFilter::new(&config);

        assert!(!filter.file_name_allowed(Path::new("SECRET.txt")));
        assert!(filter.file_name_allowed(Path::new("public.txt")));
    }

    #[test]
    fn exclude_files_wins_over_include_files_when_both_match() {
        let mut config = base_config();
        config.include_files = Some(vec!["readme.md".into()]);
        config.exclude_files = Some(vec!["readme.md".into()]);
        let filter = PathFilter::new(&config);

        assert!(!filter.file_name_allowed(Path::new("readme.md")));
    }

    #[test]
    fn file_name_allowed_by_default_when_neither_list_set() {
        let config = base_config();
        let filter = PathFilter::new(&config);

        assert!(filter.file_name_allowed(Path::new("anything.rs")));
    }

    #[test]
    fn extension_allowed_matches_include_list_without_the_dot_case_insensitively() {
        let mut config = base_config();
        config.include_ext = Some(vec!["rs".into()]);
        let filter = PathFilter::new(&config);

        assert!(filter.extension_allowed(Path::new("main.RS")));
        assert!(!filter.extension_allowed(Path::new("main.py")));
    }

    #[test]
    fn extension_allowed_matches_exclude_list_case_insensitively() {
        let mut config = base_config();
        config.exclude_ext = Some(vec!["log".into()]);
        let filter = PathFilter::new(&config);

        assert!(!filter.extension_allowed(Path::new("run.LOG")));
        assert!(filter.extension_allowed(Path::new("run.rs")));
    }

    #[test]
    fn exclude_ext_wins_over_include_ext_when_both_match() {
        let mut config = base_config();
        config.include_ext = Some(vec!["rs".into()]);
        config.exclude_ext = Some(vec!["rs".into()]);
        let filter = PathFilter::new(&config);

        assert!(!filter.extension_allowed(Path::new("main.rs")));
    }

    #[test]
    fn extension_allowed_by_default_when_neither_list_set() {
        let config = base_config();
        let filter = PathFilter::new(&config);

        assert!(filter.extension_allowed(Path::new("main.rs")));
    }

    #[test]
    fn extension_allowed_treats_missing_extension_as_empty_string() {
        let mut config = base_config();
        config.include_ext = Some(vec!["rs".into()]);
        let filter = PathFilter::new(&config);

        // No extension at all: doesn't match the include list.
        assert!(!filter.extension_allowed(Path::new("Makefile")));

        let mut config_empty_include = base_config();
        config_empty_include.include_ext = Some(vec!["".into()]);
        let filter_empty = PathFilter::new(&config_empty_include);
        assert!(filter_empty.extension_allowed(Path::new("Makefile")));
    }

    // ---- allows_entry end-to-end -----------------------------------------

    #[test]
    fn allows_entry_returns_false_for_the_output_path_even_if_it_would_otherwise_be_allowed() {
        let dir = tempfile::tempdir().unwrap();
        let output_path = dir.path().join("fyai.txt");
        fs::write(&output_path, b"hello").unwrap();

        let mut config = base_config();
        config.output = output_path.clone();
        let filter = PathFilter::new(&config);

        assert!(!filter.allows_entry(&output_path, false));
    }

    #[test]
    fn allows_entry_short_circuits_true_for_allowed_directories() {
        let mut config = base_config();
        config.include_dirs = Some(vec!["src".into()]);
        let filter = PathFilter::new(&config);

        // Directories only go through the dir-allowed check, never
        // file_name_allowed/extension_allowed.
        assert!(filter.allows_entry(Path::new("project/src"), true));
    }

    #[test]
    fn allows_entry_returns_false_for_directories_excluded_by_dir_filters() {
        let mut config = base_config();
        config.exclude_dirs = Some(vec!["node_modules".into()]);
        let filter = PathFilter::new(&config);

        assert!(!filter.allows_entry(Path::new("project/node_modules"), true));
    }

    #[test]
    fn allows_entry_checks_file_filters_for_files() {
        let mut config = base_config();
        config.include_ext = Some(vec!["rs".into()]);
        let filter = PathFilter::new(&config);

        assert!(filter.allows_entry(Path::new("project/src/main.rs"), false));
        assert!(!filter.allows_entry(Path::new("project/src/main.py"), false));
    }

    #[test]
    fn allows_entry_returns_false_for_files_in_a_disallowed_directory_even_if_file_filters_match() {
        let mut config = base_config();
        config.include_dirs = Some(vec!["src".into()]);
        config.include_ext = Some(vec!["rs".into()]);
        let filter = PathFilter::new(&config);

        // File extension matches, but the directory doesn't contain `src`.
        assert!(!filter.allows_entry(Path::new("project/tests/main.rs"), false));
        // Both match: allowed.
        assert!(filter.allows_entry(Path::new("project/src/main.rs"), false));
    }

    #[test]
    fn allows_entry_allows_everything_by_default() {
        let config = base_config();
        let filter = PathFilter::new(&config);

        assert!(filter.allows_entry(Path::new("anything/at/all.rs"), false));
        assert!(filter.allows_entry(Path::new("anything/at/all"), true));
    }

    // ---- normalize_list / NormalizedFilterConfig --------------------------

    #[test]
    fn normalize_list_returns_none_when_input_is_none() {
        assert_eq!(normalize_list(&None), None);
    }

    #[test]
    fn normalize_list_lower_cases_every_item() {
        let list = Some(vec!["SRC".to_string(), "Docs".to_string()]);
        let normalized = normalize_list(&list).unwrap();

        assert!(normalized.contains("src"));
        assert!(normalized.contains("docs"));
        assert_eq!(normalized.len(), 2);
    }

    #[test]
    fn all_filter_lists_none_means_nothing_is_filtered() {
        let config = base_config();
        let filter = PathFilter::new(&config);

        assert!(filter.allows_entry(Path::new("literally/anything.xyz"), false));
        assert!(filter.allows_entry(Path::new("literally/anything"), true));
    }
}
