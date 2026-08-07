//! Orchestrates a single combine run: one parallel walk over
//! `config.directory` builds the directory tree and, unless
//! `config.tree_only`, every matching file's contents, written to
//! `config.output` through a single buffered writer.

use std::fs::File;
use std::io::{self, BufWriter, Write};

use crate::config::Config;

use super::collect::collect_entries;
use super::process::write_file_contents;
use super::tree::render_tree;

/// Writes `config.directory`'s (filtered) tree, and optionally its files'
/// contents, to `config.output`.
///
/// Returns the total size in bytes of every file entry the walk collected
/// (regardless of `config.tree_only` or the `min_size`/`max_size` content
/// filters, which only gate whether a file's *contents* are written).
pub fn scan(config: &Config) -> io::Result<u64> {
    let mut output = BufWriter::new(File::create(&config.output)?);

    if config.directory.read_dir()?.count() == 0 {
        write!(output, "- Tree Structure\n\nThe directory is empty.\n\n")?;
        output.flush()?;
        return Ok(0);
    }

    let entries = collect_entries(config)?;
    let total_size: u64 = entries.iter().filter_map(|entry| entry.size).sum();

    write!(
        output,
        "{}",
        render_tree(&entries, &config.directory, config.human)
    )?;

    if !config.tree_only {
        write_file_contents(&entries, config, &mut output)?;
    }

    output.flush()?;
    Ok(total_size)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::path::PathBuf;

    /// A baseline [`Config`] with every filter at its "do nothing special"
    /// default. Individual tests override only the fields they care about.
    fn base_config(directory: &std::path::Path, output: PathBuf) -> Config {
        Config {
            directory: directory.to_path_buf(),
            output,
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
    fn scan_writes_empty_message_for_an_empty_directory() {
        let scan_dir = tempfile::tempdir().expect("tempdir");
        let output_dir = tempfile::tempdir().expect("tempdir");
        let output_path = output_dir.path().join("fyai.txt");

        let config = base_config(scan_dir.path(), output_path.clone());
        scan(&config).expect("scan should succeed");

        let contents = fs::read_to_string(&output_path).expect("read output");
        assert!(contents.contains("- Tree Structure"));
        assert!(contents.contains("The directory is empty."));
    }

    #[test]
    fn scan_writes_tree_and_file_contents_by_default() {
        let scan_dir = tempfile::tempdir().expect("tempdir");
        let output_dir = tempfile::tempdir().expect("tempdir");
        let output_path = output_dir.path().join("fyai.txt");

        fs::write(scan_dir.path().join("hello.txt"), "Hello World").expect("write");
        fs::create_dir_all(scan_dir.path().join("sub")).expect("create_dir_all");
        fs::write(
            scan_dir.path().join("sub").join("nested.rs"),
            "fn main() {}",
        )
        .expect("write");

        let config = base_config(scan_dir.path(), output_path.clone());
        scan(&config).expect("scan should succeed");

        let contents = fs::read_to_string(&output_path).expect("read output");
        assert!(contents.contains("- Tree Structure"));
        assert!(contents.contains("hello.txt"));
        assert!(contents.contains("nested.rs"));
        assert!(contents.contains("Hello World"));
        assert!(contents.contains("fn main() {}"));
    }

    #[test]
    fn scan_returns_zero_total_size_for_an_empty_directory() {
        let scan_dir = tempfile::tempdir().expect("tempdir");
        let output_dir = tempfile::tempdir().expect("tempdir");
        let output_path = output_dir.path().join("fyai.txt");

        let config = base_config(scan_dir.path(), output_path);
        let total_size = scan(&config).expect("scan should succeed");

        assert_eq!(total_size, 0);
    }

    #[test]
    fn scan_returns_total_size_of_walked_files() {
        let scan_dir = tempfile::tempdir().expect("tempdir");
        let output_dir = tempfile::tempdir().expect("tempdir");
        let output_path = output_dir.path().join("fyai.txt");

        fs::write(scan_dir.path().join("hello.txt"), "Hello World").expect("write"); // 11 bytes
        fs::create_dir_all(scan_dir.path().join("sub")).expect("create_dir_all");
        fs::write(
            scan_dir.path().join("sub").join("nested.rs"),
            "fn main() {}",
        )
        .expect("write"); // 12 bytes

        let config = base_config(scan_dir.path(), output_path);
        let total_size = scan(&config).expect("scan should succeed");

        assert_eq!(total_size, 23);
    }

    #[test]
    fn scan_total_size_is_unaffected_by_tree_only() {
        let scan_dir = tempfile::tempdir().expect("tempdir");
        let output_dir = tempfile::tempdir().expect("tempdir");
        let output_path = output_dir.path().join("fyai.txt");

        fs::write(scan_dir.path().join("hello.txt"), "Hello World").expect("write"); // 11 bytes

        let mut config = base_config(scan_dir.path(), output_path);
        config.tree_only = true;
        let total_size = scan(&config).expect("scan should succeed");

        assert_eq!(total_size, 11);
    }

    #[test]
    fn scan_tree_only_excludes_file_contents() {
        let scan_dir = tempfile::tempdir().expect("tempdir");
        let output_dir = tempfile::tempdir().expect("tempdir");
        let output_path = output_dir.path().join("fyai.txt");

        fs::write(scan_dir.path().join("hello.txt"), "Hello World").expect("write");

        let mut config = base_config(scan_dir.path(), output_path.clone());
        config.tree_only = true;
        scan(&config).expect("scan should succeed");

        let contents = fs::read_to_string(&output_path).expect("read output");
        assert!(contents.contains("- Tree Structure"));
        assert!(contents.contains("hello.txt"));
        assert!(!contents.contains("Hello World"));
    }

    #[test]
    fn scan_returns_err_when_directory_does_not_exist() {
        let output_dir = tempfile::tempdir().expect("tempdir");
        let output_path = output_dir.path().join("fyai.txt");
        let missing_dir = output_dir.path().join("does-not-exist");

        let config = base_config(&missing_dir, output_path);
        let result = scan(&config);

        assert!(result.is_err());
    }
}
