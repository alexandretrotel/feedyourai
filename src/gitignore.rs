use ignore::gitignore::{Gitignore, GitignoreBuilder};
use std::fs::{self, OpenOptions};
use std::io::{self, Error, ErrorKind, Write};
use std::path::Path;

/// Builds a `Gitignore` instance from the specified directory and `.gitignore` file,
/// appending default ignored files and directories to `.gitignore` line by line if they don't already exist,
/// and normalizes existing directory entries to `folder/**`.
///
/// # Arguments
/// - `dir_path`: The directory to search for or create `.gitignore`.
/// - `test_mode`: Whether to print debug information.
///
/// # Returns
/// - `Ok(Gitignore)`: The constructed `Gitignore` instance.
/// - `Err(io::Error)`: If an error occurs while building the gitignore or writing to the file.
pub fn build_gitignore(dir_path: &Path, test_mode: bool) -> io::Result<Gitignore> {
    let mut gitignore_builder = GitignoreBuilder::new(dir_path);
    let ignored_files = [
        "bun.lock",
        "package-lock.json",
        "yarn.lock",
        "pnpm-lock.yaml",
        "Cargo.lock",
        ".DS_Store",
        "uv.lock",
    ];
    let ignored_dirs = ["node_modules", "target", "dist", "build"]; // Without trailing slashes

    let gitignore_path = dir_path.join(".gitignore");

    // Read existing .gitignore content, if it exists
    let existing_content = if gitignore_path.exists() {
        fs::read_to_string(&gitignore_path)?
    } else {
        String::new()
    };

    // Normalize existing directory entries in .gitignore
    let mut normalized_lines = Vec::new();
    let mut lines_changed = false;
    for line in existing_content.lines() {
        let original_line = line.to_string();
        let trimmed = line.trim();

        // Preserve empty lines and comments
        if trimmed.is_empty() || trimmed.starts_with('#') {
            normalized_lines.push(original_line);
            continue;
        }

        // Skip negated patterns (starting with !)
        if trimmed.starts_with('!') {
            normalized_lines.push(original_line);
            continue;
        }

        // Handle directory-like entries
        let mut normalized_line = original_line.clone();
        // Remove trailing slashes, /**, or inline comments for comparison
        let clean_line = trimmed
            .trim_end_matches('/')
            .trim_end_matches("/**")
            .trim_end();

        // Check if the line looks like a directory (no extension, not a file pattern)
        if !clean_line.contains('*') && !clean_line.contains('.') && !clean_line.contains(' ') {
            let dir_wildcard = format!("{}/**", clean_line);
            if trimmed != dir_wildcard {
                normalized_line = dir_wildcard.clone();
                lines_changed = true;
                if test_mode {
                    println!(
                        "Normalized directory '{}' to '{}' in .gitignore",
                        trimmed, normalized_line
                    );
                }
            }
        }

        normalized_lines.push(normalized_line);
    }

    // Write normalized content back to .gitignore if changes were made
    if lines_changed {
        fs::write(&gitignore_path, normalized_lines.join("\n") + "\n")?;
        if test_mode {
            println!("Updated .gitignore with normalized directory entries");
        }
    }

    // Re-read the content after normalization
    let existing_content = if gitignore_path.exists() {
        fs::read_to_string(&gitignore_path)?
    } else {
        String::new()
    };

    // Open .gitignore in append mode (creates it if it doesn't exist)
    let mut file = OpenOptions::new()
        .write(true)
        .append(true)
        .create(true)
        .open(&gitignore_path)?;

    // Write a newline if the file is not empty
    if !existing_content.is_empty() {
        writeln!(file)?;
    }

    // Add each ignored file to .gitignore if it's not already present
    for ignored in ignored_files {
        if !existing_content.contains(ignored) {
            writeln!(file, "{}", ignored)?;
            if test_mode {
                println!("Added file {} to .gitignore", ignored);
            }
        } else if test_mode {
            println!("Skipped file {} (already in .gitignore)", ignored);
        }
    }

    // Add each ignored directory to .gitignore if it's not already present, normalized to `folder/**`
    for ignored in ignored_dirs {
        let normalized_dir = format!("{}/**", ignored);
        if !existing_content.contains(&normalized_dir) {
            writeln!(file, "{}", normalized_dir)?;
            if test_mode {
                println!("Added directory {} to .gitignore", normalized_dir);
            }
        } else if test_mode {
            println!(
                "Skipped directory {} (already in .gitignore)",
                normalized_dir
            );
        }
    }

    // Ensure the file is flushed to disk
    file.flush()?;

    // Add the .gitignore file to the builder
    if gitignore_path.exists() {
        gitignore_builder.add(&gitignore_path);
        if test_mode {
            println!("Loaded .gitignore from: {:?}", gitignore_path);
            if let Ok(contents) = fs::read_to_string(&gitignore_path) {
                println!(".gitignore contents:\n{}", contents);
            }
        }
    }

    gitignore_builder
        .build()
        .map_err(|e| Error::new(ErrorKind::Other, e))
}
