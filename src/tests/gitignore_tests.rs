#[cfg(test)]
mod tests {
    use std::io;
    use tempfile::TempDir;

    use crate::{IGNORED_DIRS, IGNORED_FILES, gitignore::build_gitignore};

    #[test]
    fn test_build_gitignore_new_file() -> io::Result<()> {
        // Create a temporary directory
        let temp_dir = TempDir::new()?;

        // Build the Gitignore instance with no existing .gitignore and no excluded dirs
        let gitignore = build_gitignore(temp_dir.path(), &IGNORED_FILES, &IGNORED_DIRS, &None)?;

        // Verify that .gitignore file was not created
        let gitignore_path = temp_dir.path().join(".gitignore");
        assert!(
            !gitignore_path.exists(),
            ".gitignore file should not be created"
        );

        // Verify that default ignored files are ignored
        for file in IGNORED_FILES {
            let path = temp_dir.path().join(file);
            assert!(
                gitignore
                    .matched_path_or_any_parents(&path, false)
                    .is_ignore(),
                "Expected {} to be ignored",
                file
            );
        }

        // Verify that files in ignored directories are ignored
        for dir in IGNORED_DIRS {
            let test_file = temp_dir.path().join(dir).join("test.txt");
            assert!(
                gitignore
                    .matched_path_or_any_parents(&test_file, false)
                    .is_ignore(),
                "Expected files in {} to be ignored",
                dir
            );
        }

        // Verify that non-ignored files are not ignored
        let non_ignored_paths = [
            temp_dir.path().join("src/main.rs"),
            temp_dir.path().join("README.md"),
            temp_dir.path().join("docs/index.html"),
        ];
        for path in &non_ignored_paths {
            assert!(
                !gitignore
                    .matched_path_or_any_parents(path, false)
                    .is_ignore(),
                "Expected {} not to be ignored",
                path.display()
            );
        }

        Ok(())
    }
}
