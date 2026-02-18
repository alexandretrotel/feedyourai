use crate::config::Config;
use crate::constants::{IGNORED_DIRS, IGNORED_FILES};
use crate::errors::{AppError, AppResult};
use crate::scanner::{get_directory_structure, process_files};
use crate::utils::clipboard::{copy_to_clipboard, should_ignore_clipboard_error};

pub mod init;

pub fn run(config: Config) -> AppResult<()> {
    let dir_structure =
        get_directory_structure(&config.directory, IGNORED_FILES, IGNORED_DIRS, &config)?;

    if config.tree_only {
        std::fs::write(&config.output, &dir_structure)?;
        println!("Project tree written to {}", config.output.display());
    } else {
        process_files(&config, &dir_structure, IGNORED_FILES, IGNORED_DIRS)?;
        let mut copied = true;
        if let Err(err) = copy_to_clipboard(&config.output) {
            if matches!(err, AppError::Clipboard(_)) && should_ignore_clipboard_error() {
                copied = false;
                eprintln!("Warning: clipboard unavailable; skipping copy. {}", err);
            } else {
                return Err(err);
            }
        }
        println!(
            "Files combined successfully into {}",
            config.output.display()
        );
        if copied {
            println!("Output copied to clipboard successfully!");
        }
    }
    Ok(())
}
