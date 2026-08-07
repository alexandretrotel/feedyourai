//! Writes the contents of every collected file entry that passes the size
//! filter to the output writer.
//!
//! Files that fail UTF-8 decoding are silently skipped (binary files aren't
//! meaningful to include in an LLM-facing text dump); everything else is
//! appended as a `### path (size)` heading followed by a language-tagged,
//! fenced code block (see `write_file_block`).

use std::fs;
use std::io::{self, Write};
use std::path::{Path, PathBuf};

use rayon::prelude::*;

use crate::config::Config;

use super::collect::Entry;
use super::lang::fence_language;

/// Reads and decodes every file `entry` in parallel (I/O and UTF-8
/// validation are the expensive parts, and are independent per file), then
/// writes the resulting blocks to `output` in the original, deterministic
/// order.
pub(crate) fn write_file_contents<W: Write>(
    entries: &[Entry],
    config: &Config,
    output: &mut W,
) -> io::Result<()> {
    let blocks: Vec<(PathBuf, u64, String)> = entries
        .par_iter()
        .filter(|entry| !entry.is_dir)
        .filter_map(|entry| read_file_block(entry, config))
        .collect();

    for (path, size, text) in &blocks {
        write_file_block(output, &config.directory, path, *size, text)?;
    }

    Ok(())
}

/// Reads `entry`'s contents if its size passes `config`'s bounds and it
/// decodes as UTF-8; returns `None` otherwise (binary/oversized/undersized
/// files are silently skipped, same as before).
fn read_file_block(entry: &Entry, config: &Config) -> Option<(PathBuf, u64, String)> {
    let size = entry.size.unwrap_or(0);
    if !size_allowed(size, config.min_size, config.max_size) {
        return None;
    }

    let contents = fs::read(&entry.path).ok()?;
    simdutf8::basic::from_utf8(&contents).ok()?;
    // SAFETY: `contents` was just validated as well-formed UTF-8 above, and
    // hasn't been touched since.
    let text = unsafe { String::from_utf8_unchecked(contents) };

    Some((entry.path.clone(), size, text))
}

/// Returns true if `size` falls within the inclusive `[min, max]` bounds,
/// treating a missing bound as unconstrained.
fn size_allowed(size: u64, min: Option<u64>, max: Option<u64>) -> bool {
    if let Some(min) = min
        && size < min
    {
        return false;
    }
    if let Some(max) = max
        && size > max
    {
        return false;
    }
    true
}

/// Formats `bytes` as a human-readable size (`"512 B"`, `"1.2 KB"`, `"3.4
/// MB"`, ...), using 1024 as the unit step.
fn format_size(bytes: u64) -> String {
    const UNITS: [&str; 4] = ["B", "KB", "MB", "GB"];

    let mut size = bytes as f64;
    let mut unit = 0;
    while size >= 1024.0 && unit < UNITS.len() - 1 {
        size /= 1024.0;
        unit += 1;
    }

    if unit == 0 {
        format!("{bytes} B")
    } else {
        format!("{size:.1} {}", UNITS[unit])
    }
}

/// Appends one file's heading and fenced code block to `output`.
///
/// The fence widens from ``` to ```` when `text` itself contains a triple
/// backtick, so the block's end is never ambiguous. The language tag is
/// inferred from `path`'s extension via [`fence_language`], falling back to
/// a plain, untagged fence when unrecognized.
fn write_file_block<W: Write>(
    output: &mut W,
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
