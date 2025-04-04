use clap::{Arg, Command};
use clipboard::{ClipboardContext, ClipboardProvider};
use ignore::gitignore::{Gitignore, GitignoreBuilder};
use std::fs::{self, File};
use std::io::{self, Error, ErrorKind, Read, Write};
use std::path::Path;
use walkdir::WalkDir;

fn is_in_ignored_dir(path: &Path, ignored_dirs: &[&str]) -> bool {
    path.components().any(|comp| {
        if let Some(name) = comp.as_os_str().to_str() {
            ignored_dirs.contains(&name)
        } else {
            false
        }
    })
}

fn get_directory_structure(root: &Path, gitignore: &Gitignore, ignored_dirs: &[&str]) -> String {
    let mut structure = String::new();
    structure.push_str("=== Project Directory Structure ===\n\n");

    for entry in WalkDir::new(root).into_iter().filter_map(Result::ok) {
        let path = entry.path();
        
        // Skip if in ignored directory
        if is_in_ignored_dir(path, ignored_dirs) {
            continue;
        }

        // Skip if matched by gitignore
        let is_dir = path.is_dir();
        if gitignore.matched(path, is_dir).is_ignore() {
            continue;
        }

        let depth = entry.depth();
        let indent = "  ".repeat(depth);
        if let Some(name) = path.file_name() {
            let marker = if path.is_dir() { "/" } else { "" };
            structure.push_str(&format!(
                "{}{}{}\n",
                indent,
                name.to_string_lossy(),
                marker
            ));
        }
    }
    structure.push_str("\n");
    structure
}

fn main() -> io::Result<()> {
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
                .default_value("0"),
        )
        .arg(
            Arg::new("max_size")
                .short('m')
                .long("max-size")
                .value_name("BYTES")
                .help("Exclude files larger than this size in bytes")
                .default_value("1048576"),
        )
        .arg(
            Arg::new("test")
                .short('t')
                .long("test")
                .help("Enable test mode to display debugging information"),
        )
        .get_matches();

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
        println!(" - Min File Size: {} bytes", min_size.unwrap_or(0));
        println!(" - Max File Size: {} bytes", max_size.unwrap_or(1_048_576));
        println!(
            " - Included Extensions: {:?}",
            excluded_extensions.as_ref().map(|e| e.join(", "))
        );
    }

    let mut gitignore_builder = GitignoreBuilder::new(dir_path);
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

    let gitignore_path = Path::new(dir_path).join(".gitignore");
    if gitignore_path.exists() {
        gitignore_builder.add(gitignore_path);
    }
    let gitignore = gitignore_builder
        .build()
        .unwrap_or_else(|_| Gitignore::empty());

    let ignored_dirs = [
        "node_modules",
        ".git",
        ".svn",
        ".hg",
        ".idea",
        ".vscode",
        "build",
        "dist",
        "src-tauri"
    ];
    let mut output = File::create(output_file)?;

    // Write the directory structure to the output file
    let dir_structure = get_directory_structure(Path::new(dir_path), &gitignore, &ignored_dirs);
    write!(output, "{}", dir_structure)?;

    println!("Processing files in: {}", dir_path);

    for entry in WalkDir::new(dir_path).into_iter().filter_map(Result::ok) {
        let path = entry.path();

        if is_in_ignored_dir(path, &ignored_dirs) {
            if test_mode {
                println!("Skipping (ignored folder): {}", path.display());
            }
            continue;
        }

        let is_dir = path.is_dir();
        if gitignore.matched(path, is_dir).is_ignore() {
            if test_mode {
                println!("Skipping (gitignore): {}", path.display());
            }
            continue;
        }

        if is_dir {
            if test_mode {
                println!("Skipping directory: {}", path.display());
            }
            continue;
        }

        if path.is_file() {
            let metadata = fs::metadata(&path)?;
            let file_size = metadata.len();

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

            let ext = path
                .extension()
                .and_then(|e| e.to_str())
                .map(|e| e.to_lowercase());
            if let Some(ref excluded_exts) = excluded_extensions {
                if ext.is_some_and(|e| excluded_exts.contains(&e)) {
                    if test_mode {
                        println!("Skipping (excluded extension): {}", path.display());
                    }
                    continue;
                }
            }

            println!("Processing: {} ({} bytes)", path.display(), file_size);

            let mut file = File::open(&path)?;
            let mut contents = Vec::new();
            file.read_to_end(&mut contents)?;

            if let Ok(text) = String::from_utf8(contents) {
                writeln!(
                    output,
                    "\n=== File: {} ({} bytes) ===\n",
                    path.display(),
                    file_size
                )?;
                write!(output, "{}", text)?;
            } else if test_mode {
                println!("Skipping (not UTF-8): {}", path.display());
            }
        }
    }

    println!("Files combined successfully into {}", output_file);

    // Read the output file and copy to clipboard
    let mut output_contents = String::new();
    File::open(output_file)?.read_to_string(&mut output_contents)?;

    let mut clipboard: ClipboardContext = ClipboardProvider::new()
        .map_err(|e| io::Error::new(ErrorKind::Other, format!("Clipboard error: {}", e)))?;

    clipboard
        .set_contents(output_contents)
        .map_err(|e| io::Error::new(ErrorKind::Other, format!("Clipboard error: {}", e)))?;

    println!("Output copied to clipboard successfully!");

    Ok(())
}
