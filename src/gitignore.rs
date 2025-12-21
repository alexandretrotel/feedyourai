use ignore::gitignore::{Gitignore, GitignoreBuilder};
use std::io;
use std::path::Path;

/// Builds a `Gitignore` instance from the specified directory and `.gitignore` file,
/// appending default ignored files and directories to `.gitignore` if they don't exist,
/// and normalizes existing directory entries to `folder/**`.
use crate::cli::Config;

pub fn build_gitignore(
    dir_path: &Path,
    ignored_files: &[&str],
    ignored_dirs: &[&str],
    config: &Config,
) -> io::Result<Gitignore> {
    let mut builder = GitignoreBuilder::new(dir_path);

    // Load existing .gitignore if it exists, without modifying it
    let gitignore_path = dir_path.join(".gitignore");
    if gitignore_path.exists() {
        builder.add(&gitignore_path);
    }

    // Add default ignored files as patterns
    for file in ignored_files {
        builder.add_line(None, file).map_err(io::Error::other)?;
    }

    // Add default ignored directories with trailing `/` to ignore contents
    for dir in ignored_dirs {
        builder
            .add_line(None, &format!("{}/", dir))
            .map_err(io::Error::other)?;
    }

    // Add user-specified excluded directories from CLI
    if let Some(exclude_dirs) = &config.exclude_dirs {
        for dir in exclude_dirs {
            builder
                .add_line(None, &format!("{}/", dir))
                .map_err(io::Error::other)?;
        }
    }

    builder.build().map_err(io::Error::other)
}
