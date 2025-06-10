use ignore::gitignore::{Gitignore, GitignoreBuilder};
use std::io;
use std::path::Path;

/// Builds a `Gitignore` instance from the specified directory and `.gitignore` file,
/// appending default ignored files and directories to `.gitignore` if they don't exist,
/// and normalizes existing directory entries to `folder/**`.
///
/// # Arguments
/// - `dir_path`: The directory to search for or create `.gitignore`.
/// - `ignored_files`: A slice of file names to ignore.
/// - `ignored_dirs`: A slice of directory names to ignore.
/// - `exclude_dirs`: An optional vector of user-specified directories to exclude from the gitignore.
///
/// # Returns
/// - `Ok(Gitignore)`: The constructed `Gitignore` instance.
/// - `Err(io::Error)`: If an error occurs while building the gitignore or writing to the file.
pub fn build_gitignore(
    dir_path: &Path,
    ignored_files: &[&str],
    ignored_dirs: &[&str],
    exclude_dirs: &Option<Vec<String>>,
) -> io::Result<Gitignore> {
    let mut builder = GitignoreBuilder::new(dir_path);

    // Load existing .gitignore if it exists, without modifying it
    let gitignore_path = dir_path.join(".gitignore");
    if gitignore_path.exists() {
        builder.add(&gitignore_path);
    }

    // Add default ignored files as patterns
    for file in ignored_files {
        builder
            .add_line(None, file)
            .map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;
    }

    // Add default ignored directories with trailing `/` to ignore contents
    for dir in ignored_dirs {
        builder
            .add_line(None, &format!("{}/", dir))
            .map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;
    }

    // Add user-specified excluded directories from CLI
    if let Some(exclude_dirs) = exclude_dirs {
        for dir in exclude_dirs {
            builder
                .add_line(None, &format!("{}/", dir))
                .map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;
        }
    }

    builder
        .build()
        .map_err(|e| io::Error::new(io::ErrorKind::Other, e))
}
