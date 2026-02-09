use std::{fs, path::PathBuf, process::Command};

use tempfile::TempDir;

use crate::{
    repository::{clone_repository, run_on_repository},
    tests::common::create_test_config,
};

fn init_sample_repo() -> TempDir {
    let repo_dir = TempDir::new().expect("create temp repo");

    let status = Command::new("git")
        .args(["init", "."])
        .current_dir(repo_dir.path())
        .status()
        .expect("init git repo");
    assert!(status.success(), "git init failed");

    for (key, value) in [
        ("user.email", "test@example.com"),
        ("user.name", "Test User"),
    ] {
        let status = Command::new("git")
            .args(["config", key, value])
            .current_dir(repo_dir.path())
            .status()
            .expect("configure git");
        assert!(status.success(), "git config {key} failed", key = key);
    }

    fs::write(repo_dir.path().join("README.md"), "hello remote repo").expect("write README.md");

    let status = Command::new("git")
        .args(["add", "."])
        .current_dir(repo_dir.path())
        .status()
        .expect("git add");
    assert!(status.success(), "git add failed");

    let status = Command::new("git")
        .args(["commit", "-m", "init"])
        .current_dir(repo_dir.path())
        .status()
        .expect("git commit");
    assert!(status.success(), "git commit failed");

    repo_dir
}

#[test]
fn run_on_repository_processes_remote_repo() {
    let remote_repo = init_sample_repo();
    let output_dir = TempDir::new().expect("create output temp dir");
    let output_path = output_dir.path().join("combined.txt");

    let config = create_test_config(PathBuf::from("."), output_path.clone(), |_| {});

    run_on_repository(
        remote_repo
            .path()
            .to_str()
            .expect("repo path should be valid UTF-8"),
        None,
        config,
    )
    .expect("run_on_repository should succeed");

    let contents = fs::read_to_string(&output_path).expect("read output file");
    assert!(
        contents.contains("README.md") && contents.contains("hello remote repo"),
        "output should include cloned file contents"
    );
}

#[test]
fn clone_repository_cleans_up_temp_dir() {
    let remote_repo = init_sample_repo();
    let clone_path = {
        let (_temp_dir, path) = clone_repository(
            remote_repo
                .path()
                .to_str()
                .expect("repo path should be valid UTF-8"),
            None,
        )
        .expect("clone_repository should succeed");
        assert!(path.join(".git").exists(), "expected cloned repo");
        path
    };

    assert!(
        !clone_path.exists(),
        "temp clone directory should be cleaned up after drop"
    );
}
