#[cfg(test)]
mod tests {
    use crate::cli::Config;
    use crate::file_processing::{get_directory_structure, is_in_ignored_dir, process_files};
    use crate::tests::common::{create_file, setup_temp_dir};
    use ignore::gitignore::Gitignore;
    use std::fs;
    use std::io::{self, Write};
    use std::path::{Path, PathBuf};

    fn create_gitignore_empty() -> Gitignore {
        Gitignore::empty()
    }

    #[test]
    fn test_is_in_ignored_dir() {
        let path = PathBuf::from("node_modules/test.txt");
        let ignored_dirs = ["node_modules", ".git"];
        assert!(is_in_ignored_dir(&path, &ignored_dirs));

        let path = PathBuf::from("src/test.txt");
        assert!(!is_in_ignored_dir(&path, &ignored_dirs));
    }

    #[test]
    fn test_path_not_in_ignored_dir() {
        let path = Path::new("/home/user/project/src/main.rs");
        let ignored_dirs = vec![".git", "node_modules"];
        assert!(!is_in_ignored_dir(path, &ignored_dirs));
    }

    #[test]
    fn test_empty_ignored_dirs() {
        let path = Path::new("/home/user/.git/config");
        let ignored_dirs: Vec<&str> = vec![];
        assert!(!is_in_ignored_dir(path, &ignored_dirs));
    }

    #[test]
    fn test_root_path() {
        let path = Path::new("/");
        let ignored_dirs = vec![".git", "node_modules"];
        assert!(!is_in_ignored_dir(path, &ignored_dirs));
    }

    #[test]
    fn test_single_component_path() {
        let path = Path::new(".git");
        let ignored_dirs = vec![".git", "node_modules"];
        assert!(is_in_ignored_dir(path, &ignored_dirs));
    }

    #[test]
    fn test_path_with_similar_prefix() {
        let path = Path::new("/home/user/gitlab/project");
        let ignored_dirs = vec![".git", "node_modules"];
        assert!(!is_in_ignored_dir(path, &ignored_dirs));
    }

    #[test]
    fn test_case_sensitivity() {
        let path = Path::new("/home/user/NODE_MODULES/cache");
        let ignored_dirs = vec!["node_modules"];
        assert!(is_in_ignored_dir(path, &ignored_dirs));
    }

    #[test]
    fn test_empty_path() {
        let path = Path::new("");
        let ignored_dirs = vec![".git", "node_modules"];
        assert!(!is_in_ignored_dir(path, &ignored_dirs));
    }

    #[test]
    fn test_get_directory_structure() -> io::Result<()> {
        let temp_dir = setup_temp_dir();
        create_file(temp_dir.path().join("file1.txt"), "Content 1")?;
        fs::create_dir(temp_dir.path().join("subdir"))?;
        create_file(temp_dir.path().join("subdir/file2.txt"), "Content 2")?;

        let ignored_dirs = ["node_modules"];
        let gitignore = create_gitignore_empty();
        let structure = get_directory_structure(temp_dir.path(), &gitignore, &ignored_dirs)?;

        assert!(structure.contains("=== Project Directory Structure ==="));
        assert!(structure.contains("file1.txt"));
        assert!(structure.contains("subdir/"));
        assert!(structure.contains("file2.txt"));
        Ok(())
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
            test_mode: false,
        };

        let ignored_dirs = ["node_modules"];
        let gitignore = create_gitignore_empty();
        let dir_structure = get_directory_structure(temp_dir.path(), &gitignore, &ignored_dirs)?;
        process_files(&config, &gitignore, &ignored_dirs, &dir_structure)?;

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
            test_mode: false,
        };

        let ignored_dirs = ["node_modules"];
        let gitignore = create_gitignore_empty();
        let dir_structure = get_directory_structure(temp_dir.path(), &gitignore, &ignored_dirs)?;
        process_files(&config, &gitignore, &ignored_dirs, &dir_structure)?;

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
            test_mode: true,
        };

        let ignored_dirs = ["node_modules"];
        let gitignore = create_gitignore_empty();
        let dir_structure = get_directory_structure(temp_dir.path(), &gitignore, &ignored_dirs)?;
        process_files(&config, &gitignore, &ignored_dirs, &dir_structure)?;

        // Output should not include non-UTF-8 file content
        let output_content = fs::read_to_string(&config.output)?;
        assert!(
            !output_content.contains("=== File: non_utf8.bin"),
            "Output contains non_utf8.bin header"
        );
        Ok(())
    }
}
