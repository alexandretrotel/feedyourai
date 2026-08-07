//! A single parallel walk that collects every filtered entry once, shared
//! by tree rendering and file-content writing so a run stats and filters
//! the directory tree exactly one time, no matter how many outputs it
//! produces from it.

use std::io;
use std::path::PathBuf;
use std::sync::mpsc;

use ignore::WalkState;

use crate::config::Config;

use super::filter::PathFilter;
use super::walker::build_walker;

/// One filtered walk entry: enough for both tree rendering and, for files,
/// reading contents.
pub(crate) struct Entry {
    /// Absolute path of the entry.
    pub path: PathBuf,
    /// Number of path components between this entry and `config.directory`
    /// (the walk root is depth 0; its direct children are depth 1, etc.).
    pub depth: usize,
    /// Whether this entry is a directory rather than a file.
    pub is_dir: bool,
    /// File size in bytes, already `stat`'d during the walk; `None` for
    /// directories.
    pub size: Option<u64>,
}

/// Walks `config.directory` in parallel, keeping only entries [`PathFilter`]
/// allows, then sorts the result back into a deterministic preorder.
///
/// [`Path`](std::path::Path)'s `Ord` compares path components rather than
/// raw bytes, so sorting by path exactly reconstructs the preorder a
/// sequential depth-first walk would have produced (a directory's own path
/// is always ordered immediately before all of its descendants' paths, and
/// before any sibling's), which is what the tree renderer's depth-based
/// nesting logic assumes.
pub(crate) fn collect_entries(config: &Config) -> io::Result<Vec<Entry>> {
    let filter = PathFilter::new(config);
    let walker = build_walker(config)?;
    let (tx, rx) = mpsc::channel::<Entry>();

    walker.run(|| {
        let tx = tx.clone();
        let filter = &filter;
        Box::new(move |result| {
            let Ok(entry) = result else {
                return WalkState::Continue;
            };
            let depth = entry.depth();
            if depth == 0 {
                return WalkState::Continue;
            }

            let path = entry.path();
            let is_dir = entry
                .file_type()
                .map(|file_type| file_type.is_dir())
                .unwrap_or_else(|| path.is_dir());

            if !filter.allows_entry(path, is_dir) {
                return WalkState::Continue;
            }

            let size = (!is_dir).then(|| entry.metadata().map(|m| m.len()).unwrap_or(0));
            let _ = tx.send(Entry {
                path: path.to_path_buf(),
                depth,
                is_dir,
                size,
            });
            WalkState::Continue
        })
    });
    drop(tx);

    let mut entries: Vec<Entry> = rx.into_iter().collect();
    entries.sort_by(|a, b| a.path.cmp(&b.path));
    Ok(entries)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

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

    #[test]
    fn collect_entries_returns_empty_vec_for_an_empty_directory() {
        let dir = tempfile::tempdir().expect("tempdir");
        let config = base_config(dir.path());

        let entries = collect_entries(&config).expect("collect_entries");
        assert!(entries.is_empty());
    }

    #[test]
    fn collect_entries_returns_sorted_preorder_entries_with_depth_and_size() {
        let dir = tempfile::tempdir().expect("tempdir");
        fs::write(dir.path().join("a.txt"), b"aaaaa").expect("write"); // 5 bytes
        fs::create_dir_all(dir.path().join("sub")).expect("create_dir_all");
        fs::write(dir.path().join("sub").join("b.txt"), b"bbb").expect("write"); // 3 bytes

        let config = base_config(dir.path());
        let entries = collect_entries(&config).expect("collect_entries");

        let paths: Vec<PathBuf> = entries.iter().map(|e| e.path.clone()).collect();
        let mut expected = vec![
            dir.path().join("a.txt"),
            dir.path().join("sub"),
            dir.path().join("sub").join("b.txt"),
        ];
        expected.sort();
        assert_eq!(paths, expected);

        let by_path = |name: &PathBuf| entries.iter().find(|e| &e.path == name).unwrap();

        let a = by_path(&dir.path().join("a.txt"));
        assert_eq!(a.depth, 1);
        assert!(!a.is_dir);
        assert_eq!(a.size, Some(5));

        let sub = by_path(&dir.path().join("sub"));
        assert_eq!(sub.depth, 1);
        assert!(sub.is_dir);
        assert_eq!(sub.size, None);

        let b = by_path(&dir.path().join("sub").join("b.txt"));
        assert_eq!(b.depth, 2);
        assert!(!b.is_dir);
        assert_eq!(b.size, Some(3));
    }

    #[test]
    fn collect_entries_excludes_hidden_files_when_hidden_is_true() {
        let dir = tempfile::tempdir().expect("tempdir");
        fs::write(dir.path().join(".hidden_file"), b"hidden").expect("write");
        fs::write(dir.path().join("normal_file"), b"normal").expect("write");

        let mut config = base_config(dir.path());
        config.hidden = true;
        let entries = collect_entries(&config).expect("collect_entries");
        let paths: Vec<PathBuf> = entries.iter().map(|e| e.path.clone()).collect();

        assert!(!paths.contains(&dir.path().join(".hidden_file")));
        assert!(paths.contains(&dir.path().join("normal_file")));
    }

    #[test]
    fn collect_entries_includes_hidden_files_when_hidden_is_false() {
        let dir = tempfile::tempdir().expect("tempdir");
        fs::write(dir.path().join(".hidden_file"), b"hidden").expect("write");
        fs::write(dir.path().join("normal_file"), b"normal").expect("write");

        let mut config = base_config(dir.path());
        config.hidden = false;
        let entries = collect_entries(&config).expect("collect_entries");
        let paths: Vec<PathBuf> = entries.iter().map(|e| e.path.clone()).collect();

        assert!(paths.contains(&dir.path().join(".hidden_file")));
        assert!(paths.contains(&dir.path().join("normal_file")));
    }

    #[test]
    fn collect_entries_excludes_the_output_file_when_it_lives_inside_the_scanned_directory() {
        let dir = tempfile::tempdir().expect("tempdir");
        let output_path = dir.path().join("fyai.txt");
        fs::write(&output_path, b"this is the output file").expect("write");
        fs::write(dir.path().join("kept.txt"), b"keep me").expect("write");

        let mut config = base_config(dir.path());
        config.output = output_path.clone();
        let entries = collect_entries(&config).expect("collect_entries");
        let paths: Vec<PathBuf> = entries.iter().map(|e| e.path.clone()).collect();

        assert!(!paths.contains(&output_path));
        assert!(paths.contains(&dir.path().join("kept.txt")));
    }
}
