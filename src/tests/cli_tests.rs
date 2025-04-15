#[cfg(test)]
mod tests {
    use crate::cli::Config;
    use std::io;
    use std::path::PathBuf;

    // Simulate CLI arguments for testing
    fn simulate_args(args: &[&str]) -> io::Result<Config> {
        let matches = clap::Command::new("FeedYourAI")
            .version("1.2.4")
            .arg(
                clap::Arg::new("directory")
                    .short('d')
                    .long("dir")
                    .value_name("DIR")
                    .default_value("."),
            )
            .arg(
                clap::Arg::new("output")
                    .short('o')
                    .long("output")
                    .value_name("FILE")
                    .default_value("feedyourai.txt"),
            )
            .arg(
                clap::Arg::new("extensions")
                    .short('e')
                    .long("ext")
                    .value_name("EXT"),
            )
            .arg(
                clap::Arg::new("min_size")
                    .short('n')
                    .long("min-size")
                    .value_name("BYTES")
                    .default_value("51200"),
            )
            .arg(
                clap::Arg::new("max_size")
                    .short('m')
                    .long("max-size")
                    .value_name("BYTES"),
            )
            .arg(
                clap::Arg::new("test")
                    .short('t')
                    .long("test")
                    .action(clap::ArgAction::SetTrue),
            )
            .try_get_matches_from(args)
            .map_err(|e| io::Error::new(io::ErrorKind::InvalidInput, e.to_string()))?;

        Ok(Config {
            directory: matches.get_one::<String>("directory").unwrap().into(),
            output: matches.get_one::<String>("output").unwrap().into(),
            extensions: matches.get_one::<String>("extensions").and_then(|ext| {
                if ext.is_empty() {
                    Some(vec![])
                } else {
                    Some(
                        ext.split(',')
                            .map(|s| s.trim().to_lowercase())
                            .collect::<Vec<_>>(),
                    )
                }
            }),
            min_size: matches
                .get_one::<String>("min_size")
                .and_then(|s| s.parse().ok()),
            max_size: matches
                .get_one::<String>("max_size")
                .and_then(|s| s.parse().ok()),
            test_mode: matches.get_flag("test"),
        })
    }

    #[test]
    fn test_default_args() -> io::Result<()> {
        let config = simulate_args(&["fyai"])?;
        assert_eq!(config.directory, PathBuf::from("."));
        assert_eq!(config.output, PathBuf::from("feedyourai.txt"));
        assert_eq!(config.min_size, Some(51200));
        assert_eq!(config.max_size, None);
        assert_eq!(config.extensions, None);
        assert_eq!(config.test_mode, false);
        Ok(())
    }

    #[test]
    fn test_custom_args() -> io::Result<()> {
        let config = simulate_args(&[
            "fyai",
            "-d",
            "/path/to/dir",
            "-o",
            "output.txt",
            "-e",
            "txt,md",
            "-n",
            "1000",
            "-m",
            "100000",
            "-t",
        ])?;
        assert_eq!(config.directory, PathBuf::from("/path/to/dir"));
        assert_eq!(config.output, PathBuf::from("output.txt"));
        assert_eq!(
            config.extensions,
            Some(vec!["txt".to_string(), "md".to_string()])
        );
        assert_eq!(config.min_size, Some(1000));
        assert_eq!(config.max_size, Some(100000));
        assert_eq!(config.test_mode, true);
        Ok(())
    }

    #[test]
    fn test_empty_extensions() -> io::Result<()> {
        let config = simulate_args(&["fyai", "-e", ""])?;
        assert_eq!(config.extensions, Some(vec![]));
        Ok(())
    }
}
