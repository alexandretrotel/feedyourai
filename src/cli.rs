use clap::{Arg, Command};
use std::io;
use std::path::PathBuf;

#[derive(Debug, PartialEq, Clone)]
pub struct Config {
    pub directory: PathBuf,
    pub output: PathBuf,
    pub include_dirs: Option<Vec<String>>,
    pub exclude_dirs: Option<Vec<String>>,
    pub include_ext: Option<Vec<String>>,
    pub exclude_ext: Option<Vec<String>>,
    pub include_files: Option<Vec<String>>,
    pub exclude_files: Option<Vec<String>>,
    pub min_size: Option<u64>,
    pub max_size: Option<u64>,
    pub respect_gitignore: bool,
    pub tree_only: bool,
}

pub fn config_from_matches(matches: clap::ArgMatches) -> io::Result<Config> {
    let directory = matches
        .get_one::<String>("directory")
        .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidInput, "Missing directory"))?
        .into();
    let output = matches
        .get_one::<String>("output")
        .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidInput, "Missing output"))?
        .into();

    let include_dirs = matches.get_one::<String>("include_dirs").map(|dirs| {
        dirs.split(',')
            .map(|s| s.trim().to_lowercase())
            .filter(|s| !s.is_empty())
            .collect::<Vec<_>>()
    });

    let exclude_dirs = matches.get_one::<String>("exclude_dirs").map(|dirs| {
        dirs.split(',')
            .map(|s| s.trim().to_lowercase())
            .filter(|s| !s.is_empty())
            .collect::<Vec<_>>()
    });

    let include_ext = matches.get_one::<String>("include_ext").map(|ext| {
        ext.split(',')
            .map(|s| s.trim().to_lowercase())
            .filter(|s| !s.is_empty())
            .collect::<Vec<_>>()
    });

    let exclude_ext = matches.get_one::<String>("exclude_ext").map(|ext| {
        ext.split(',')
            .map(|s| s.trim().to_lowercase())
            .filter(|s| !s.is_empty())
            .collect::<Vec<_>>()
    });

    let include_files = matches.get_one::<String>("include_files").map(|files| {
        files
            .split(',')
            .map(|s| s.trim().to_lowercase())
            .filter(|s| !s.is_empty())
            .collect::<Vec<_>>()
    });

    let exclude_files = matches.get_one::<String>("exclude_files").map(|files| {
        files
            .split(',')
            .map(|s| s.trim().to_lowercase())
            .filter(|s| !s.is_empty())
            .collect::<Vec<_>>()
    });

    let min_size = matches
        .get_one::<String>("min_size")
        .map(|s| {
            s.parse::<u64>()
                .map_err(|_| io::Error::new(io::ErrorKind::InvalidInput, "Invalid min-size"))
        })
        .transpose()?;
    let max_size = matches
        .get_one::<String>("max_size")
        .map(|s| {
            s.parse::<u64>()
                .map_err(|_| io::Error::new(io::ErrorKind::InvalidInput, "Invalid max-size"))
        })
        .transpose()?;
    let respect_gitignore = matches
        .get_one::<String>("respect_gitignore")
        .map(|s| s == "true" || s == "1")
        .unwrap_or(true);

    let tree_only = matches.get_flag("tree_only");

    Ok(Config {
        directory,
        output,
        include_dirs,
        exclude_dirs,
        include_ext,
        exclude_ext,
        include_files,
        exclude_files,
        min_size,
        max_size,
        respect_gitignore,
        tree_only,
    })
}

/// Parses command-line arguments and returns a `Config` struct.
pub fn parse_args() -> io::Result<Config> {
    let matches = create_commands().get_matches();
    config_from_matches(matches)
}

pub fn create_commands() -> Command {
    Command::new("fyai")
        .version(env!("CARGO_PKG_VERSION"))
        .about("A tool to combine text files for AI processing with flexible filtering options.")
        .arg(
            Arg::new("directory")
                .short('d')
                .long("dir")
                .value_name("DIR")
                .help("Sets the input directory")
                .default_value("."),
        )
        .arg(
            Arg::new("output")
                .short('o')
                .long("output")
                .value_name("FILE")
                .help("Sets the output file")
                .default_value("fyai.txt"),
        )
        .arg(
            Arg::new("include_dirs")
                .long("include-dirs")
                .value_name("DIRS")
                .help("Comma-separated list of directories to include (e.g., src,docs)"),
        )
        .arg(
            Arg::new("exclude_dirs")
                .long("exclude-dirs")
                .value_name("DIRS")
                .help("Comma-separated list of directories to exclude (e.g., node_modules,dist)"),
        )
        .arg(
            Arg::new("include_ext")
                .long("include-ext")
                .value_name("EXT")
                .help("Comma-separated list of file extensions to include (e.g., txt,md)"),
        )
        .arg(
            Arg::new("exclude_ext")
                .long("exclude-ext")
                .value_name("EXT")
                .help("Comma-separated list of file extensions to exclude (e.g., log,tmp)"),
        )
        .arg(
            Arg::new("include_files")
                .long("include-files")
                .value_name("FILES")
                .help("Comma-separated list of file names to include (e.g., README.md,main.rs)"),
        )
        .arg(
            Arg::new("exclude_files")
                .long("exclude-files")
                .value_name("FILES")
                .help("Comma-separated list of file names to exclude (e.g., LICENSE,config.json)"),
        )
        .arg(
            Arg::new("respect_gitignore")
                .long("respect-gitignore")
                .value_name("BOOL")
                .help("Whether to respect .gitignore rules (true/false) [default: true]"),
        )
        .arg(
            Arg::new("min_size")
                .short('n')
                .long("min-size")
                .value_name("BYTES")
                .help("Exclude files smaller than this size in bytes"),
        )
        .arg(
            Arg::new("max_size")
                .short('m')
                .long("max-size")
                .value_name("BYTES")
                .help("Exclude files larger than this size in bytes"),
        )
        .arg(
            clap::Arg::new("tree_only")
                .long("tree-only")
                .action(clap::ArgAction::SetTrue)
                .help("Only output the project directory tree, no file contents"),
        )
        .arg(
            clap::Arg::new("test")
                .short('t')
                .long("test")
                .action(clap::ArgAction::SetTrue)
                .help("Run in test mode"),
        )
}
