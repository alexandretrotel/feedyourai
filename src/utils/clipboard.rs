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

pub(crate) fn should_ignore_clipboard_error() -> bool {
    if std::env::var_os("CI").is_some() {
        return true;
    }
    if cfg!(target_os = "linux") {
        let has_display = std::env::var_os("DISPLAY").is_some()
            || std::env::var_os("WAYLAND_DISPLAY").is_some()
            || std::env::var_os("SWAYSOCK").is_some();
        return !has_display;
    }
    false
}
