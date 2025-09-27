use clap::{Arg, Command};
use std::io;
use std::path::PathBuf;

#[derive(Debug, PartialEq, Clone)]
pub struct Config {
    pub directory: PathBuf,
    pub output: PathBuf,
    pub extensions: Option<Vec<String>>,
    pub min_size: Option<u64>,
    pub max_size: Option<u64>,
    pub exclude_dirs: Option<Vec<String>>,
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
    let extensions = matches.get_one::<String>("extensions").map(|ext| {
        ext.split(',')
            .map(|s| s.trim().to_lowercase())
            .filter(|s| !s.is_empty())
            .collect::<Vec<_>>()
    });
    let min_size = matches
        .get_one::<String>("min_size")
        .map(|s| {
            s.parse::<u64>().map_err(|_| {
                io::Error::new(io::ErrorKind::InvalidInput, "Invalid value for min_size")
            })
        })
        .transpose()?;
    let max_size = matches
        .get_one::<String>("max_size")
        .map(|s| {
            s.parse::<u64>().map_err(|_| {
                io::Error::new(io::ErrorKind::InvalidInput, "Invalid value for max_size")
            })
        })
        .transpose()?;
    let exclude_dirs = matches.get_one::<String>("exclude_dirs").map(|dirs| {
        dirs.split(',')
            .map(|s| s.trim().to_lowercase())
            .filter(|s| !s.is_empty())
            .collect::<Vec<_>>()
    });
    let tree_only = matches.get_flag("tree_only");

    Ok(Config {
        directory,
        output,
        extensions,
        min_size,
        max_size,
        exclude_dirs,
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
        .about("A tool to combine text files for AI processing with filtering options.")
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
            Arg::new("extensions")
                .short('e')
                .long("ext")
                .value_name("EXT")
                .help("Comma-separated list of file extensions to exclude (e.g., txt,md)"),
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
            Arg::new("exclude_dirs")
                .short('x')
                .long("exclude-dirs")
                .value_name("DIRS")
                .help("Comma-separated list of directories to exclude (e.g., node_modules,dist)"),
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
                .help("Enable test mode"),
        )
}
