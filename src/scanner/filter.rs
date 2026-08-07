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
