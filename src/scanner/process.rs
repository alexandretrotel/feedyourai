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

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    /// A minimal baseline [`Config`], with only `directory` varying per
    /// test and size bounds overridden as needed.
    fn base_config(directory: PathBuf) -> Config {
        Config {
            directory,
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

    // ---- format_size ----

    #[test]
    fn format_size_zero_bytes() {
        assert_eq!(format_size(0), "0 B");
    }

    #[test]
    fn format_size_stays_bytes_under_1024() {
        assert_eq!(format_size(500), "500 B");
        assert_eq!(format_size(1023), "1023 B");
    }

    #[test]
    fn format_size_exactly_1024_is_one_kb() {
        assert_eq!(format_size(1024), "1.0 KB");
    }

    #[test]
    fn format_size_kb_rounding() {
        assert_eq!(format_size(1536), "1.5 KB");
    }

    #[test]
    fn format_size_just_under_1_mb_rounds_up_in_kb() {
        // 1024*1024 - 1 bytes is still < 1024 KB numerically once divided,
        // but `{:.1}` rounding pushes the printed value to "1024.0 KB"
        // rather than promoting to MB (the promotion check already ran).
        assert_eq!(format_size(1024 * 1024 - 1), "1024.0 KB");
    }

    #[test]
    fn format_size_mb_and_gb() {
        assert_eq!(format_size(1024 * 1024), "1.0 MB");
        assert_eq!(format_size(1024 * 1024 * 1024), "1.0 GB");
    }

    #[test]
    fn format_size_caps_at_gb_for_huge_values() {
        // The unit index is capped at GB (index 3, the last unit), so
        // divisions stop there no matter how large `bytes` is.
        let expected = {
            let mut size = u64::MAX as f64;
            size /= 1024.0;
            size /= 1024.0;
            size /= 1024.0;
            format!("{size:.1} GB")
        };
        let got = format_size(u64::MAX);
        assert_eq!(got, expected);
        assert!(got.ends_with(" GB"));
    }

    // ---- size_allowed ----

    #[test]
    fn size_allowed_unconstrained_when_both_bounds_none() {
        assert!(size_allowed(0, None, None));
        assert!(size_allowed(u64::MAX, None, None));
    }

    #[test]
    fn size_allowed_min_only() {
        assert!(!size_allowed(9, Some(10), None));
        assert!(size_allowed(10, Some(10), None)); // boundary, inclusive
        assert!(size_allowed(11, Some(10), None));
    }

    #[test]
    fn size_allowed_max_only() {
        assert!(size_allowed(0, None, Some(10)));
        assert!(size_allowed(10, None, Some(10))); // boundary, inclusive
        assert!(!size_allowed(11, None, Some(10)));
    }

    #[test]
    fn size_allowed_min_and_max() {
        assert!(!size_allowed(9, Some(10), Some(20)));
        assert!(size_allowed(10, Some(10), Some(20))); // lower boundary
        assert!(size_allowed(15, Some(10), Some(20)));
        assert!(size_allowed(20, Some(10), Some(20))); // upper boundary
        assert!(!size_allowed(21, Some(10), Some(20)));
    }

    // ---- write_file_block ----

    #[test]
    fn write_file_block_normal_text_ending_with_newline() {
        let mut buf: Vec<u8> = Vec::new();
        write_file_block(
            &mut buf,
            Path::new("/root"),
            Path::new("/root/src/main.rs"),
            13,
            "fn main() {}\n",
        )
        .unwrap();
        let out = String::from_utf8(buf).unwrap();
        assert_eq!(
            out,
            "\n### src/main.rs (13 B)\n\n```rust\nfn main() {}\n```\n"
        );
    }

    #[test]
    fn write_file_block_text_without_trailing_newline_gets_one_appended() {
        let mut buf: Vec<u8> = Vec::new();
        write_file_block(
            &mut buf,
            Path::new("/root"),
            Path::new("/root/file.xyz"),
            10,
            "no newline",
        )
        .unwrap();
        let out = String::from_utf8(buf).unwrap();
        assert_eq!(out, "\n### file.xyz (10 B)\n\n```\nno newline\n```\n");
    }

    #[test]
    fn write_file_block_widens_fence_when_text_contains_triple_backtick() {
        let text = "some ```code``` here\n";
        let mut buf: Vec<u8> = Vec::new();
        write_file_block(
            &mut buf,
            Path::new("/root"),
            Path::new("/root/README"),
            text.len() as u64,
            text,
        )
        .unwrap();
        let out = String::from_utf8(buf).unwrap();
        let expected_size = format_size(text.len() as u64);
        assert_eq!(
            out,
            format!("\n### README ({expected_size})\n\n````\n{text}````\n")
        );
    }

    #[test]
    fn write_file_block_falls_back_to_full_path_when_root_is_not_a_prefix() {
        let mut buf: Vec<u8> = Vec::new();
        write_file_block(
            &mut buf,
            Path::new("/unrelated"),
            Path::new("/other/tree/file.txt"),
            3,
            "hi\n",
        )
        .unwrap();
        let out = String::from_utf8(buf).unwrap();
        assert_eq!(out, "\n### /other/tree/file.txt (3 B)\n\n```\nhi\n```\n");
    }

    // ---- read_file_block ----

    #[test]
    fn read_file_block_returns_contents_within_bounds() {
        let dir = tempfile::tempdir().unwrap();
        let file_path = dir.path().join("a.txt");
        fs::write(&file_path, "hello world").unwrap();

        let entry = Entry {
            path: file_path.clone(),
            depth: 1,
            is_dir: false,
            size: Some(11),
        };
        let config = base_config(dir.path().to_path_buf());

        let result = read_file_block(&entry, &config);
        assert_eq!(result, Some((file_path, 11, "hello world".to_string())));
    }

    #[test]
    fn read_file_block_none_when_below_min_size() {
        let dir = tempfile::tempdir().unwrap();
        let file_path = dir.path().join("a.txt");
        fs::write(&file_path, "hello world").unwrap();

        let entry = Entry {
            path: file_path,
            depth: 1,
            is_dir: false,
            size: Some(11),
        };
        let mut config = base_config(dir.path().to_path_buf());
        config.min_size = Some(100);

        assert_eq!(read_file_block(&entry, &config), None);
    }

    #[test]
    fn read_file_block_none_when_above_max_size() {
        let dir = tempfile::tempdir().unwrap();
        let file_path = dir.path().join("a.txt");
        fs::write(&file_path, "hello world").unwrap();

        let entry = Entry {
            path: file_path,
            depth: 1,
            is_dir: false,
            size: Some(11),
        };
        let mut config = base_config(dir.path().to_path_buf());
        config.max_size = Some(5);

        assert_eq!(read_file_block(&entry, &config), None);
    }

    #[test]
    fn read_file_block_none_for_invalid_utf8() {
        let dir = tempfile::tempdir().unwrap();
        let file_path = dir.path().join("bin.dat");
        fs::write(&file_path, [0xFFu8, 0xFE]).unwrap();

        let entry = Entry {
            path: file_path,
            depth: 1,
            is_dir: false,
            size: Some(2),
        };
        let config = base_config(dir.path().to_path_buf());

        assert_eq!(read_file_block(&entry, &config), None);
    }

    #[test]
    fn read_file_block_none_for_nonexistent_path() {
        let dir = tempfile::tempdir().unwrap();
        let file_path = dir.path().join("missing.txt");

        let entry = Entry {
            path: file_path,
            depth: 1,
            is_dir: false,
            size: Some(0),
        };
        let config = base_config(dir.path().to_path_buf());

        assert_eq!(read_file_block(&entry, &config), None);
    }

    // ---- write_file_contents ----

    #[test]
    fn write_file_contents_preserves_input_order_and_filters_bad_entries() {
        let dir = tempfile::tempdir().unwrap();

        let sub_dir = dir.path().join("sub");
        fs::create_dir(&sub_dir).unwrap();

        let good1 = dir.path().join("good1.txt");
        fs::write(&good1, "first file\n").unwrap();

        let big = dir.path().join("big.txt");
        let big_contents = "x".repeat(100);
        fs::write(&big, &big_contents).unwrap();

        let bin = dir.path().join("bin.dat");
        fs::write(&bin, [0xFFu8, 0xFE]).unwrap();

        let good2 = dir.path().join("good2.txt");
        fs::write(&good2, "second file\n").unwrap();

        let entries = vec![
            Entry {
                path: sub_dir,
                depth: 1,
                is_dir: true,
                size: None,
            },
            Entry {
                path: good1,
                depth: 1,
                is_dir: false,
                size: Some(11),
            },
            Entry {
                path: big,
                depth: 1,
                is_dir: false,
                size: Some(100),
            },
            Entry {
                path: bin,
                depth: 1,
                is_dir: false,
                size: Some(2),
            },
            Entry {
                path: good2,
                depth: 1,
                is_dir: false,
                size: Some(12),
            },
        ];

        let mut config = base_config(dir.path().to_path_buf());
        config.max_size = Some(50);

        let mut output: Vec<u8> = Vec::new();
        write_file_contents(&entries, &config, &mut output).unwrap();
        let text = String::from_utf8(output).unwrap();

        assert!(text.contains("### good1.txt (11 B)"));
        assert!(text.contains("### good2.txt (12 B)"));
        assert!(text.contains("first file\n"));
        assert!(text.contains("second file\n"));
        assert!(!text.contains("big.txt"));
        assert!(!text.contains("bin.dat"));

        let pos1 = text.find("good1.txt").expect("good1 present");
        let pos2 = text.find("good2.txt").expect("good2 present");
        assert!(pos1 < pos2, "entries must be written in input order");
    }

    #[test]
    fn write_file_contents_empty_entries_writes_nothing() {
        let dir = tempfile::tempdir().unwrap();
        let config = base_config(dir.path().to_path_buf());
        let mut output: Vec<u8> = Vec::new();
        write_file_contents(&[], &config, &mut output).unwrap();
        assert!(output.is_empty());
    }
}
