use ignore::gitignore::Gitignore;
use std::fs::{self, File};
use std::io::{self, Read, Write};
use std::path::Path;
use walkdir::WalkDir;

use crate::config::Config;

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
                    || exclude_dirs.as_ref().is_some_and(|dirs| {
                        dirs.iter().any(|dir| dir.eq_ignore_ascii_case(&name_lower))
                    })
            })
            .unwrap_or(false)
    })
}

/// Checks if a path is within an included directory, if specified.
fn is_in_included_dir(path: &Path, include_dirs: &Option<Vec<String>>) -> bool {
    if let Some(dirs) = include_dirs {
        for comp in path.components() {
            if let Some(name) = comp.as_os_str().to_str()
                && dirs
                    .iter()
                    .any(|dir| dir.eq_ignore_ascii_case(&name.to_lowercase()))
            {
                return true;
            }
        }
        false
    } else {
        true // If not specified, include all
    }
}

/// Checks if a file name is included/excluded based on the provided lists.
fn is_file_included_excluded(
    file_name: &str,
    include_files: &Option<Vec<String>>,
    exclude_files: &Option<Vec<String>>,
) -> bool {
    if let Some(excludes) = exclude_files
        && excludes.iter().any(|f| f.eq_ignore_ascii_case(file_name))
    {
        return false;
    }
    if let Some(includes) = include_files {
        includes.iter().any(|f| f.eq_ignore_ascii_case(file_name))
    } else {
        true
    }
}

/// Checks if a file extension is included/excluded based on the provided lists.
fn is_ext_included_excluded(
    ext: Option<&str>,
    include_ext: &Option<Vec<String>>,
    exclude_ext: &Option<Vec<String>>,
) -> bool {
    let ext = ext.unwrap_or("").to_lowercase();
    if let Some(excludes) = exclude_ext
        && excludes.iter().any(|e| e == &ext)
    {
        return false;
    }
    if let Some(includes) = include_ext {
        includes.iter().any(|e| e == &ext)
    } else {
        true
    }
}

/// Generates a string representation of the project directory structure.
pub fn get_directory_structure(
    root: &Path,
    gitignore: &Gitignore,
    ignored_dirs: &[&str],
    config: &Config,
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

        if should_skip_path_advanced(path, is_dir, gitignore, ignored_dirs, config) {
            continue;
        }

        let depth = entry.depth();
        let indent = "  ".repeat(depth);
        if let Some(name) = path.file_name() {
            let marker = if path.is_dir() { "/" } else { "" };
            structure.push_str(&format!("{}{}{}\n", indent, name.to_string_lossy(), marker));
        }
    }

    structure.push('\n');
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

        if should_skip_path_advanced(path, is_dir, gitignore, ignored_dirs, config) {
            continue;
        }

        if is_dir {
            continue; // Skip directories
        }

        let metadata = fs::metadata(path)?;
        let file_size = metadata.len();

        if let Some(min) = config.min_size
            && file_size < min
        {
            continue;
        }
        if let Some(max) = config.max_size
            && file_size > max
        {
            continue;
        }

        let ext = path.extension().and_then(|e| e.to_str());

        let file_name = path
            .file_name()
            .and_then(|f| f.to_str())
            .unwrap_or_default();
        let file_name_lower = file_name.to_lowercase();

        // Extension filtering
        if !is_ext_included_excluded(ext, &config.include_ext, &config.exclude_ext) {
            continue;
        }

        // File name filtering
        if !is_file_included_excluded(
            &file_name_lower,
            &config.include_files,
            &config.exclude_files,
        ) {
            continue;
        }

        println!("Processing: {} ({} bytes)", path.display(), file_size);

        let mut file = File::open(path)?;
        let mut contents = Vec::new();
        file.read_to_end(&mut contents)?;

        if let Ok(text) = String::from_utf8(contents) {
            writeln!(
                output,
                "\n=== File: {} ({} bytes) ===\n",
                file_name, file_size
            )?;
            write!(output, "{}", text)?;
        }
    }

    output.flush()?;
    Ok(())
}

/// Determines if a path should be skipped during file processing, using advanced config.
///
/// This function checks if a path should be excluded from processing based on:
/// 1. User-specified ignored directories (case-insensitive matching)
/// 2. Custom exclude directories provided via CLI configuration
/// 3. Gitignore rules that apply to the path
pub fn should_skip_path_advanced(
    path: &Path,
    is_dir: bool,
    gitignore: &Gitignore,
    ignored_dirs: &[&str],
    config: &Config,
) -> bool {
    // Directory filtering
    if !is_in_included_dir(path, &config.include_dirs) {
        return true;
    }
    if is_in_ignored_dir(path, ignored_dirs, &config.exclude_dirs) {
        return true;
    }
    // .gitignore (only if respect_gitignore is true)
    if config.respect_gitignore && gitignore.matched(path, is_dir).is_ignore() {
        return true;
    }
    // File filtering (only for files)
    if !is_dir {
        let file_name = path
            .file_name()
            .and_then(|f| f.to_str())
            .unwrap_or_default()
            .to_lowercase();
        if let Some(excludes) = &config.exclude_files
            && excludes.iter().any(|f| f.eq_ignore_ascii_case(&file_name))
        {
            return true;
        }
        if let Some(includes) = &config.include_files
            && !includes.iter().any(|f| f.eq_ignore_ascii_case(&file_name))
        {
            return true;
        }
        let ext = path.extension().and_then(|e| e.to_str());
        if let Some(excludes) = &config.exclude_ext
            && ext.is_some()
            && excludes.iter().any(|e| e == &ext.unwrap().to_lowercase())
        {
            return true;
        }
        if let Some(includes) = &config.include_ext
            && (ext.is_none() || !includes.iter().any(|e| e == &ext.unwrap().to_lowercase()))
        {
            return true;
        }
    }
    false
}
