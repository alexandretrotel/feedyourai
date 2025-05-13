#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use crate::cli::{config_from_matches, create_commands};

    #[test]
    fn test_default_config() {
        let args = create_commands().get_matches_from(vec!["feedyourai"]);
        let config = config_from_matches(args).unwrap();

        assert_eq!(config.directory, PathBuf::from("."));
        assert_eq!(config.output, PathBuf::from("feedyourai.txt"));
        assert_eq!(config.extensions, None);
        assert_eq!(config.min_size, None);
        assert_eq!(config.max_size, None);
        assert_eq!(config.exclude_dirs, None); // Check default
    }

    #[test]
    fn test_custom_directory_and_output() {
        let args = create_commands().get_matches_from(vec![
            "feedyourai",
            "--dir",
            "/path/to/dir",
            "--output",
            "custom.txt",
        ]);
        let config = config_from_matches(args).unwrap();

        assert_eq!(config.directory, PathBuf::from("/path/to/dir"));
        assert_eq!(config.output, PathBuf::from("custom.txt"));
        assert_eq!(config.extensions, None);
        assert_eq!(config.min_size, None);
        assert_eq!(config.max_size, None);
        assert_eq!(config.exclude_dirs, None);
    }

    #[test]
    fn test_extensions_parsing() {
        let args = create_commands().get_matches_from(vec!["feedyourai", "--ext", "txt, md, pdf"]);
        let config = config_from_matches(args).unwrap();

        assert_eq!(
            config.extensions,
            Some(vec!["txt".to_string(), "md".to_string(), "pdf".to_string()])
        );
    }

    #[test]
    fn test_exclude_dirs_parsing() {
        let args =
            create_commands().get_matches_from(vec!["feedyourai", "--exclude-dirs", "src,tests"]);
        let config = config_from_matches(args).unwrap();

        assert_eq!(
            config.exclude_dirs,
            Some(vec!["src".to_string(), "tests".to_string()])
        );
    }

    #[test]
    fn test_exclude_dirs_with_empty_and_spaces() {
        let args = create_commands().get_matches_from(vec![
            "feedyourai",
            "--exclude-dirs",
            "src,, tests ,docs",
        ]);
        let config = config_from_matches(args).unwrap();

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
            "feedyourai",
            "--min-size",
            "1000",
            "--max-size",
            "5000",
        ]);
        let config = config_from_matches(args).unwrap();

        assert_eq!(config.min_size, Some(1000));
        assert_eq!(config.max_size, Some(5000));
    }

    #[test]
    fn test_invalid_min_size() {
        let args = create_commands().get_matches_from(vec!["feedyourai", "--min-size", "invalid"]);
        let result = config_from_matches(args);

        assert!(result.is_err());
        assert_eq!(
            result.unwrap_err().to_string(),
            "Invalid value for min_size"
        );
    }

    #[test]
    fn test_invalid_max_size() {
        let args = create_commands().get_matches_from(vec!["feedyourai", "--max-size", "invalid"]);
        let result = config_from_matches(args);

        assert!(result.is_err());
        assert_eq!(
            result.unwrap_err().to_string(),
            "Invalid value for max_size"
        );
    }

    #[test]
    fn test_extensions_with_empty_and_spaces() {
        let args = create_commands().get_matches_from(vec!["feedyourai", "--ext", "txt,, md ,pdf"]);
        let config = config_from_matches(args).unwrap();

        assert_eq!(
            config.extensions,
            Some(vec!["txt".to_string(), "md".to_string(), "pdf".to_string()])
        );
    }
}
