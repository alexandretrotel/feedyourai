#[cfg(test)]
mod tests {
    use std::{fs, io};
    use tempfile::TempDir;

    use crate::gitignore::build_gitignore;

    #[test]
    fn test_build_gitignore_new_file() -> io::Result<()> {
        let temp_dir = TempDir::new()?;
        let gitignore = build_gitignore(temp_dir.path(), false)?;

        // Verify .gitignore file was created
        let gitignore_path = temp_dir.path().join(".gitignore");
        assert!(gitignore_path.exists());

        // Read and verify content
        let content = fs::read_to_string(&gitignore_path)?;
        let expected_files = [
            "bun.lock",
            "package-lock.json",
            "yarn.lock",
            "pnpm-lock.yaml",
            "Cargo.lock",
            ".DS_Store",
            "uv.lock",
        ];
        let expected_dirs = ["node_modules/**", "target/**", "dist/**", "build/**"];

        for file in expected_files.iter() {
            assert!(content.contains(file), "Expected {} in .gitignore", file);
        }
        for dir in expected_dirs.iter() {
            assert!(content.contains(dir), "Expected {} in .gitignore", dir);
        }

        // Verify Gitignore instance
        assert!(
            gitignore
                .matched_path_or_any_parents(temp_dir.path().join("node_modules/test.txt"), false)
                .is_ignore()
        );
        assert!(
            gitignore
                .matched_path_or_any_parents(temp_dir.path().join("target/test.md"), false)
                .is_ignore()
        );
        assert!(
            gitignore
                .matched_path_or_any_parents(temp_dir.path().join("bun.lock"), false)
                .is_ignore()
        );
        assert!(
            gitignore
                .matched_path_or_any_parents(temp_dir.path().join("package-lock.json"), false)
                .is_ignore()
        );
        assert!(
            gitignore
                .matched_path_or_any_parents(temp_dir.path().join("yarn.lock"), false)
                .is_ignore()
        );
        assert!(
            gitignore
                .matched_path_or_any_parents(temp_dir.path().join("pnpm-lock.yaml"), false)
                .is_ignore()
        );

        Ok(())
    }
}
