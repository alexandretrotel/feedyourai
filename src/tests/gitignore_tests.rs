#[cfg(test)]
mod tests {
    use crate::gitignore::build_gitignore;
    use crate::tests::common::{create_gitignore, setup_temp_dir};
    use std::io;

    #[test]
    fn test_build_gitignore_with_file() -> io::Result<()> {
        let temp_dir = setup_temp_dir();
        create_gitignore(temp_dir.path(), "*.log\ntemp/")?;

        let gitignore = build_gitignore(temp_dir.path(), true)?; // Enable test_mode for debugging
        let temp_file = temp_dir.path().join("temp/file.txt");
        let is_ignored = gitignore.matched(&temp_file, false).is_ignore();
        println!("Checking temp/file.txt: is_ignored = {}", is_ignored);
        assert!(is_ignored, "temp/file.txt should be ignored");
        assert!(
            gitignore
                .matched(temp_dir.path().join("test.log"), false)
                .is_ignore(),
            "test.log should be ignored"
        );
        assert!(
            !gitignore
                .matched(temp_dir.path().join("other.txt"), false)
                .is_ignore(),
            "other.txt should not be ignored"
        );
        Ok(())
    }

    #[test]
    fn test_build_gitignore_no_file() -> io::Result<()> {
        let temp_dir = setup_temp_dir();
        let gitignore = build_gitignore(temp_dir.path(), false)?;

        // Should include default ignored files like Cargo.lock
        assert!(
            gitignore
                .matched(temp_dir.path().join("Cargo.lock"), false)
                .is_ignore()
        );
        assert!(
            !gitignore
                .matched(temp_dir.path().join("src/main.rs"), false)
                .is_ignore()
        );
        Ok(())
    }

    #[test]
    fn test_build_gitignore_invalid_pattern() -> io::Result<()> {
        let temp_dir = setup_temp_dir();
        create_gitignore(temp_dir.path(), "**/[")?; // Invalid pattern
        let result = build_gitignore(temp_dir.path(), false);
        assert!(result.is_ok()); // Should fall back to empty gitignore
        Ok(())
    }
}
