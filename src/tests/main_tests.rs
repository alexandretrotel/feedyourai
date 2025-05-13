#[cfg(test)]
mod tests {
    use tempfile::TempDir;

    use crate::build_gitignore;
    use crate::copy_to_clipboard;
    use crate::get_directory_structure;
    use crate::process_files;
    use std::io;

    use crate::cli;

    // Mock trait for cli
    trait CliParser {
        fn parse_args(&self) -> io::Result<cli::Config>;
    }

    struct MockCliParser {
        result: io::Result<cli::Config>,
    }

    impl CliParser for MockCliParser {
        fn parse_args(&self) -> io::Result<cli::Config> {
            match &self.result {
                Ok(config) => Ok(config.clone()),
                Err(err) => Err(io::Error::new(err.kind(), err.to_string())),
            }
        }
    }

    #[test]
    fn test_main_with_mock_cli() {
        // Create a temporary directory for testing
        let temp_dir = TempDir::new().expect("Failed to create temporary directory");
        let temp_output = temp_dir.path().join("output.txt");

        let mock_cli = MockCliParser {
            result: Ok(cli::Config {
                directory: temp_dir.path().to_path_buf(),
                output: temp_output.clone(),
                extensions: vec![].into(),
                min_size: Some(0),
                max_size: Some(1024),
                exclude_dirs: None,
            }),
        };

        // Simulate main's logic with the mock
        let result = mock_cli.parse_args().and_then(|config| {
            // Use real implementations for other dependencies or mock them similarly
            let gitignore = build_gitignore(&config.directory)?;
            let ignored_dirs = [
                "node_modules",
                ".git",
                ".svn",
                ".hg",
                ".idea",
                ".vscode",
                "build",
                "dist",
                "src-tauri",
                ".venv",
                "__pycache__",
                ".pytest_cache",
                ".next",
                ".turbo",
                "out",
                "target",
            ];
            let dir_structure = get_directory_structure(
                &config.directory,
                &gitignore,
                &ignored_dirs,
                &config.exclude_dirs,
            )?;
            process_files(&config, &gitignore, &ignored_dirs, &dir_structure)?;
            copy_to_clipboard(&config.output)?;
            Ok(())
        });

        assert!(result.is_ok(), "Expected Ok, got {:?}", result);
    }
}
