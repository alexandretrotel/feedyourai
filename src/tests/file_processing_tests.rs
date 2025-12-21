#[cfg(test)]
mod tests {
    use crate::config::Config;
    use crate::file_processing::{
        get_directory_structure, is_in_ignored_dir, process_files, should_skip_path_advanced,
    };
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
        let config = Config {
            directory: temp_dir.path().to_path_buf(),
            output: temp_dir.path().join("output.txt"),
            include_dirs: None,
            exclude_dirs: Some(vec!["subdir".to_string()]),
            include_ext: None,
            exclude_ext: None,
            include_files: None,
            exclude_files: None,
            min_size: None,
            max_size: None,
            respect_gitignore: true,
            tree_only: false,
        };
        let gitignore = create_gitignore_empty();
        let structure =
            get_directory_structure(temp_dir.path(), &gitignore, &ignored_dirs, &config)?;

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

        let config = Config {
            directory: root.to_path_buf(),
            output: root.join("output.txt"),
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

        let result = get_directory_structure(root, &gitignore, &ignored_dirs, &config).unwrap();

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

        let config = Config {
            directory: root.to_path_buf(),
            output: root.join("output.txt"),
            include_dirs: None,
            exclude_dirs: Some(vec!["src".to_string()]),
            include_ext: None,
            exclude_ext: None,
            include_files: None,
            exclude_files: None,
            min_size: None,
            max_size: None,
            respect_gitignore: true,
            tree_only: false,
        };

        let result = get_directory_structure(root, &gitignore, &ignored_dirs, &config).unwrap();

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

        let config = Config {
            directory: root.to_path_buf(),
            output: root.join("output.txt"),
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

        let result = get_directory_structure(root, &gitignore, &ignored_dirs, &config).unwrap();

        // Verify that target/ directory is ignored due to .gitignore
        assert!(!result.contains("target/"));
    }

    #[test]
    fn test_empty_directory() {
        let temp_dir = TempDir::new().unwrap();
        let root = temp_dir.path();
        let gitignore = Gitignore::empty();
        let ignored_dirs = vec![];

        let config = Config {
            directory: root.to_path_buf(),
            output: root.join("output.txt"),
            include_dirs: None,
            exclude_dirs: None,
            include_ext: None,
            exclude_ext: None,
            include_files: None,
            exclude_files: None,
            min_size: Some(0),
            max_size: None,
            respect_gitignore: true,
            tree_only: false,
        };

        let result = get_directory_structure(root, &gitignore, &ignored_dirs, &config).unwrap();

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
        let config = Config {
            directory: root.to_path_buf(),
            output: root.join("output.txt"),
            include_dirs: None,
            exclude_dirs: Some(vec!["core".to_string()]),
            include_ext: None,
            exclude_ext: None,
            include_files: None,
            exclude_files: None,
            min_size: None,
            max_size: None,
            respect_gitignore: true,
            tree_only: false,
        };

        let result = get_directory_structure(root, &gitignore, &ignored_dirs, &config).unwrap();

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
        let config = Config {
            directory: root.to_path_buf(),
            output: root.join("output.txt"),
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

        let result = get_directory_structure(root, &gitignore, &ignored_dirs, &config);
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
            include_dirs: None,
            exclude_dirs: None,
            include_ext: None,
            exclude_ext: None,
            include_files: None,
            exclude_files: None,
            min_size: Some(0),
            max_size: None,
            respect_gitignore: true,
            tree_only: false,
        };

        let ignored_dirs = ["node_modules"];
        let gitignore = create_gitignore_empty();
        let dir_structure =
            get_directory_structure(temp_dir.path(), &gitignore, &ignored_dirs, &config)?;
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
            include_dirs: None,
            exclude_dirs: None,
            include_ext: None,
            exclude_ext: None,
            include_files: None,
            exclude_files: None,
            min_size: Some(10000),
            max_size: Some(100000),
            respect_gitignore: true,
            tree_only: false,
        };

        let ignored_dirs = ["node_modules"];
        let gitignore = create_gitignore_empty();
        let dir_structure =
            get_directory_structure(temp_dir.path(), &gitignore, &ignored_dirs, &config)?;
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
            include_dirs: None,
            exclude_dirs: None,
            include_ext: None,
            exclude_ext: None,
            include_files: None,
            exclude_files: None,
            min_size: Some(0),
            max_size: None,
            respect_gitignore: true,
            tree_only: false,
        };

        let ignored_dirs = ["node_modules"];
        let gitignore = create_gitignore_empty();
        let dir_structure =
            get_directory_structure(temp_dir.path(), &gitignore, &ignored_dirs, &config)?;
        process_files(&config, &gitignore, &dir_structure, &ignored_dirs)?;

        // Output should not include non-UTF-8 file content
        let output_content = fs::read_to_string(&config.output)?;
        assert!(
            !output_content.contains("=== File: non_utf8.bin"),
            "Output contains non_utf8.bin header"
        );
        Ok(())
    }

    #[test]
    fn test_should_skip_path_ignored_dirs() {
        let gitignore = create_gitignore_empty();
        let ignored_dirs = ["node_modules", ".git", "target"];
        let config = Config {
            directory: PathBuf::from("."),
            output: PathBuf::from("out.txt"),
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

        // Test directory paths that should be skipped
        let path = Path::new("project/node_modules");
        assert!(should_skip_path_advanced(
            path,
            true,
            &gitignore,
            &ignored_dirs,
            &config
        ));

        let path = Path::new("project/.git/config");
        assert!(should_skip_path_advanced(
            path,
            false,
            &gitignore,
            &ignored_dirs,
            &config
        ));

        let path = Path::new("rust_project/target/debug/main");
        assert!(should_skip_path_advanced(
            path,
            false,
            &gitignore,
            &ignored_dirs,
            &config
        ));

        // Test paths that should not be skipped
        let path = Path::new("project/src/main.rs");
        assert!(!should_skip_path_advanced(
            path,
            false,
            &gitignore,
            &ignored_dirs,
            &config
        ));

        let path = Path::new("project/README.md");
        assert!(!should_skip_path_advanced(
            path,
            false,
            &gitignore,
            &ignored_dirs,
            &config
        ));
    }

    #[test]
    fn test_should_skip_path_exclude_dirs() {
        let gitignore = create_gitignore_empty();
        let ignored_dirs: Vec<&str> = vec![];
        let config = Config {
            directory: PathBuf::from("."),
            output: PathBuf::from("out.txt"),
            include_dirs: None,
            exclude_dirs: Some(vec!["tests".to_string(), "docs".to_string()]),
            include_ext: None,
            exclude_ext: None,
            include_files: None,
            exclude_files: None,
            min_size: None,
            max_size: None,
            respect_gitignore: true,
            tree_only: false,
        };

        // Test directory paths that should be skipped due to exclude_dirs
        let path = Path::new("project/tests/unit_test.rs");
        assert!(should_skip_path_advanced(
            path,
            false,
            &gitignore,
            &ignored_dirs,
            &config
        ));

        let path = Path::new("project/docs/README.md");
        assert!(should_skip_path_advanced(
            path,
            false,
            &gitignore,
            &ignored_dirs,
            &config
        ));

        // Test paths that should not be skipped
        let path = Path::new("project/src/main.rs");
        assert!(!should_skip_path_advanced(
            path,
            false,
            &gitignore,
            &ignored_dirs,
            &config
        ));
    }

    #[test]
    fn test_should_skip_path_case_insensitive() {
        let gitignore = create_gitignore_empty();
        let ignored_dirs = ["node_modules"];
        let config = Config {
            directory: PathBuf::from("."),
            output: PathBuf::from("out.txt"),
            include_dirs: None,
            exclude_dirs: Some(vec!["Tests".to_string()]),
            include_ext: None,
            exclude_ext: None,
            include_files: None,
            exclude_files: None,
            min_size: None,
            max_size: None,
            respect_gitignore: true,
            tree_only: false,
        };

        // Test case insensitive matching for ignored_dirs
        let path = Path::new("project/NODE_MODULES/package");
        assert!(should_skip_path_advanced(
            path,
            true,
            &gitignore,
            &ignored_dirs,
            &config
        ));

        let path = Path::new("project/Node_Modules/package");
        assert!(should_skip_path_advanced(
            path,
            true,
            &gitignore,
            &ignored_dirs,
            &config
        ));

        // Test case insensitive matching for exclude_dirs
        let path = Path::new("project/tests/unit.rs");
        assert!(should_skip_path_advanced(
            path,
            false,
            &gitignore,
            &ignored_dirs,
            &config
        ));

        let path = Path::new("project/TESTS/integration.rs");
        assert!(should_skip_path_advanced(
            path,
            false,
            &gitignore,
            &ignored_dirs,
            &config
        ));
    }

    #[test]
    fn test_should_skip_path_with_gitignore() -> io::Result<()> {
        let temp_dir = setup_temp_dir();
        let root = temp_dir.path();

        // Create a .gitignore file with specific rules
        let gitignore_content = "*.log\n/build/\ntmp/\n";
        create_file(root.join(".gitignore"), gitignore_content)?;
        let gitignore = Gitignore::new(root.join(".gitignore")).0;

        let ignored_dirs: Vec<&str> = vec![];
        let config = Config {
            directory: PathBuf::from("."),
            output: PathBuf::from("out.txt"),
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

        // Test files that should be skipped due to gitignore rules
        let path = root.join("app.log");
        assert!(should_skip_path_advanced(
            &path,
            false,
            &gitignore,
            &ignored_dirs,
            &config
        ));

        let path = root.join("build");
        assert!(should_skip_path_advanced(
            &path,
            true,
            &gitignore,
            &ignored_dirs,
            &config
        ));

        let path = root.join("tmp");
        assert!(should_skip_path_advanced(
            &path,
            true,
            &gitignore,
            &ignored_dirs,
            &config
        ));

        // Test files that should not be skipped
        let path = root.join("src/main.rs");
        assert!(!should_skip_path_advanced(
            &path,
            false,
            &gitignore,
            &ignored_dirs,
            &config
        ));

        let path = root.join("README.md");
        assert!(!should_skip_path_advanced(
            &path,
            false,
            &gitignore,
            &ignored_dirs,
            &config
        ));

        Ok(())
    }

    #[test]
    fn test_should_skip_path_combined_rules() -> io::Result<()> {
        let temp_dir = setup_temp_dir();
        let root = temp_dir.path();

        // Create a .gitignore file
        let gitignore_content = "*.tmp\n";
        create_file(root.join(".gitignore"), gitignore_content)?;
        let gitignore = Gitignore::new(root.join(".gitignore")).0;

        let ignored_dirs = ["node_modules", ".git"];
        let config = Config {
            directory: PathBuf::from("."),
            output: PathBuf::from("out.txt"),
            include_dirs: None,
            exclude_dirs: Some(vec!["target".to_string(), "tests".to_string()]),
            include_ext: None,
            exclude_ext: None,
            include_files: None,
            exclude_files: None,
            min_size: None,
            max_size: None,
            respect_gitignore: true,
            tree_only: false,
        };

        // Test path that matches multiple rules (should be skipped)
        let path = root.join("node_modules/package.tmp");
        assert!(should_skip_path_advanced(
            &path,
            false,
            &gitignore,
            &ignored_dirs,
            &config
        ));

        // Test path that matches gitignore only
        let path = root.join("src/cache.tmp");
        assert!(should_skip_path_advanced(
            &path,
            false,
            &gitignore,
            &ignored_dirs,
            &config
        ));

        // Test path that matches ignored_dirs only
        let path = root.join("node_modules/package.json");
        assert!(should_skip_path_advanced(
            &path,
            false,
            &gitignore,
            &ignored_dirs,
            &config
        ));

        // Test path that matches exclude_dirs only
        let path = root.join("tests/unit.rs");
        assert!(should_skip_path_advanced(
            &path,
            false,
            &gitignore,
            &ignored_dirs,
            &config
        ));

        // Test path that doesn't match any rule
        let path = root.join("src/main.rs");
        assert!(!should_skip_path_advanced(
            &path,
            false,
            &gitignore,
            &ignored_dirs,
            &config
        ));

        Ok(())
    }

    #[test]
    fn test_should_skip_path_empty_rules() {
        let gitignore = create_gitignore_empty();
        let ignored_dirs: Vec<&str> = vec![];
        let config = Config {
            directory: PathBuf::from("."),
            output: PathBuf::from("out.txt"),
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

        // When no rules are defined, no paths should be skipped
        let path = Path::new("any/path/file.txt");
        assert!(!should_skip_path_advanced(
            path,
            false,
            &gitignore,
            &ignored_dirs,
            &config
        ));

        let path = Path::new(".git/config");
        assert!(!should_skip_path_advanced(
            path,
            false,
            &gitignore,
            &ignored_dirs,
            &config
        ));

        let path = Path::new("node_modules/package.json");
        assert!(!should_skip_path_advanced(
            path,
            false,
            &gitignore,
            &ignored_dirs,
            &config
        ));
    }

    #[test]
    fn test_should_skip_path_file_vs_directory() {
        let gitignore = create_gitignore_empty();
        let ignored_dirs = ["target"];
        let config = Config {
            directory: PathBuf::from("."),
            output: PathBuf::from("out.txt"),
            include_dirs: None,
            exclude_dirs: Some(vec!["target".to_string()]),
            include_ext: None,
            exclude_ext: None,
            include_files: None,
            exclude_files: None,
            min_size: None,
            max_size: None,
            respect_gitignore: true,
            tree_only: false,
        };

        // Test the same path as both file and directory
        let path = Path::new("project/target");

        // As a directory, it should be skipped
        assert!(should_skip_path_advanced(
            path,
            true,
            &gitignore,
            &ignored_dirs,
            &config
        ));

        // As a file, it should also be skipped (because it's in the ignored directory)
        assert!(should_skip_path_advanced(
            path,
            false,
            &gitignore,
            &ignored_dirs,
            &config
        ));

        // Test a file inside the ignored directory
        let path = Path::new("project/target/debug/main");
        assert!(should_skip_path_advanced(
            path,
            false,
            &gitignore,
            &ignored_dirs,
            &config
        ));
    }

    // New tests to increase coverage around file processing branches:
    #[test]
    fn test_process_files_extension_filters() -> io::Result<()> {
        // Use separate temporary directories for each subcase so outputs from one run
        // cannot be picked up by subsequent runs.
        let ignored_dirs = ["node_modules"];
        let gitignore = create_gitignore_empty();

        // Subcase 1: include_ext only allows .md
        let temp_dir1 = setup_temp_dir();
        create_file(temp_dir1.path().join("a.txt"), "A")?;
        create_file(temp_dir1.path().join("b.md"), "B")?;
        create_file(temp_dir1.path().join("noext"), "NOEXT")?;

        let config_md = Config {
            directory: temp_dir1.path().to_path_buf(),
            output: temp_dir1.path().join("out_md.txt"),
            include_dirs: None,
            exclude_dirs: None,
            include_ext: Some(vec!["md".to_string()]),
            exclude_ext: None,
            include_files: None,
            exclude_files: None,
            min_size: Some(0),
            max_size: None,
            respect_gitignore: true,
            tree_only: false,
        };
        let dir_structure =
            get_directory_structure(temp_dir1.path(), &gitignore, &ignored_dirs, &config_md)?;
        process_files(&config_md, &gitignore, &dir_structure, &ignored_dirs)?;
        let out_md = fs::read_to_string(&config_md.output)?;
        assert!(out_md.contains("=== File: b.md"));
        assert!(!out_md.contains("=== File: a.txt"));
        assert!(!out_md.contains("=== File: noext"));

        // Subcase 2: exclude_ext prevents .md files
        let temp_dir2 = setup_temp_dir();
        create_file(temp_dir2.path().join("a.txt"), "A")?;
        create_file(temp_dir2.path().join("b.md"), "B")?;
        create_file(temp_dir2.path().join("noext"), "NOEXT")?;

        let config_excl = Config {
            directory: temp_dir2.path().to_path_buf(),
            output: temp_dir2.path().join("out_excl.txt"),
            include_dirs: None,
            exclude_dirs: None,
            include_ext: None,
            exclude_ext: Some(vec!["md".to_string()]),
            include_files: None,
            exclude_files: None,
            min_size: Some(0),
            max_size: None,
            respect_gitignore: true,
            tree_only: false,
        };
        let dir_structure =
            get_directory_structure(temp_dir2.path(), &gitignore, &ignored_dirs, &config_excl)?;
        process_files(&config_excl, &gitignore, &dir_structure, &ignored_dirs)?;
        let out_excl = fs::read_to_string(&config_excl.output)?;
        assert!(out_excl.contains("=== File: a.txt"));
        assert!(!out_excl.contains("=== File: b.md"));

        // Subcase 3: include_ext containing empty string to include files with no extension
        let temp_dir3 = setup_temp_dir();
        create_file(temp_dir3.path().join("a.txt"), "A")?;
        create_file(temp_dir3.path().join("b.md"), "B")?;
        create_file(temp_dir3.path().join("noext"), "NOEXT")?;

        let config_noext = Config {
            directory: temp_dir3.path().to_path_buf(),
            output: temp_dir3.path().join("out_noext.txt"),
            include_dirs: None,
            exclude_dirs: None,
            include_ext: Some(vec!["".to_string()]),
            exclude_ext: None,
            include_files: None,
            exclude_files: None,
            min_size: Some(0),
            max_size: None,
            respect_gitignore: true,
            tree_only: false,
        };
        let dir_structure =
            get_directory_structure(temp_dir3.path(), &gitignore, &ignored_dirs, &config_noext)?;
        process_files(&config_noext, &gitignore, &dir_structure, &ignored_dirs)?;
        let out_noext = fs::read_to_string(&config_noext.output)?;
        assert!(out_noext.contains("=== File: noext"));
        assert!(!out_noext.contains("=== File: b.md"));

        Ok(())
    }

    #[test]
    fn test_process_files_skips_output_file() -> io::Result<()> {
        let temp_dir = setup_temp_dir();
        // Create a file that has the same name as the output file to ensure it's skipped
        create_file(temp_dir.path().join("output.txt"), "SHOULD_NOT_BE_INCLUDED")?;
        create_file(temp_dir.path().join("keep.txt"), "KEEP")?;

        let config = Config {
            directory: temp_dir.path().to_path_buf(),
            output: temp_dir.path().join("output.txt"),
            include_dirs: None,
            exclude_dirs: None,
            include_ext: None,
            exclude_ext: None,
            include_files: None,
            exclude_files: None,
            min_size: Some(0),
            max_size: None,
            respect_gitignore: true,
            tree_only: false,
        };
        let ignored_dirs = ["node_modules"];
        let gitignore = create_gitignore_empty();
        let dir_structure =
            get_directory_structure(temp_dir.path(), &gitignore, &ignored_dirs, &config)?;
        process_files(&config, &gitignore, &dir_structure, &ignored_dirs)?;

        let output_content = fs::read_to_string(&config.output)?;
        // The pre-existing content "SHOULD_NOT_BE_INCLUDED" should NOT be treated as a processed file content
        assert!(!output_content.contains("SHOULD_NOT_BE_INCLUDED"));
        // But the other file should be present
        assert!(output_content.contains("=== File: keep.txt"));
        assert!(output_content.contains("KEEP"));
        Ok(())
    }

    #[test]
    fn test_get_directory_structure_with_include_dirs() -> io::Result<()> {
        let temp_dir = TempDir::new().unwrap();
        let root = temp_dir.path();
        // Create directories
        fs::create_dir_all(root.join("docs"))?;
        fs::create_dir_all(root.join("src"))?;
        create_file(root.join("docs/guide.md"), "Guide")?;
        create_file(root.join("src/main.rs"), "fn main() {}")?;

        let gitignore = Gitignore::empty();
        let ignored_dirs: Vec<&str> = vec![];
        let config = Config {
            directory: root.to_path_buf(),
            output: root.join("output.txt"),
            include_dirs: Some(vec!["docs".to_string()]),
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

        let result = get_directory_structure(root, &gitignore, &ignored_dirs, &config)?;
        assert!(result.contains("docs/"));
        assert!(result.contains("guide.md"));
        assert!(!result.contains("src/"));
        assert!(!result.contains("main.rs"));
        Ok(())
    }
}
