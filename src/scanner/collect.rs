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
