use clipboard::{ClipboardContext, ClipboardProvider};
use std::fs::File;
use std::io::{self, Error, ErrorKind, Read};
use std::path::Path;

/// Copies the contents of the specified file to the system clipboard.
pub fn copy_to_clipboard(output_path: &Path) -> io::Result<()> {
    let mut output_contents = String::new();
    File::open(output_path)?.read_to_string(&mut output_contents)?;

    let mut clipboard: ClipboardContext = ClipboardProvider::new()
        .map_err(|e| Error::new(ErrorKind::Other, format!("Clipboard error: {}", e)))?;

    clipboard
        .set_contents(output_contents)
        .map_err(|e| Error::new(ErrorKind::Other, format!("Clipboard error: {}", e)))?;

    Ok(())
}
