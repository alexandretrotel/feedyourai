use crate::cli::Config;
use ignore::gitignore::Gitignore;
use std::fs::{self, File};
use std::io::{self, Read, Write};
use std::path::Path;
use walkdir::WalkDir;

/// Checks if a path is within an ignored directory, including user-specified excluded directories.
pub fn is_in_ignored_dir(
    path: &Path,
    ignored_dirs: &[&str],
    exclude_dirs: &Option<Vec<String>>,
) -> bool {
    path.components().any(|comp| {
        comp.as_os_str()
            .to_str()
            .map(|name| {
                let name_lower = name.to_lowercase();
                ignored_dirs
                    .iter()
                    .any(|&ignored| ignored.eq_ignore_ascii_case(&name_lower))
                    || exclude_dirs.as_ref().map_or(false, |dirs| {
                        dirs.iter().any(|dir| dir.eq_ignore_ascii_case(&name_lower))
                    })
            })
            .unwrap_or(false)
    })
}

/// Generates a string representation of the project directory structure.
pub fn get_directory_structure(
    root: &Path,
    gitignore: &Gitignore,
    ignored_dirs: &[&str],
    exclude_dirs: &Option<Vec<String>>,
) -> io::Result<String> {
    let mut structure = String::new();
    structure.push_str("=== Project Directory Structure ===\n\n");

    // Check if the root directory is empty
    if root.read_dir()?.count() == 0 {
        structure.push_str("The directory is empty.\n");
        return Ok(structure);
    }

    for entry in WalkDir::new(root).into_iter().filter_map(Result::ok) {
        let path = entry.path();
        let is_dir = path.is_dir();
        if should_skip_path(path, is_dir, gitignore, ignored_dirs, exclude_dirs) {
            continue;
        }

        let depth = entry.depth();
        let indent = "  ".repeat(depth);
        if let Some(name) = path.file_name() {
            let marker = if path.is_dir() { "/" } else { "" };
            structure.push_str(&format!("{}{}{}\n", indent, name.to_string_lossy(), marker));
        }
    }

    structure.push_str("\n");
    Ok(structure)
}

/// Processes files in the input directory and combines them into the output file.
pub fn process_files(
    config: &Config,
    gitignore: &Gitignore,
    dir_structure: &str,
    ignored_dirs: &[&str],
) -> io::Result<()> {
    let mut output = File::create(&config.output)?;
    write!(output, "{}", dir_structure)?;

    println!("Processing files in: {:?}", config.directory);

    for entry in WalkDir::new(&config.directory)
        .into_iter()
        .filter_map(Result::ok)
    {
        let path = entry.path();
        if path == config.output {
            continue;
        }

        let is_dir = path.is_dir();

        if should_skip_path(path, is_dir, gitignore, ignored_dirs, &config.exclude_dirs) {
            continue;
        }

        if is_dir {
            continue; // Skip directories
        }

        let metadata = fs::metadata(path)?;
        let file_size = metadata.len();

        if let Some(min) = config.min_size {
            if file_size < min {
                continue;
            }
        }
        if let Some(max) = config.max_size {
            if file_size > max {
                continue;
            }
        }

        let ext = path
            .extension()
            .and_then(|e| e.to_str())
            .map(|e| e.to_lowercase());
        if let Some(ref exts) = config.extensions {
            if ext.is_none() || exts.contains(&ext.unwrap()) {
                continue;
            }
        }

        println!("Processing: {} ({} bytes)", path.display(), file_size);

        let mut file = File::open(path)?;
        let mut contents = Vec::new();
        file.read_to_end(&mut contents)?;
        let file_name = path.file_name().unwrap_or_default();

        if let Ok(text) = String::from_utf8(contents) {
            writeln!(
                output,
                "\n=== File: {} ({} bytes) ===\n",
                file_name.to_string_lossy(),
                file_size
            )?;
            write!(output, "{}", text)?;
        }
    }

    output.flush()?;
    Ok(())
}

/// Determines if a path should be skipped during file processing.
///
/// This function checks if a path should be excluded from processing based on:
/// 1. User-specified ignored directories (case-insensitive matching)
/// 2. Custom exclude directories provided via CLI configuration
/// 3. Gitignore rules that apply to the path
pub fn should_skip_path(
    path: &Path,
    is_dir: bool,
    gitignore: &Gitignore,
    ignored_dirs: &[&str],
    exclude_dirs: &Option<Vec<String>>,
) -> bool {
    is_in_ignored_dir(path, ignored_dirs, exclude_dirs)
        || gitignore.matched(path, is_dir).is_ignore()
}
