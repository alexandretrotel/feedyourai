use arboard::Clipboard;
use std::fs::File;
use std::io::Read;
use std::path::Path;

use crate::errors::{AppError, AppResult};

pub fn copy_to_clipboard(output_path: &Path) -> AppResult<()> {
    let mut output_contents = String::new();
    File::open(output_path)?.read_to_string(&mut output_contents)?;

    let mut clipboard = Clipboard::new().map_err(|e| AppError::Clipboard(e.to_string()))?;
    clipboard
        .set_text(output_contents)
        .map_err(|e| AppError::Clipboard(e.to_string()))?;

    Ok(())
}
