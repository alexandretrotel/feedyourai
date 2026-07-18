use arboard::Clipboard;

use eyre::{Result, eyre};

pub fn copy_to_clipboard(contents: &str) -> Result<()> {
    let mut clipboard = Clipboard::new().map_err(|e| eyre!("clipboard error: {e}"))?;
    clipboard
        .set_text(contents.to_owned())
        .map_err(|e| eyre!("clipboard error: {e}"))?;

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
