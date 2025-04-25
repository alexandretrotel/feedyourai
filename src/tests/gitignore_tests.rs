#[cfg(test)]
mod tests {
    use ignore::gitignore::GitignoreBuilder;
    use std::{
        fs::{self, File},
        io::{self, Read, Write},
        path::Path,
    };
    use tempfile::{NamedTempFile, TempDir, tempdir};

    use crate::{
        gitignore::{
            append_directories, append_files, append_ignored_items, build_gitignore,
            load_gitignore, normalize_gitignore, normalize_lines,
        },
        tests::common::{create_gitignore, read_file_content, setup_gitignore},
    };

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

    #[test]
    fn test_create_new_gitignore() {
        let (_temp_dir, gitignore_path) = setup_gitignore("");
        let files = &["file1.txt", "file2.txt"];
        let dirs = &["dir1", "dir2"];

        let result = append_ignored_items(&gitignore_path, files, dirs, false);
        assert!(result.is_ok());

        let content = read_file_content(&gitignore_path);
        assert_eq!(content, "file1.txt\nfile2.txt\ndir1/**\ndir2/**\n");
    }

    #[test]
    fn test_append_to_existing_gitignore() {
        let (_temp_dir, gitignore_path) = setup_gitignore("existing.txt\n");
        let files = &["file1.txt"];
        let dirs = &["dir1"];

        let result = append_ignored_items(&gitignore_path, files, dirs, false);
        assert!(result.is_ok());

        let content = read_file_content(&gitignore_path);
        assert_eq!(content, "existing.txt\n\nfile1.txt\ndir1/**\n");
    }

    #[test]
    fn test_empty_input() {
        let (_temp_dir, gitignore_path) = setup_gitignore("existing.txt\n");
        let files: &[&str] = &[];
        let dirs: &[&str] = &[];

        let result = append_ignored_items(&gitignore_path, files, dirs, false);
        assert!(result.is_ok());

        let content = read_file_content(&gitignore_path);
        assert_eq!(content, "existing.txt\n\n");
    }

    #[test]
    fn test_invalid_path() {
        let invalid_path = Path::new("/invalid/path/.gitignore");
        let files = &["file1.txt"];
        let dirs = &["dir1/"];

        let result = append_ignored_items(invalid_path, files, dirs, false);
        assert!(result.is_err());
    }

    #[test]
    fn test_append_new_files() -> io::Result<()> {
        let dir = tempdir()?;
        let file_path = dir.path().join(".gitignore");
        let mut file = File::create(&file_path)?;
        let ignored_files = vec!["node_modules", ".env"];
        let existing_content = "";

        append_files(&mut file, existing_content, &ignored_files, false)?;

        let content = read_file_content(&file_path);
        assert_eq!(content, "node_modules\n.env\n");
        Ok(())
    }

    #[test]
    fn test_skip_existing_files() -> io::Result<()> {
        let dir = tempdir()?;
        let file_path = dir.path().join(".gitignore");
        let mut file = File::create(&file_path)?;
        let initial_content = "node_modules\n";
        write!(file, "{}", initial_content)?;
        file.sync_all()?;

        let ignored_files = vec!["node_modules", ".env"];
        let mut file = File::open(&file_path)?;
        let mut existing_content = String::new();
        file.read_to_string(&mut existing_content)?;
        let mut file = File::create(&file_path)?;

        append_files(&mut file, &existing_content, &ignored_files, false)?;

        let content = read_file_content(&file_path);
        assert_eq!(content, ".env\n");
        Ok(())
    }

    #[test]
    fn test_empty_ignored_files() -> io::Result<()> {
        let dir = tempdir()?;
        let file_path = dir.path().join(".gitignore");
        let mut file = File::create(&file_path)?;
        let ignored_files: Vec<&str> = vec![];
        let existing_content = "node_modules\n";

        append_files(&mut file, &existing_content, &ignored_files, false)?;

        let content = read_file_content(&file_path);
        assert_eq!(content, "");
        Ok(())
    }

    #[test]
    fn test_append_new_directories() -> io::Result<()> {
        let mut temp_file = NamedTempFile::new()?;
        let existing_content = "";
        let ignored_dirs = vec!["node_modules", "dist"];
        let test_mode = false;

        append_directories(
            temp_file.as_file_mut(),
            existing_content,
            &ignored_dirs,
            test_mode,
        )?;
        let content = read_file_content(temp_file.path());

        assert_eq!(content, "node_modules/**\ndist/**\n");
        Ok(())
    }

    #[test]
    fn test_skip_existing_directories() -> io::Result<()> {
        let mut temp_file = NamedTempFile::new()?;
        let existing_content = "node_modules/**";
        writeln!(temp_file.as_file_mut(), "{}", existing_content)?;
        let ignored_dirs = vec!["node_modules", "dist"];
        let test_mode = false;

        append_directories(
            temp_file.as_file_mut(),
            existing_content,
            &ignored_dirs,
            test_mode,
        )?;
        let content = read_file_content(temp_file.path());

        assert_eq!(content, "node_modules/**\ndist/**\n");
        Ok(())
    }

    #[test]
    fn test_empty_directories_list() -> io::Result<()> {
        let mut temp_file = NamedTempFile::new()?;
        let existing_content = "";
        let ignored_dirs: Vec<&str> = vec![];
        let test_mode = false;

        append_directories(
            temp_file.as_file_mut(),
            existing_content,
            &ignored_dirs,
            test_mode,
        )?;
        let content = read_file_content(temp_file.path());

        assert_eq!(content, "");
        Ok(())
    }

    #[test]
    fn test_load_gitignore_file_exists() {
        let temp_dir = TempDir::new().unwrap();
        let gitignore_content = "*.log\nnode_modules/";
        create_gitignore(&temp_dir, gitignore_content).unwrap();
        let gitignore_path = temp_dir.path().join(".gitignore");

        let mut builder = GitignoreBuilder::new(temp_dir.path());

        let result = load_gitignore(&mut builder, &gitignore_path, false);

        assert!(result.is_ok());
    }

    #[test]
    fn test_load_gitignore_file_does_not_exist() {
        let temp_dir = TempDir::new().unwrap();
        let gitignore_path = temp_dir.path().join(".gitignore");

        let mut builder = GitignoreBuilder::new(temp_dir.path());

        let result = load_gitignore(&mut builder, &gitignore_path, false);

        assert!(result.is_ok());
    }

    #[test]
    fn test_load_gitignore_invalid_file() {
        let temp_dir = TempDir::new().unwrap();
        let gitignore_path = temp_dir.path().join(".gitignore");
        File::create(&gitignore_path).unwrap();
        let mut perms = fs::metadata(&gitignore_path).unwrap().permissions();
        perms.set_readonly(true);
        fs::set_permissions(&gitignore_path, perms).unwrap();

        let mut builder = GitignoreBuilder::new(temp_dir.path());

        let result = load_gitignore(&mut builder, &gitignore_path, false);

        assert!(result.is_ok());
    }
}
