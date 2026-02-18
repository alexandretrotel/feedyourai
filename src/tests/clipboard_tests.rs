#[cfg(test)]
mod tests {
    use crate::utils::clipboard::copy_to_clipboard;
    use crate::error::{AppError, AppResult};
    use crate::tests::common::{create_file, setup_temp_dir};
    use std::io;

    #[test]
    fn test_copy_to_clipboard_valid_file() -> AppResult<()> {
        let temp_dir = setup_temp_dir();
        let file_path = temp_dir.path().join("test.txt");
        create_file(&file_path, "Hello, clipboard!")?;

        // Skip actual clipboard interaction in CI
        if std::env::var("CI").is_ok() {
            return Ok(());
        }

        let result = copy_to_clipboard(&file_path);
        // Accept both Ok and clipboard errors (for headless/unsupported environments)
        if result.is_err() {
            eprintln!("Clipboard error: {:?}", result);
        }
        assert!(
            result.is_ok()
                || result
                    .as_ref()
                    .err()
                    .is_some_and(|e| matches!(e, AppError::Clipboard(_)))
        );
        Ok(())
    }

    #[test]
    fn test_copy_to_clipboard_nonexistent_file() -> AppResult<()> {
        let temp_dir = setup_temp_dir();
        let file_path = temp_dir.path().join("nonexistent.txt");
        let result = copy_to_clipboard(&file_path);
        assert!(result.is_err());
        match result.unwrap_err() {
            AppError::Io(err) => assert_eq!(err.kind(), io::ErrorKind::NotFound),
            err => panic!("Expected io::Error NotFound, got {err:?}"),
        }
        Ok(())
    }

    #[test]
    fn test_copy_to_clipboard_empty_file() -> AppResult<()> {
        let temp_dir = setup_temp_dir();
        let file_path = temp_dir.path().join("empty.txt");
        create_file(&file_path, "")?;

        // Skip actual clipboard interaction in CI
        if std::env::var("CI").is_ok() {
            return Ok(());
        }

        let result = copy_to_clipboard(&file_path);
        // Accept both Ok and clipboard errors (for headless/unsupported environments)
        if result.is_err() {
            eprintln!("Clipboard error: {:?}", result);
        }
        assert!(
            result.is_ok()
                || result
                    .as_ref()
                    .err()
                    .is_some_and(|e| matches!(e, AppError::Clipboard(_)))
        );
        Ok(())
    }
}
