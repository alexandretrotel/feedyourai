use ignore::gitignore::Gitignore;
use std::fs;
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

pub fn setup_test_dir() -> (TempDir, Gitignore) {
    let temp_dir = TempDir::new().unwrap();
    let root = temp_dir.path();

    // Create a sample directory structure
    fs::create_dir_all(root.join("src")).unwrap();
    fs::create_dir_all(root.join("tests")).unwrap();
    fs::create_dir_all(root.join("target")).unwrap();
    File::create(root.join("README.md")).unwrap();
    File::create(root.join("src/main.rs")).unwrap();
    File::create(root.join("tests/test1.rs")).unwrap();

    // Create a .gitignore file
    let mut gitignore_file = File::create(root.join(".gitignore")).unwrap();
    gitignore_file.write_all(b"target/\n").unwrap();
    let gitignore = Gitignore::new(root.join(".gitignore")).0;

    (temp_dir, gitignore)
}
