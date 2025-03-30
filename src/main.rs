use clap::{Arg, Command};
use ignore::gitignore::{Gitignore, GitignoreBuilder};
use std::fs::{self, File};
use std::io::{self, Error, ErrorKind, Read, Write};
use std::path::Path;
use walkdir::WalkDir;

fn main() -> io::Result<()> {
    // Set up command line argument parsing
    let matches = Command::new("FeedYourAI")
        .version("1.0.0")
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
                .default_value("0"), // 0 bytes default
        )
        .arg(
            Arg::new("max_size")
                .short('m')
                .long("max-size")
                .value_name("BYTES")
                .help("Exclude files larger than this size in bytes")
                .default_value("1048576"), // 1MB default
        )
        .arg(
            Arg::new("test")
                .short('t')
                .long("test")
                .help("Enable test mode to display debugging information"),
        )
        .get_matches();

    // Get the values from the command line arguments
    let dir_path: &String = matches.get_one("directory").unwrap();
    let output_file: &String = matches.get_one("output").unwrap();
    let excluded_extensions: Option<Vec<String>> = matches
        .get_one::<String>("extensions")
        .map(|ext| ext.split(',').map(|s| s.trim().to_lowercase()).collect());
    let min_size: Option<u64> = matches
        .get_one::<String>("min_size")
        .and_then(|s| s.parse().ok());
    let max_size: Option<u64> = matches
        .get_one::<String>("max_size")
        .and_then(|s| s.parse().ok());
    let test_mode = matches.contains_id("test");

    if test_mode {
        println!("DEBUG MODE ENABLED:");
        println!(" - Input Directory: {}", dir_path);
        println!(" - Output File: {}", output_file);
        println!(
            " - Min File Size: {} bytes",
            min_size.map_or("None".to_string(), |v| v.to_string())
        );
        println!(
            " - Max File Size: {} bytes",
            max_size.map_or("None".to_string(), |v| v.to_string())
        );
        println!(
            " - Included Extensions: {:?}",
            excluded_extensions.as_ref().map(|e| e.join(", "))
        );
    }

    // Build gitignore patterns
    let mut gitignore_builder = GitignoreBuilder::new(dir_path);
    gitignore_builder
        .add_line(None, "node_modules/")
        .map_err(|e| Error::new(ErrorKind::Other, e))?; // Default ignore for node_modules

    // Add common lock files and system files to the ignore list
    let ignored_files = [
        "bun.lock",
        "package-lock.json",
        "yarn.lock",
        "pnpm-lock.yaml",
        "Cargo.lock",
        ".DS_Store",
    ];
    for ignored in &ignored_files {
        gitignore_builder
            .add_line(None, ignored)
            .map_err(|e| Error::new(ErrorKind::Other, e))?;
    }

    // Check for .gitignore in the specified directory and add its rules
    let gitignore_path = Path::new(dir_path).join(".gitignore");
    if gitignore_path.exists() {
        gitignore_builder.add(gitignore_path);
    }
    let gitignore = gitignore_builder
        .build()
        .unwrap_or_else(|_| Gitignore::empty());

    // Ensure the output file is writable
    let mut output = File::create(output_file)?;

    // Iterate over the directory entries
    for entry in WalkDir::new(dir_path).into_iter().filter_map(Result::ok) {
        let path = entry.path();

        // Skip if path matches gitignore patterns
        let is_dir = path.is_dir();
        if gitignore.matched(path, is_dir).is_ignore() {
            if test_mode {
                println!("Skipping (gitignore): {}", path.display());
            }
            continue;
        }

        // Skip if the path is a directory and test mode is enabled
        if is_dir {
            if test_mode {
                println!("Skipping directory: {}", path.display());
            }
            continue;
        }

        // Check if the path is a file
        if path.is_file() {
            let metadata = fs::metadata(&path)?;
            let file_size = metadata.len();

            // Check file size against min and max size
            if let Some(min) = min_size {
                if file_size < min {
                    if test_mode {
                        println!(
                            "Skipping (too small): {} ({} bytes)",
                            path.display(),
                            file_size
                        );
                    }
                    continue;
                }
            }
            if let Some(max) = max_size {
                if file_size > max {
                    if test_mode {
                        println!(
                            "Skipping (too large): {} ({} bytes)",
                            path.display(),
                            file_size
                        );
                    }
                    continue;
                }
            }

            // Check file extension if specified
            let ext = path
                .extension()
                .and_then(|e| e.to_str())
                .map(|e| e.to_lowercase());

            // Check if the file extension is in the excluded list
            if let Some(ref excluded_exts) = excluded_extensions {
                if ext.is_some_and(|e| excluded_exts.contains(&e)) {
                    if test_mode {
                        println!("Skipping (excluded extension): {}", path.display());
                    }
                    continue; // Skip files with excluded extensions
                }
            }

            // If test mode is enabled, print the file being processed
            if test_mode {
                println!("Processing: {} ({} bytes)", path.display(), file_size);
            }

            // Read the file as binary and check if it's UTF-8
            let mut file = File::open(&path)?;
            let mut contents = Vec::new();
            file.read_to_end(&mut contents)?;

            if let Ok(text) = String::from_utf8(contents) {
                // Write the file name and size to the output file
                let filename = path
                    .file_name()
                    .and_then(|n| n.to_str())
                    .unwrap_or("unknown");

                writeln!(
                    output,
                    "\n=== File: {} ({} bytes) ===\n",
                    filename, file_size
                )?;
                write!(output, "{}", text)?;
            } else {
                if test_mode {
                    println!("Skipping (not UTF-8): {}", path.display());
                }
            }
        }
    }

    println!("Files combined successfully into {}", output_file);
    Ok(())
}
