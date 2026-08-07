//! Writes the directory tree plus the contents of every file that passes
//! the configured filters into the output file.

use std::fs::{self, File};
use std::io::{self, Read, Write};
use std::path::Path;

use crate::config::Config;

use super::filter::PathFilter;
use super::lang::fence_language;
use super::utils::{format_size, size_allowed};
use super::walker::build_walker;

/// Writes `dir_structure` followed by the contents of every matching file
/// under `config.directory` to `config.output`.
///
/// Files that fail UTF-8 decoding are silently skipped (binary files aren't
/// meaningful to include in an AI-facing text dump); everything else is
/// filtered by the path filter and size bounds before being appended as a
/// `### path (size)` heading followed by a language-tagged, fenced code
/// block (see [`write_file_block`]).
pub fn process_files(
    config: &Config,
    dir_structure: &str,
    ignored_dirs: &[&str],
) -> io::Result<()> {
    let mut output = File::create(&config.output)?;
    write!(output, "{}", dir_structure)?;

    let filter = PathFilter::new(config, ignored_dirs);
    let walker = build_walker(config, ignored_dirs)?;
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

        let file_size = fs::metadata(path)?.len();
        if !size_allowed(file_size, config.min_size, config.max_size) {
            continue;
        }

        let mut file = File::open(path)?;
        let mut contents = Vec::new();
        file.read_to_end(&mut contents)?;

        if let Ok(text) = String::from_utf8(contents) {
            write_file_block(&mut output, &config.directory, path, file_size, &text)?;
        }
    }

    output.flush()?;
    Ok(())
}

/// Appends one file's heading and fenced code block to `output`.
///
/// The fence widens from ``` to ```` when `text` itself contains a triple
/// backtick, so the block's end is never ambiguous. The language tag is
/// inferred from `path`'s extension via [`fence_language`], falling back to
/// a plain, untagged fence when unrecognized.
fn write_file_block(
    output: &mut File,
    root: &Path,
    path: &Path,
    file_size: u64,
    text: &str,
) -> io::Result<()> {
    let display_path = path.strip_prefix(root).unwrap_or(path).display();
    let size = format_size(file_size);
    let lang = path
        .extension()
        .and_then(|ext| ext.to_str())
        .map(|ext| fence_language(&ext.to_lowercase()))
        .unwrap_or_default();
    let fence = if text.contains("```") { "````" } else { "```" };

    writeln!(output, "\n### {display_path} ({size})\n")?;
    writeln!(output, "{fence}{lang}")?;
    write!(output, "{text}")?;
    if !text.ends_with('\n') {
        writeln!(output)?;
    }
    writeln!(output, "{fence}")?;

    Ok(())
}
