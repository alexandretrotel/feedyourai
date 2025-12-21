#[cfg(test)]
mod tests {
    use tempfile::TempDir;

    use crate::config;

    #[test]
    fn test_main_run_with_config() {
        // Create a temporary directory and output path for the test
        let temp_dir = TempDir::new().expect("Failed to create temporary directory");
        let temp_output = temp_dir.path().join("output.txt");

        // Build a minimal Config to pass to `run_with_config`
        let cfg = config::Config {
            directory: temp_dir.path().to_path_buf(),
            output: temp_output.clone(),
            include_dirs: None,
            exclude_dirs: None,
            include_ext: None,
            exclude_ext: None,
            include_files: None,
            exclude_files: None,
            min_size: Some(0),
            max_size: Some(1024 * 1024),
            respect_gitignore: true,
            tree_only: false,
        };

        // Call the extracted function under test
        let result = crate::run_with_config(cfg);

        // Ensure it succeeded and produced the expected output file.
        assert!(result.is_ok(), "Expected Ok, got {:?}", result);
        assert!(
            temp_output.exists(),
            "Expected output file to be created at {:?}",
            temp_output
        );
    }
}
