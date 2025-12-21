#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use crate::{cli::create_commands, config::config_from_matches};

    #[test]
    fn test_default_config() {
        let args = create_commands().get_matches_from(vec!["fyai"]);
        let (config, _explicit) = config_from_matches(args).unwrap();

        assert_eq!(config.directory, PathBuf::from("."));
        assert_eq!(config.output, PathBuf::from("fyai.txt"));
        assert!(config.include_ext.is_none());
        assert!(config.exclude_ext.is_none());
        assert!(config.min_size.is_none());
        assert!(config.max_size.is_none());
        assert!(config.exclude_dirs.is_none()); // Check default
        assert!(!config.tree_only);
    }

    #[test]
    fn test_custom_directory_and_output() {
        let args = create_commands().get_matches_from(vec![
            "fyai",
            "--dir",
            "/path/to/dir",
            "--output",
            "custom.txt",
        ]);
        let (config, _explicit) = config_from_matches(args).unwrap();

        assert_eq!(config.directory, PathBuf::from("/path/to/dir"));
        assert_eq!(config.output, PathBuf::from("custom.txt"));
        assert!(config.include_ext.is_none());
        assert!(config.exclude_ext.is_none());
        assert!(config.min_size.is_none());
        assert!(config.max_size.is_none());
        assert!(config.exclude_dirs.is_none());
        assert!(!config.tree_only);
    }

    #[test]
    fn test_extensions_parsing() {
        let args =
            create_commands().get_matches_from(vec!["fyai", "--include-ext", "txt, md, pdf"]);
        let (config, _explicit) = config_from_matches(args).unwrap();

        assert_eq!(
            config.include_ext,
            Some(vec!["txt".to_string(), "md".to_string(), "pdf".to_string()])
        );
    }

    #[test]
    fn test_exclude_dirs_parsing() {
        let args = create_commands().get_matches_from(vec!["fyai", "--exclude-dirs", "src,tests"]);
        let (config, _explicit) = config_from_matches(args).unwrap();

        assert_eq!(
            config.exclude_dirs,
            Some(vec!["src".to_string(), "tests".to_string()])
        );
    }

    #[test]
    fn test_exclude_dirs_with_empty_and_spaces() {
        let args =
            create_commands().get_matches_from(vec!["fyai", "--exclude-dirs", "src,, tests ,docs"]);
        let (config, _explicit) = config_from_matches(args).unwrap();

        assert_eq!(
            config.exclude_dirs,
            Some(vec![
                "src".to_string(),
                "tests".to_string(),
                "docs".to_string()
            ])
        );
    }

    #[test]
    fn test_size_filters() {
        let args = create_commands().get_matches_from(vec![
            "fyai",
            "--min-size",
            "1000",
            "--max-size",
            "5000",
        ]);
        let (config, _explicit) = config_from_matches(args).unwrap();

        assert_eq!(config.min_size, Some(1000));
        assert_eq!(config.max_size, Some(5000));
    }

    #[test]
    fn test_invalid_min_size() {
        let args = create_commands().get_matches_from(vec!["fyai", "--min-size", "invalid"]);
        let result = config_from_matches(args);

        assert!(result.is_err());
        assert_eq!(result.unwrap_err().to_string(), "Invalid min-size");
    }

    #[test]
    fn test_invalid_max_size() {
        let args = create_commands().get_matches_from(vec!["fyai", "--max-size", "invalid"]);
        let result = config_from_matches(args);

        assert!(result.is_err());
        assert_eq!(result.unwrap_err().to_string(), "Invalid max-size");
    }

    #[test]
    fn test_extensions_with_empty_and_spaces() {
        let args =
            create_commands().get_matches_from(vec!["fyai", "--include-ext", "txt,, md ,pdf"]);
        let (config, _explicit) = config_from_matches(args).unwrap();

        assert_eq!(
            config.include_ext,
            Some(vec!["txt".to_string(), "md".to_string(), "pdf".to_string()])
        );
    }

    #[test]
    fn test_tree_only_flag() {
        let args = create_commands().get_matches_from(vec!["fyai", "--tree-only"]);
        let (config, _explicit) = config_from_matches(args).unwrap();

        assert!(config.tree_only);
    }
}
