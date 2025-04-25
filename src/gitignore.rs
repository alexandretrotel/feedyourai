use ignore::gitignore::{Gitignore, GitignoreBuilder};
use std::fs::{self, OpenOptions};
use std::io::{self, Error, ErrorKind, Write};
use std::path::Path;

/// Builds a `Gitignore` instance from the specified directory and `.gitignore` file,
/// appending default ignored files and directories to `.gitignore` if they don't exist,
/// and normalizes existing directory entries to `folder/**`.
///
/// # Arguments
/// - `dir_path`: The directory to search for or create `.gitignore`.
///
/// # Returns
/// - `Ok(Gitignore)`: The constructed `Gitignore` instance.
/// - `Err(io::Error)`: If an error occurs while building the gitignore or writing to the file.
pub fn build_gitignore(dir_path: &Path) -> io::Result<Gitignore> {
    let ignored_files = [
        "bun.lock",
        "package-lock.json",
        "yarn.lock",
        "pnpm-lock.yaml",
        "Cargo.lock",
        ".DS_Store",
        "uv.lock",
    ];
    let ignored_dirs = ["node_modules", "target", "dist", "build"];
    let gitignore_path = dir_path.join(".gitignore");

    let mut gitignore_builder = GitignoreBuilder::new(dir_path);

    // Normalize existing .gitignore content
    normalize_gitignore(&gitignore_path)?;

    // Append new ignored items
    append_ignored_items(&gitignore_path, &ignored_files, &ignored_dirs)?;

    // Load .gitignore into builder
    load_gitignore(&mut gitignore_builder, &gitignore_path)?;

    gitignore_builder
        .build()
        .map_err(|e| Error::new(ErrorKind::Other, e))
}

/// Normalizes existing .gitignore content by converting directory entries to `folder/**`.
/// If the file doesn't exist, it does nothing.
///
/// # Arguments
/// - `gitignore_path`: The path to the .gitignore file.
///
/// # Returns
/// - `Ok(())`: If the normalization is successful.
/// - `Err(io::Error)`: If an error occurs while reading or writing the file.
pub fn normalize_gitignore(gitignore_path: &Path) -> io::Result<()> {
    if !gitignore_path.exists() {
        return Ok(());
    }

    let existing_content = fs::read_to_string(gitignore_path)?;
    let (normalized_lines, lines_changed) = normalize_lines(&existing_content);

    if lines_changed {
        fs::write(gitignore_path, normalized_lines.join("\n") + "\n")?;
    }

    Ok(())
}

/// Normalizes lines in .gitignore content, converting directory entries to `folder/**`.
/// If a line is empty, starts with `#`, or `!`, it is left unchanged.
/// If a line is a directory without `**`, it is converted to `folder/**`.
///
/// # Arguments
/// - `existing_content`: The existing content of the .gitignore file.
///
/// # Returns
/// - `(Vec<String>, bool)`: A tuple containing the normalized lines and a boolean indicating if any changes were made.
pub fn normalize_lines(existing_content: &str) -> (Vec<String>, bool) {
    let mut normalized_lines = Vec::new();
    let mut lines_changed = false;

    for line in existing_content.lines() {
        let original_line = line.to_string();
        let trimmed = line.trim();

        if trimmed.is_empty() || trimmed.starts_with('#') || trimmed.starts_with('!') {
            normalized_lines.push(original_line);
            continue;
        }

        let clean_line = trimmed
            .trim_end_matches('/')
            .trim_end_matches("/**")
            .trim_end();

        if !clean_line.contains('*') && !clean_line.contains('.') && !clean_line.contains(' ') {
            let dir_wildcard = format!("{}/**", clean_line);
            if trimmed != dir_wildcard {
                normalized_lines.push(dir_wildcard.clone());
                lines_changed = true;
                continue;
            }
        }

        normalized_lines.push(original_line);
    }

    (normalized_lines, lines_changed)
}

/// Appends ignored files and directories to .gitignore if they don't exist.
/// If the file doesn't exist, it creates it.
///
/// # Arguments
/// - `gitignore_path`: The path to the .gitignore file.
/// - `ignored_files`: A slice of file names to ignore.
/// - `ignored_dirs`: A slice of directory names to ignore.
///
/// # Returns
/// - `Ok(())`: If the appending is successful.
/// - `Err(io::Error)`: If an error occurs while reading or writing the file.
pub fn append_ignored_items(
    gitignore_path: &Path,
    ignored_files: &[&str],
    ignored_dirs: &[&str],
) -> io::Result<()> {
    let existing_content = if gitignore_path.exists() {
        fs::read_to_string(gitignore_path)?
    } else {
        String::new()
    };

    let mut file = OpenOptions::new()
        .write(true)
        .append(true)
        .create(true)
        .open(gitignore_path)?;

    if !existing_content.is_empty() {
        writeln!(file)?;
    }

    append_files(&mut file, &existing_content, ignored_files)?;
    append_directories(&mut file, &existing_content, ignored_dirs)?;

    file.flush()
}

/// Appends ignored files to .gitignore.
/// If the file doesn't exist, it creates it.
///
/// # Arguments
/// - `file`: The file handle to the .gitignore file.
/// - `existing_content`: The existing content of the .gitignore file.
/// - `ignored_files`: A slice of file names to ignore.
///
/// # Returns
/// - `Ok(())`: If the appending is successful.
/// - `Err(io::Error)`: If an error occurs while writing to the file.
pub fn append_files(
    file: &mut fs::File,
    existing_content: &str,
    ignored_files: &[&str],
) -> io::Result<()> {
    for ignored in ignored_files {
        if !existing_content.contains(ignored) {
            writeln!(file, "{}", ignored)?;
        }
    }
    Ok(())
}

/// Appends ignored directories to .gitignore in `folder/**` format.
/// If the file doesn't exist, it creates it.
///
/// # Arguments
/// - `file`: The file handle to the .gitignore file.
/// - `existing_content`: The existing content of the .gitignore file.
/// - `ignored_dirs`: A slice of directory names to ignore.
///
/// # Returns
/// - `Ok(())`: If the appending is successful.
/// - `Err(io::Error)`: If an error occurs while writing to the file.
pub fn append_directories(
    file: &mut fs::File,
    existing_content: &str,
    ignored_dirs: &[&str],
) -> io::Result<()> {
    for ignored in ignored_dirs {
        let normalized_dir = format!("{}/**", ignored);
        if !existing_content.contains(&normalized_dir) {
            writeln!(file, "{}", normalized_dir)?;
        }
    }
    Ok(())
}

/// Loads .gitignore into the GitignoreBuilder.
/// If the file doesn't exist, it does nothing.
///
/// # Arguments
/// - `builder`: The GitignoreBuilder instance.
/// - `gitignore_path`: The path to the .gitignore file.
///
/// # Returns
/// - `Ok(())`: If the loading is successful.
/// - `Err(io::Error)`: If an error occurs while reading the file.
pub fn load_gitignore(builder: &mut GitignoreBuilder, gitignore_path: &Path) -> io::Result<()> {
    if gitignore_path.exists() {
        builder.add(gitignore_path);
    }
    Ok(())
}
