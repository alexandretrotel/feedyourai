use std::fs::File;
use std::io::Write;
use std::path::Path;
use tempfile::TempDir;

pub fn setup_temp_dir() -> TempDir {
    TempDir::new().expect("Failed to create temporary directory")
}

pub fn create_file<P: AsRef<Path>>(path: P, contents: &str) -> std::io::Result<()> {
    let mut file = File::create(path)?;
    file.write_all(contents.as_bytes())?;
    Ok(())
}

pub fn create_gitignore<P: AsRef<Path>>(dir: P, contents: &str) -> std::io::Result<()> {
    create_file(dir.as_ref().join(".gitignore"), contents)
}
