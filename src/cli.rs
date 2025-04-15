use clap::{Arg, Command};
use std::io;
use std::path::PathBuf;

/// Configuration derived from command-line arguments.
#[derive(Debug)]
pub struct Config {
    pub directory: PathBuf,
    pub output: PathBuf,
    pub extensions: Option<Vec<String>>,
    pub min_size: Option<u64>,
    pub max_size: Option<u64>,
    pub test_mode: bool,
}

/// Parses command-line arguments and returns a `Config` struct.
///
/// # Returns
/// - `Ok(Config)`: Parsed configuration.
/// - `Err(io::Error)`: If parsing fails (e.g., invalid input).
pub fn parse_args() -> io::Result<Config> {
    let matches = Command::new("FeedYourAI")
        .version("1.3.0")
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
                .default_value("feedyourai.txt"),
        )
        .arg(
            Arg::new("extensions")
                .short('e')
                .long("ext")
                .value_name("EXT")
                .help("Comma-separated list of file extensions to include (e.g., txt,md)"),
        )
        .arg(
            Arg::new("min_size")
                .short('n')
                .long("min-size")
                .value_name("BYTES")
                .help("Exclude files smaller than this size in bytes")
                .default_value("51200"),
        )
        .arg(
            Arg::new("max_size")
                .short('m')
                .long("max-size")
                .value_name("BYTES")
                .help("Exclude files larger than this size in bytes"),
        )
        .arg(
            clap::Arg::new("test")
                .short('t')
                .long("test")
                .action(clap::ArgAction::SetTrue)
                .help("Enable test mode"),
        )
        .get_matches();

    let directory = matches.get_one::<String>("directory").unwrap().into();
    let output = matches.get_one::<String>("output").unwrap().into();
    let extensions = matches.get_one::<String>("extensions").map(|ext| {
        ext.split(',')
            .map(|s| s.trim().to_lowercase())
            .collect::<Vec<_>>()
    });
    let min_size = matches
        .get_one::<String>("min_size")
        .and_then(|s| s.parse().ok());
    let max_size = matches
        .get_one::<String>("max_size")
        .and_then(|s| s.parse().ok());
    let test_mode = matches.get_flag("test");

    if test_mode {
        println!("DEBUG MODE ENABLED:");
        println!(" - Input Directory: {:?}", directory);
        println!(" - Output File: {:?}", output);
        println!(" - Min File Size: {} bytes", min_size.unwrap_or(0));
        println!(" - Max File Size: {} bytes", max_size.unwrap_or(u64::MAX));
        println!(
            " - Included Extensions: {:?}",
            extensions.as_ref().map(|e| e.join(", "))
        );
    }

    Ok(Config {
        directory,
        output,
        extensions,
        min_size,
        max_size,
        test_mode,
    })
}
