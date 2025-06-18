#[cfg(test)]
mod tests {
    use crate::cli::Config;
    use crate::file_processing::{get_directory_structure, is_in_ignored_dir, process_files};
    use crate::tests::common::{create_file, setup_temp_dir, setup_test_dir};
    use ignore::gitignore::Gitignore;
    use std::fs;
    use std::fs::File;
    use std::io::{self, Write};
    use std::path::{Path, PathBuf};
    use tempfile::TempDir;

    fn create_gitignore_empty() -> Gitignore {
        Gitignore::empty()
    }

    #[test]
    fn test_is_in_ignored_dir() {
        let path = PathBuf::from("node_modules/test.txt");
        let ignored_dirs = ["node_modules", ".git"];
        let exclude_dirs = Some(vec!["src".to_string()]);
        assert!(is_in_ignored_dir(&path, &ignored_dirs, &exclude_dirs));

        let path = PathBuf::from("src/test.txt");
        assert!(is_in_ignored_dir(&path, &ignored_dirs, &exclude_dirs));

        let path = PathBuf::from("docs/test.txt");
        assert!(!is_in_ignored_dir(&path, &ignored_dirs, &exclude_dirs));
    }

    #[test]
    fn test_is_in_user_excluded_dir() {
        let path = PathBuf::from("custom_dir/test.txt");
        let ignored_dirs: Vec<&str> = vec![];
        let exclude_dirs = Some(vec!["custom_dir".to_string()]);
        assert!(is_in_ignored_dir(&path, &ignored_dirs, &exclude_dirs));

        let path = PathBuf::from("other_dir/test.txt");
        assert!(!is_in_ignored_dir(&path, &ignored_dirs, &exclude_dirs));
    }

    #[test]
    fn test_path_not_in_ignored_dir() {
        let path = Path::new("/home/user/project/src/main.rs");
        let ignored_dirs = vec![".git", "node_modules"];
        let exclude_dirs = Some(vec!["tests".to_string()]);
        assert!(!is_in_ignored_dir(path, &ignored_dirs, &exclude_dirs));
    }

    #[test]
    fn test_empty_ignored_dirs() {
        let path = Path::new("/home/user/.git/config");
        let ignored_dirs: Vec<&str> = vec![];
        let exclude_dirs: Option<Vec<String>> = None;
        assert!(!is_in_ignored_dir(path, &ignored_dirs, &exclude_dirs));
    }

    #[test]
    fn test_root_path() {
        let path = Path::new("/");
        let ignored_dirs = vec![".git", "node_modules"];
        let exclude_dirs = Some(vec!["tests".to_string()]);
        assert!(!is_in_ignored_dir(path, &ignored_dirs, &exclude_dirs));
    }

    #[test]
    fn test_single_component_path() {
        let path = Path::new(".git");
        let ignored_dirs = vec![".git", "node_modules"];
        let exclude_dirs = Some(vec!["tests".to_string()]);
        assert!(is_in_ignored_dir(path, &ignored_dirs, &exclude_dirs));
    }

    #[test]
    fn test_path_with_similar_prefix() {
        let path = Path::new("/home/user/gitlab/project");
        let ignored_dirs = vec![".git", "node_modules"];
        let exclude_dirs = Some(vec!["tests".to_string()]);
        assert!(!is_in_ignored_dir(path, &ignored_dirs, &exclude_dirs));
    }

    #[test]
    fn test_case_sensitivity() {
        let path = Path::new("/home/user/NODE_MODULES/cache");
        let ignored_dirs = vec!["node_modules"];
        let exclude_dirs = Some(vec!["tests".to_string()]);
        assert!(is_in_ignored_dir(path, &ignored_dirs, &exclude_dirs));

        let path = Path::new("/home/user/TESTS/doc.txt");
        assert!(is_in_ignored_dir(path, &ignored_dirs, &exclude_dirs));
    }

    #[test]
    fn test_empty_path() {
        let path = Path::new("");
        let ignored_dirs = vec![".git", "node_modules"];
        let exclude_dirs = Some(vec!["tests".to_string()]);
        assert!(!is_in_ignored_dir(path, &ignored_dirs, &exclude_dirs));
    }

    #[test]
    fn test_get_directory_structure() -> io::Result<()> {
        let temp_dir = setup_temp_dir();
        create_file(temp_dir.path().join("file1.txt"), "Content 1")?;
        fs::create_dir(temp_dir.path().join("subdir"))?;
        create_file(temp_dir.path().join("subdir/file2.txt"), "Content 2")?;

        let ignored_dirs = ["node_modules"];
        let exclude_dirs = Some(vec!["subdir".to_string()]);
        let gitignore = create_gitignore_empty();
        let structure =
            get_directory_structure(temp_dir.path(), &gitignore, &ignored_dirs, &exclude_dirs)?;

        assert!(structure.contains("=== Project Directory Structure ==="));
        assert!(structure.contains("file1.txt"));
        assert!(!structure.contains("subdir/"));
        assert!(!structure.contains("file2.txt"));
        Ok(())
    }

    #[test]
    fn test_basic_directory_structure() {
        let (temp_dir, gitignore) = setup_test_dir();
        let root = temp_dir.path();
        let ignored_dirs = vec![];
        let exclude_dirs: Option<Vec<String>> = None;

        let result =
            get_directory_structure(root, &gitignore, &ignored_dirs, &exclude_dirs).unwrap();

        assert!(result.contains("=== Project Directory Structure ==="));
        assert!(result.contains(".gitignore"));
        assert!(result.contains("README.md"));
        assert!(result.contains("src/"));
        assert!(result.contains("main.rs"));
        assert!(result.contains("tests/"));
        assert!(result.contains("test1.rs"));
    }

    #[test]
    fn test_ignored_directories() {
        let (temp_dir, gitignore) = setup_test_dir();
        let root = temp_dir.path();
        let ignored_dirs = vec!["tests"];
        let exclude_dirs = Some(vec!["src".to_string()]);

        let result =
            get_directory_structure(root, &gitignore, &ignored_dirs, &exclude_dirs).unwrap();

        assert!(result.contains("=== Project Directory Structure ==="));
        assert!(result.contains(".gitignore"));
        assert!(result.contains("README.md"));
        assert!(!result.contains("src/"));
        assert!(!result.contains("main.rs"));
        assert!(!result.contains("tests/"));
    }

    #[test]
    fn test_gitignore_rules() {
        let (temp_dir, gitignore) = setup_test_dir();
        let root = temp_dir.path();
        let ignored_dirs = vec![];
        let exclude_dirs: Option<Vec<String>> = None;

        let result =
            get_directory_structure(root, &gitignore, &ignored_dirs, &exclude_dirs).unwrap();

        // Verify that target/ directory is ignored due to .gitignore
        assert!(!result.contains("target/"));
    }

    #[test]
    fn test_empty_directory() {
        let temp_dir = TempDir::new().unwrap();
        let root = temp_dir.path();
        let gitignore = Gitignore::empty();
        let ignored_dirs = vec![];
        let exclude_dirs: Option<Vec<String>> = None;

        let result =
            get_directory_structure(root, &gitignore, &ignored_dirs, &exclude_dirs).unwrap();

        assert!(result.contains("=== Project Directory Structure ==="));
        assert!(result.contains("The directory is empty."));
    }

    #[test]
    fn test_nested_directories() {
        let temp_dir = TempDir::new().unwrap();
        let root = temp_dir.path();

        // Create nested directory structure
        fs::create_dir_all(root.join("src/core/utils")).unwrap();
        File::create(root.join("src/core/utils/helper.rs")).unwrap();

        let gitignore = Gitignore::empty();
        let ignored_dirs = vec![];
        let exclude_dirs = Some(vec!["core".to_string()]);

        let result =
            get_directory_structure(root, &gitignore, &ignored_dirs, &exclude_dirs).unwrap();

        assert!(result.contains("=== Project Directory Structure ==="));
        assert!(result.contains("src/"));
        assert!(!result.contains("core/"));
        assert!(!result.contains("utils/"));
        assert!(!result.contains("helper.rs"));
    }

    #[test]
    fn test_non_existent_root() {
        let root = Path::new("/non/existent/path");
        let gitignore = Gitignore::empty();
        let ignored_dirs = vec![];
        let exclude_dirs: Option<Vec<String>> = None;

        let result = get_directory_structure(root, &gitignore, &ignored_dirs, &exclude_dirs);
        assert!(result.is_err());
    }

    #[test]
    fn test_process_files_basic() -> io::Result<()> {
        let temp_dir = setup_temp_dir();
        create_file(temp_dir.path().join("file1.txt"), "Hello, AI!")?;
        create_file(temp_dir.path().join("file2.md"), "# Markdown")?;

        let config = Config {
            directory: temp_dir.path().to_path_buf(),
            output: temp_dir.path().join("output.txt"),
            extensions: None,
            min_size: Some(0),
            max_size: None,
            exclude_dirs: None,
        };

        let ignored_dirs = ["node_modules"];
        let gitignore = create_gitignore_empty();
        let dir_structure = get_directory_structure(
            temp_dir.path(),
            &gitignore,
            &ignored_dirs,
            &config.exclude_dirs,
        )?;
        process_files(&config, &gitignore, &dir_structure, &ignored_dirs)?;

        let output_content = fs::read_to_string(&config.output)?;
        assert!(output_content.contains("=== File: file1.txt"));
        assert!(output_content.contains("Hello, AI!"));
        assert!(output_content.contains("=== File: file2.md"));
        assert!(output_content.contains("# Markdown"));
        Ok(())
    }

    #[test]
    fn test_process_files_size_filter() -> io::Result<()> {
        let temp_dir = setup_temp_dir();
        create_file(temp_dir.path().join("small.txt"), "Small")?;
        create_file(temp_dir.path().join("large.txt"), &"a".repeat(60000))?;

        let config = Config {
            directory: temp_dir.path().to_path_buf(),
            output: temp_dir.path().join("output.txt"),
            extensions: None,
            min_size: Some(10000),
            max_size: Some(100000),
            exclude_dirs: None,
        };

        let ignored_dirs = ["node_modules"];
        let gitignore = create_gitignore_empty();
        let dir_structure = get_directory_structure(
            temp_dir.path(),
            &gitignore,
            &ignored_dirs,
            &config.exclude_dirs,
        )?;
        process_files(&config, &gitignore, &dir_structure, &ignored_dirs)?;

        let output_content = fs::read_to_string(&config.output)?;
        assert!(
            !output_content.contains("=== File: small.txt"),
            "Output contains small.txt header"
        );
        assert!(output_content.contains("=== File: large.txt"));
        Ok(())
    }

    #[test]
    fn test_process_files_non_utf8() -> io::Result<()> {
        let temp_dir = setup_temp_dir();
        let file_path = temp_dir.path().join("non_utf8.bin");
        let mut file = fs::File::create(&file_path)?;
        file.write_all(&[0xFF, 0xFF, 0xFF])?;

        let config = Config {
            directory: temp_dir.path().to_path_buf(),
            output: temp_dir.path().join("output.txt"),
            extensions: None,
            min_size: Some(0),
            max_size: None,
            exclude_dirs: None,
        };

        let ignored_dirs = ["node_modules"];
        let gitignore = create_gitignore_empty();
        let dir_structure = get_directory_structure(
            temp_dir.path(),
            &gitignore,
            &ignored_dirs,
            &config.exclude_dirs,
        )?;
        process_files(&config, &gitignore, &dir_structure, &ignored_dirs)?;

        // Output should not include non-UTF-8 file content
        let output_content = fs::read_to_string(&config.output)?;
        assert!(
            !output_content.contains("=== File: non_utf8.bin"),
            "Output contains non_utf8.bin header"
        );
        Ok(())
    }
}
