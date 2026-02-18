use std::{fs, path::PathBuf, process::Command};

use tempfile::TempDir;

use crate::{
    git::{clone_repository, run_on_repository},
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

fn get_head_commit(repo_dir: &TempDir) -> String {
    let output = Command::new("git")
        .args(["rev-parse", "HEAD"])
        .current_dir(repo_dir.path())
        .output()
        .expect("git rev-parse");
    assert!(output.status.success(), "git rev-parse failed");
    String::from_utf8_lossy(&output.stdout).trim().to_string()
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
fn run_on_repository_supports_commit() {
    let remote_repo = init_sample_repo();
    let commit = get_head_commit(&remote_repo);
    let output_dir = TempDir::new().expect("create output temp dir");
    let output_path = output_dir.path().join("combined.txt");

    let config = create_test_config(PathBuf::from("."), output_path.clone(), |_| {});

    run_on_repository(
        remote_repo
            .path()
            .to_str()
            .expect("repo path should be valid UTF-8"),
        None,
        Some(&commit),
        config,
    )
    .expect("run_on_repository should succeed with commit");

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
