use ignore::gitignore::Gitignore;
use std::fs::{self, File};
use std::io::{self, Read, Write};
use std::path::Path;
use walkdir::WalkDir;

use crate::cli::Config;

/// Checks if a path is within an ignored directory.
///
/// # Arguments
/// - `path`: The path to check.
/// - `ignored_dirs`: List of directory names to ignore.
///
/// # Returns
/// - `bool`: `true` if the path is in an ignored directory, `false` otherwise.
pub fn is_in_ignored_dir(path: &Path, ignored_dirs: &[&str]) -> bool {
    path.components().any(|comp| {
        comp.as_os_str()
            .to_str()
            .map(|name| ignored_dirs.contains(&name))
            .unwrap_or(false)
    })
}

/// Generates a string representation of the project directory structure.
///
/// # Arguments
/// - `root`: The root directory path.
/// - `gitignore`: Gitignore rules to apply.
/// - `ignored_dirs`: List of directories to ignore.
///
/// # Returns
/// - `Ok(String)`: The formatted directory structure.
/// - `Err(io::Error)`: If an error occurs during traversal.
pub fn get_directory_structure(
    root: &Path,
    gitignore: &Gitignore,
    ignored_dirs: &[&str],
) -> io::Result<String> {
    let mut structure = String::new();
    structure.push_str("=== Project Directory Structure ===\n\n");

    for entry in WalkDir::new(root).into_iter().filter_map(Result::ok) {
        let path = entry.path();
        if is_in_ignored_dir(path, ignored_dirs)
            || gitignore.matched(path, path.is_dir()).is_ignore()
        {
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
///
/// # Arguments
/// - `config`: The CLI configuration.
/// - `gitignore`: Gitignore rules to apply.
/// - `ignored_dirs`: List of directories to ignore.
/// - `dir_structure`: The directory structure string to write to the output.
///
/// # Returns
/// - `Ok(())`: On successful processing.
/// - `Err(io::Error)`: If an error occurs during file processing.
pub fn process_files(
    config: &Config,
    gitignore: &Gitignore,
    ignored_dirs: &[&str],
    dir_structure: &str,
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
            if config.test_mode {
                println!("Skipping (output file): {}", path.display());
            }
            continue;
        }

        if is_in_ignored_dir(path, ignored_dirs) {
            if config.test_mode {
                println!("Skipping (ignored directory): {}", path.display());
            }
            continue;
        }

        let is_dir = path.is_dir();
        if gitignore.matched(path, is_dir).is_ignore() {
            if config.test_mode {
                println!("Skipping (gitignore): {}", path.display());
            }
            continue;
        }

        if is_dir {
            if config.test_mode {
                println!("Skipping (directory): {}", path.display());
            }
            continue;
        }

        let metadata = fs::metadata(path)?;
        let file_size = metadata.len();

        if let Some(min) = config.min_size {
            if file_size < min {
                if config.test_mode {
                    println!(
                        "Skipping (too small): {} ({} bytes)",
                        path.display(),
                        file_size
                    );
                }
                continue;
            }
        }
        if let Some(max) = config.max_size {
            if file_size > max {
                if config.test_mode {
                    println!(
                        "Skipping (too large): {} ({} bytes)",
                        path.display(),
                        file_size
                    );
                }
                continue;
            }
        }

        let ext = path
            .extension()
            .and_then(|e| e.to_str())
            .map(|e| e.to_lowercase());
        if let Some(ref exts) = config.extensions {
            if ext.is_none() || exts.contains(&ext.unwrap()) {
                if config.test_mode {
                    println!("Skipping (excluded extension): {}", path.display());
                }
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
        } else if config.test_mode {
            println!("Skipping (not UTF-8): {}", path.display());
        }
    }

    output.flush()?;
    Ok(())
}
