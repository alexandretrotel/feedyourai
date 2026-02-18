use std::collections::HashSet;
use std::fs;
use std::path::{Path, PathBuf};

use crate::config::Config;

pub struct PathFilter<'a> {
    config: &'a Config,
    ignored_dirs: &'a [&'a str],
    canonical_output_path: Option<PathBuf>,
    normalized_filters: NormalizedFilterConfig,
}

impl<'a> PathFilter<'a> {
    pub fn new(config: &'a Config, ignored_dirs: &'a [&'a str]) -> Self {
        let canonical_output_path = fs::canonicalize(&config.output).ok();
        let normalized_filters = NormalizedFilterConfig::new(config);

        Self {
            config,
            ignored_dirs,
            canonical_output_path,
            normalized_filters,
        }
    }

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

    fn is_output_path(&self, path: &Path) -> bool {
        if let Some(canonical_output_path) = &self.canonical_output_path
            && let Ok(path_canon) = fs::canonicalize(path)
        {
            return path_canon == *canonical_output_path;
        }
        path == self.config.output
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
        if any_component_matches_list(path, self.ignored_dirs) {
            return true;
        }
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
        let file_name = path
            .file_name()
            .and_then(|f| f.to_str())
            .unwrap_or_default()
            .to_lowercase();

        if let Some(excludes) = &self.normalized_filters.exclude_files
            && excludes.contains(&file_name)
        {
            return false;
        }

        match &self.normalized_filters.include_files {
            Some(includes) => includes.contains(&file_name),
            None => true,
        }
    }

    fn extension_allowed(&self, path: &Path) -> bool {
        let ext = path
            .extension()
            .and_then(|e| e.to_str())
            .unwrap_or("")
            .to_lowercase();

        if let Some(excludes) = &self.normalized_filters.exclude_ext
            && excludes.contains(&ext)
        {
            return false;
        }

        match &self.normalized_filters.include_ext {
            Some(includes) => includes.contains(&ext),
            None => true,
        }
    }
}

struct NormalizedFilterConfig {
    include_dirs: Option<HashSet<String>>,
    exclude_dirs: Option<HashSet<String>>,
    include_files: Option<HashSet<String>>,
    exclude_files: Option<HashSet<String>>,
    include_ext: Option<HashSet<String>>,
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

fn normalize_list(list: &Option<Vec<String>>) -> Option<HashSet<String>> {
    list.as_ref()
        .map(|items| items.iter().map(|item| item.to_lowercase()).collect())
}

fn any_component_in_set(path: &Path, set: &HashSet<String>) -> bool {
    path.components().any(|component| {
        component
            .as_os_str()
            .to_str()
            .map(|name| set.contains(&name.to_lowercase()))
            .unwrap_or(false)
    })
}

fn any_component_matches_list(path: &Path, list: &[&str]) -> bool {
    path.components().any(|component| {
        component
            .as_os_str()
            .to_str()
            .map(|name| {
                let name_lower = name.to_lowercase();
                list.iter()
                    .any(|ignored| ignored.eq_ignore_ascii_case(&name_lower))
            })
            .unwrap_or(false)
    })
}
