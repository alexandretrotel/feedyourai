#[cfg(test)]
mod tests {
    use std::{
        fs::{self, File},
        io::{self, Write},
    };
    use tempfile::TempDir;

    use crate::gitignore::{build_gitignore, normalize_gitignore, normalize_lines};

    #[test]
    fn test_build_gitignore_new_file() -> io::Result<()> {
        let temp_dir = TempDir::new()?;
        let gitignore = build_gitignore(temp_dir.path(), false)?;

        // Verify .gitignore file was created
        let gitignore_path = temp_dir.path().join(".gitignore");
        assert!(gitignore_path.exists());

        // Read and verify content
        let content = fs::read_to_string(&gitignore_path)?;
        let expected_files = [
            "bun.lock",
            "package-lock.json",
            "yarn.lock",
            "pnpm-lock.yaml",
            "Cargo.lock",
            ".DS_Store",
            "uv.lock",
        ];
        let expected_dirs = ["node_modules/**", "target/**", "dist/**", "build/**"];

        for file in expected_files.iter() {
            assert!(content.contains(file), "Expected {} in .gitignore", file);
        }
        for dir in expected_dirs.iter() {
            assert!(content.contains(dir), "Expected {} in .gitignore", dir);
        }

        // Verify Gitignore instance
        assert!(
            gitignore
                .matched_path_or_any_parents(temp_dir.path().join("node_modules/test.txt"), false)
                .is_ignore()
        );
        assert!(
            gitignore
                .matched_path_or_any_parents(temp_dir.path().join("target/test.md"), false)
                .is_ignore()
        );
        assert!(
            gitignore
                .matched_path_or_any_parents(temp_dir.path().join("bun.lock"), false)
                .is_ignore()
        );
        assert!(
            gitignore
                .matched_path_or_any_parents(temp_dir.path().join("package-lock.json"), false)
                .is_ignore()
        );
        assert!(
            gitignore
                .matched_path_or_any_parents(temp_dir.path().join("yarn.lock"), false)
                .is_ignore()
        );
        assert!(
            gitignore
                .matched_path_or_any_parents(temp_dir.path().join("pnpm-lock.yaml"), false)
                .is_ignore()
        );

        Ok(())
    }

    #[test]
    fn test_normalize_gitignore_nonexistent_file() -> io::Result<()> {
        let temp_dir = TempDir::new()?;
        let gitignore_path = temp_dir.path().join(".gitignore");

        // Test with a non-existent .gitignore file
        let result = normalize_gitignore(&gitignore_path, false);
        assert!(result.is_ok(), "Expected Ok for non-existent file");

        Ok(())
    }

    #[test]
    fn test_normalize_gitignore_no_changes_needed() -> io::Result<()> {
        let temp_dir = TempDir::new()?;
        let gitignore_path = temp_dir.path().join(".gitignore");

        // Create .gitignore with already normalized content
        let content = "node_modules/**\n*.log\n";
        File::create(&gitignore_path)?.write_all(content.as_bytes())?;

        let result = normalize_gitignore(&gitignore_path, false);
        assert!(result.is_ok(), "Expected Ok for no changes needed");

        // Verify content unchanged
        let new_content = fs::read_to_string(&gitignore_path)?;
        assert_eq!(new_content, content, "Content should remain unchanged");

        Ok(())
    }

    #[test]
    fn test_normalize_gitignore_directory_normalization() -> io::Result<()> {
        let temp_dir = TempDir::new()?;
        let gitignore_path = temp_dir.path().join(".gitignore");

        // Create .gitignore with directories to normalize
        let original_content = "node_modules/\nbuild\n*.log\n";
        let expected_content = "node_modules/**\nbuild/**\n*.log\n";
        File::create(&gitignore_path)?.write_all(original_content.as_bytes())?;

        let result = normalize_gitignore(&gitignore_path, false);
        assert!(result.is_ok(), "Expected Ok for successful normalization");

        // Verify content was normalized
        let new_content = fs::read_to_string(&gitignore_path)?;
        assert_eq!(
            new_content, expected_content,
            "Content should be normalized"
        );

        Ok(())
    }

    #[test]
    fn test_normalize_gitignore_test_mode() -> io::Result<()> {
        let temp_dir = TempDir::new()?;
        let gitignore_path = temp_dir.path().join(".gitignore");

        // Create .gitignore with directories to normalize
        let original_content = "dist/\n";
        let expected_content = "dist/**\n";
        File::create(&gitignore_path)?.write_all(original_content.as_bytes())?;

        // Capture stdout for test_mode output
        let output = std::panic::catch_unwind(|| {
            let result = normalize_gitignore(&gitignore_path, true);
            assert!(result.is_ok(), "Expected Ok in test mode");
        });

        // Verify content was normalized
        let new_content = fs::read_to_string(&gitignore_path)?;
        assert_eq!(
            new_content, expected_content,
            "Content should be normalized in test mode"
        );

        assert!(
            output.is_ok(),
            "Expected test mode to execute without panic"
        );

        Ok(())
    }

    #[test]
    fn test_normalize_gitignore_empty_file() -> io::Result<()> {
        let temp_dir = TempDir::new()?;
        let gitignore_path = temp_dir.path().join(".gitignore");

        // Create empty .gitignore
        File::create(&gitignore_path)?;

        let result = normalize_gitignore(&gitignore_path, false);
        assert!(result.is_ok(), "Expected Ok for empty file");

        // Verify content unchanged (still empty)
        let new_content = fs::read_to_string(&gitignore_path)?;
        assert_eq!(new_content, "", "Empty file should remain empty");

        Ok(())
    }

    #[test]
    fn test_normalize_gitignore_file_permission_error() -> io::Result<()> {
        let temp_dir = TempDir::new()?;
        let gitignore_path = temp_dir.path().join(".gitignore");

        // Create .gitignore and make it read-only
        let content = "node_modules/\n";
        File::create(&gitignore_path)?.write_all(content.as_bytes())?;
        let mut permissions = fs::metadata(&gitignore_path)?.permissions();
        permissions.set_readonly(true);
        fs::set_permissions(&gitignore_path, permissions)?;

        let result = normalize_gitignore(&gitignore_path, false);
        assert!(result.is_err(), "Expected Err for read-only file");

        Ok(())
    }

    #[test]
    fn test_empty_content() {
        let input = "";
        let (lines, changed) = normalize_lines(input, false);
        assert_eq!(lines, Vec::<String>::new());
        assert_eq!(changed, false);
    }

    #[test]
    fn test_empty_line() {
        let input = "\n";
        let (lines, changed) = normalize_lines(input, false);
        assert_eq!(lines, vec![""]);
        assert_eq!(changed, false);
    }

    #[test]
    fn test_comment_line() {
        let input = "# This is a comment";
        let (lines, changed) = normalize_lines(input, false);
        assert_eq!(lines, vec!["# This is a comment"]);
        assert_eq!(changed, false);
    }

    #[test]
    fn test_negation_line() {
        let input = "!important.txt";
        let (lines, changed) = normalize_lines(input, false);
        assert_eq!(lines, vec!["!important.txt"]);
        assert_eq!(changed, false);
    }

    #[test]
    fn test_directory_normalization() {
        let input = "folder/";
        let (lines, changed) = normalize_lines(input, false);
        assert_eq!(lines, vec!["folder/**"]);
        assert_eq!(changed, true);
    }

    #[test]
    fn test_directory_already_normalized() {
        let input = "folder/**";
        let (lines, changed) = normalize_lines(input, false);
        assert_eq!(lines, vec!["folder/**"]);
        assert_eq!(changed, false);
    }

    #[test]
    fn test_file_pattern_unchanged() {
        let input = "*.log";
        let (lines, changed) = normalize_lines(input, false);
        assert_eq!(lines, vec!["*.log"]);
        assert_eq!(changed, false);
    }

    #[test]
    fn test_file_with_extension_unchanged() {
        let input = "config.json";
        let (lines, changed) = normalize_lines(input, false);
        assert_eq!(lines, vec!["config.json"]);
        assert_eq!(changed, false);
    }

    #[test]
    fn test_mixed_content() {
        let input = "# Comment\nfolder/\n*.log\n!important.txt\nanother_folder";
        let expected = vec![
            "# Comment",
            "folder/**",
            "*.log",
            "!important.txt",
            "another_folder/**",
        ];
        let (lines, changed) = normalize_lines(input, false);
        assert_eq!(lines, expected);
        assert_eq!(changed, true);
    }

    #[test]
    fn test_trailing_slashes_and_wildcards() {
        let input = "folder///";
        let (lines, changed) = normalize_lines(input, false);
        assert_eq!(lines, vec!["folder/**"]);
        assert_eq!(changed, true);
    }

    #[test]
    fn test_test_mode_no_output() {
        let input = "folder/";
        let (lines, changed) = normalize_lines(input, true);
        assert_eq!(lines, vec!["folder/**"]);
        assert_eq!(changed, true);
    }

    #[test]
    fn test_line_with_spaces_unchanged() {
        let input = "folder with spaces";
        let (lines, changed) = normalize_lines(input, false);
        assert_eq!(lines, vec!["folder with spaces"]);
        assert_eq!(changed, false);
    }
}
