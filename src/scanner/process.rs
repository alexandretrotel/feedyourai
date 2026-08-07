//! Writes the directory tree plus the contents of every file that passes
//! the configured filters into the output file.

use std::fs::{self, File};
use std::io::{self, Read, Write};

use crate::config::Config;

use super::filter::PathFilter;
use super::utils::size_allowed;
use super::walker::build_walker;

/// Writes `dir_structure` followed by the contents of every matching file
/// under `config.directory` to `config.output`.
///
/// Files that fail UTF-8 decoding are silently skipped (binary files aren't
/// meaningful to include in an AI-facing text dump); everything else is
/// filtered by the path filter and size bounds before being appended, each
/// prefixed with a `- File: name (size bytes)` header.
pub fn process_files(
    config: &Config,
    dir_structure: &str,
    ignored_dirs: &[&str],
) -> io::Result<()> {
    let mut output = File::create(&config.output)?;
    write!(output, "{}", dir_structure)?;

    let filter = PathFilter::new(config, ignored_dirs);
    let walker = build_walker(&config.directory, ignored_dirs, config)?;
    for entry in walker {
        let entry = entry.map_err(io::Error::other)?;
        let path = entry.path();
        let is_dir = entry
            .file_type()
            .map(|file_type| file_type.is_dir())
            .unwrap_or_else(|| path.is_dir());

        if !filter.allows_entry(path, is_dir) {
            continue;
        }
        if is_dir {
            continue;
        }

        let metadata = fs::metadata(path)?;
        let file_size = metadata.len();

        if !size_allowed(file_size, config.min_size, config.max_size) {
            continue;
        }

        let mut file = File::open(path)?;
        let mut contents = Vec::new();
        file.read_to_end(&mut contents)?;

        if let Ok(text) = String::from_utf8(contents) {
            let file_name = path
                .file_name()
                .and_then(|f| f.to_str())
                .unwrap_or_default();
            writeln!(output, "\n- File: {} ({} bytes)\n", file_name, file_size)?;
            write!(output, "{}", text)?;
        }
    }

    output.flush()?;
    Ok(())
}
