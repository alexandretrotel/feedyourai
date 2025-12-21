#[cfg(test)]
mod tests {
    use std::fs;
    use std::io;
    use tempfile::TempDir;

    use crate::{IGNORED_DIRS, IGNORED_FILES, gitignore::build_gitignore};

    #[test]
    fn test_build_gitignore_new_file() -> io::Result<()> {
        // Create a temporary directory
        let temp_dir = TempDir::new()?;

        // Build the Gitignore instance with no existing .gitignore and no excluded dirs
        let config = crate::config::Config {
            directory: temp_dir.path().to_path_buf(),
            output: temp_dir.path().join("output.txt"),
            include_dirs: None,
            exclude_dirs: None,
            include_ext: None,
            exclude_ext: None,
            include_files: None,
            exclude_files: None,
            min_size: None,
            max_size: None,
            respect_gitignore: true,
            tree_only: false,
        };
        let gitignore = build_gitignore(temp_dir.path(), IGNORED_FILES, IGNORED_DIRS, &config)?;

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

    #[test]
    fn test_build_gitignore_loads_existing_gitignore() -> io::Result<()> {
        // Create a temporary directory and a .gitignore file
        let temp_dir = TempDir::new()?;
        let gitignore_path = temp_dir.path().join(".gitignore");

        // Write a custom ignore pattern and a directory entry to the .gitignore
        fs::write(&gitignore_path, "special.ignore\nlogs/\n")?;

        // Create files that should be ignored according to the .gitignore
        fs::write(temp_dir.path().join("special.ignore"), "ignored")?;
        fs::create_dir_all(temp_dir.path().join("logs"))?;
        fs::write(temp_dir.path().join("logs").join("a.log"), "ignored")?;

        // Build the Gitignore instance
        let config = crate::config::Config {
            directory: temp_dir.path().to_path_buf(),
            output: temp_dir.path().join("output.txt"),
            include_dirs: None,
            exclude_dirs: None,
            include_ext: None,
            exclude_ext: None,
            include_files: None,
            exclude_files: None,
            min_size: None,
            max_size: None,
            respect_gitignore: true,
            tree_only: false,
        };
        let gitignore = build_gitignore(temp_dir.path(), IGNORED_FILES, IGNORED_DIRS, &config)?;

        // Ensure .gitignore was detected (we created it, so builder.add(...) branch is executed)
        assert!(
            gitignore_path.exists(),
            ".gitignore should exist for this test"
        );

        // Verify that patterns from the existing .gitignore are respected
        let special = temp_dir.path().join("special.ignore");
        assert!(
            gitignore
                .matched_path_or_any_parents(&special, false)
                .is_ignore(),
            "Expected special.ignore to be ignored because of existing .gitignore"
        );

        let log_file = temp_dir.path().join("logs").join("a.log");
        assert!(
            gitignore
                .matched_path_or_any_parents(&log_file, false)
                .is_ignore(),
            "Expected files in logs/ to be ignored because of existing .gitignore entry"
        );

        Ok(())
    }

    #[test]
    fn test_build_gitignore_respects_cli_exclude_dirs() -> io::Result<()> {
        // Create a temporary directory
        let temp_dir = TempDir::new()?;

        // Create a directory that will be excluded via CLI config
        let cli_dir = "cli_exclude";
        fs::create_dir_all(temp_dir.path().join(cli_dir))?;
        fs::write(temp_dir.path().join(cli_dir).join("test.txt"), "ignored")?;

        // Build the Gitignore instance with exclude_dirs provided in config
        let config = crate::config::Config {
            directory: temp_dir.path().to_path_buf(),
            output: temp_dir.path().join("output.txt"),
            include_dirs: None,
            exclude_dirs: Some(vec![cli_dir.to_string()]),
            include_ext: None,
            exclude_ext: None,
            include_files: None,
            exclude_files: None,
            min_size: None,
            max_size: None,
            respect_gitignore: true,
            tree_only: false,
        };
        let gitignore = build_gitignore(temp_dir.path(), IGNORED_FILES, IGNORED_DIRS, &config)?;

        // Verify that files inside the CLI-specified excluded dir are ignored
        let test_file = temp_dir.path().join(cli_dir).join("test.txt");
        assert!(
            gitignore
                .matched_path_or_any_parents(&test_file, false)
                .is_ignore(),
            "Expected files in CLI exclude_dirs to be ignored"
        );

        Ok(())
    }
}
