use clap::{Arg, Command};

use crate::config::{Config, config_from_matches};

pub fn create_commands() -> Command {
    Command::new("fyai")
        .version(env!("CARGO_PKG_VERSION"))
        .about("A tool to combine text files for AI processing with flexible filtering options.\n\nCONFIG FILE SUPPORT:\n  - You can specify options in a config file (YAML format).\n  - Local config: ./fyai.yaml (used if present in current directory)\n  - Global config: ~/.fyai/config.yaml (used if no local config found)\n  - CLI options override config file values.\n  - See README for details and examples.")
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

/// Parses command-line arguments and returns a `Config` struct.
pub fn parse_args() -> std::io::Result<Config> {
    let matches = create_commands().get_matches();
    config_from_matches(matches)
}
