use clap::{Arg, Command};
use ignore::gitignore::{Gitignore, GitignoreBuilder};
use std::fs::{self, File};
use std::io::{self, Error, ErrorKind, Read, Write};
use std::path::Path;

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
                .help("Exclude files smaller than this size in bytes (default: 51200)"),
        )
        .arg(
            Arg::new("max_size")
                .short('m')
                .long("max-size")
                .value_name("BYTES")
                .help("Exclude files larger than this size in bytes"),
        )
        .get_matches();

    // Get the values from the command line arguments
    let dir_path: &String = matches.get_one("directory").unwrap();
    let output_file: &String = matches.get_one("output").unwrap();
    let extensions: Option<Vec<String>> = matches
        .get_one::<String>("extensions")
        .map(|ext| ext.split(',').map(|s| s.trim().to_string()).collect());
    let min_size: u64 = matches
        .get_one::<String>("min_size")
        .and_then(|s| s.parse().ok())
        .unwrap_or(51_200); // 50KB default
    let max_size: Option<u64> = matches
        .get_one::<String>("max_size")
        .and_then(|s| s.parse().ok());

    // Build gitignore patterns
    let mut gitignore_builder = GitignoreBuilder::new(dir_path);
    gitignore_builder
        .add_line(None, "node_modules")
        .map_err(|e| Error::new(ErrorKind::Other, e))?; // Default ignore for node_modules

    // Add common lock files to the ignore list
    let lock_files = ["bun.lock", "package-lock.json", "yarn.lock", "pnpm-lock.yaml", "Cargo.lock"];
    for lock_file in &lock_files {
        gitignore_builder.add_line(None, lock_file)
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
    let entries = fs::read_dir(dir_path)?;

    // Iterate over the directory entries
    for entry in entries {
        let entry = entry?;
        let path = entry.path();

        // Skip if path matches gitignore patterns
        if gitignore.matched(&path, path.is_dir()).is_ignore() {
            continue;
        }

        // Check if the path is a file
        if path.is_file() {
            let metadata = fs::metadata(&path)?;
            let file_size = metadata.len();

            // Check file size against min and max size
            if file_size < min_size {
                continue;
            }
            if let Some(max) = max_size {
                if file_size > max {
                    continue;
                }
            }

            // Check file extension if specified
            let ext = path
                .extension()
                .and_then(|e| e.to_str())
                .map(|e| e.to_lowercase());

            // If extensions are specified, check if the file's extension is allowed
            if let Some(ref allowed_exts) = extensions {
                if !ext.is_some_and(|e| allowed_exts.contains(&e)) {
                    continue;
                }
            }

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
            let mut file = File::open(&path)?;
            let mut contents = String::new();
            file.read_to_string(&mut contents)?;
            write!(output, "{}", contents)?;
        }
    }

    println!("Files combined successfully into {}", output_file);
    Ok(())
}
