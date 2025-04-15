use ignore::gitignore::{Gitignore, GitignoreBuilder};
use std::io::{self, Error, ErrorKind};
use std::path::Path;

/// Builds a `Gitignore` instance from the specified directory and `.gitignore` file.
///
/// # Arguments
/// - `dir_path`: The directory to search for `.gitignore`.
/// - `test_mode`: Whether to print debug information.
///
/// # Returns
/// - `Ok(Gitignore)`: The constructed `Gitignore` instance.
/// - `Err(io::Error)`: If an error occurs while building the gitignore.
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

    for ignored in ignored_files {
        gitignore_builder
            .add_line(None, ignored)
            .map_err(|e| Error::new(ErrorKind::Other, e))?;
    }

    let gitignore_path = dir_path.join(".gitignore");
    if gitignore_path.exists() {
        gitignore_builder.add(&gitignore_path);
        if test_mode {
            println!("Loaded .gitignore from: {:?}", gitignore_path);
        }
    }

    gitignore_builder
        .build()
        .map_err(|e| Error::new(ErrorKind::Other, e))
        .or_else(|_| Ok(Gitignore::empty()))
}
