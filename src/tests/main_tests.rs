use std::{
    env, fs, io,
    path::PathBuf,
    sync::{Mutex, OnceLock},
};
use tempfile::TempDir;

use crate::cli::create_commands;

static SERIALIZE_TESTS: OnceLock<Mutex<()>> = OnceLock::new();

fn acquire_lock() -> &'static Mutex<()> {
    SERIALIZE_TESTS.get_or_init(|| Mutex::new(()))
}

fn lock_tests() -> std::sync::MutexGuard<'static, ()> {
    acquire_lock()
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner())
}

/// Helper to restore the current working directory when dropped.
struct CwdGuard {
    orig: PathBuf,
}

impl CwdGuard {
    fn new() -> Self {
        let orig = env::current_dir().expect("failed to get current dir");
        Self { orig }
    }
}

impl Drop for CwdGuard {
    fn drop(&mut self) {
        // Best-effort restore; tests will fail earlier if this can't be done.
        let _ = env::set_current_dir(&self.orig);
    }
}

/// Helper to restore an environment variable when dropped.
struct EnvVarGuard {
    key: String,
    prev: Option<String>,
}

impl EnvVarGuard {
    fn set(key: &str, val: &str) -> Self {
        let prev = env::var(key).ok();
        unsafe { std::env::set_var(key, val) };
        Self {
            key: key.to_string(),
            prev,
        }
    }
}

impl Drop for EnvVarGuard {
    fn drop(&mut self) {
        match &self.prev {
            Some(v) => unsafe { std::env::set_var(&self.key, v) },
            None => unsafe { std::env::remove_var(&self.key) },
        }
    }
}

#[test]
fn test_init_local_creates_file() {
    // Create a temporary directory and switch to it so init writes ./fyai.yaml there.
    let _serial = lock_tests();
    let temp = TempDir::new().expect("create tempdir");
    let _cwd_guard = CwdGuard::new();
    env::set_current_dir(temp.path()).expect("set cwd to temp");

    let matches = create_commands().get_matches_from(vec!["fyai", "init"]);
    let handled = crate::handle_init_subcommand(&matches)
        .expect("handle_init_subcommand should succeed for local init");
    assert!(handled, "Expected init subcommand to be handled");

    let file_path = temp.path().join("fyai.yaml");
    assert!(file_path.exists(), "Expected local fyai.yaml to be created");

    // Check the file contains the expected header to ensure it's the template.
    let content = fs::read_to_string(&file_path).expect("read created fyai.yaml");
    assert!(
        content.contains("# fyai.yaml - Configuration file for fyai"),
        "Template content not found"
    );
}

#[test]
fn test_init_global_uses_home_dir() {
    // Create a temporary directory to act as HOME and set HOME env var.
    let _serial = lock_tests();
    let temp_home = TempDir::new().expect("create tempdir for HOME");
    let _env_guard = EnvVarGuard::set("HOME", temp_home.path().to_str().unwrap());

    let matches = create_commands().get_matches_from(vec!["fyai", "init", "--global"]);
    let handled = crate::handle_init_subcommand(&matches)
        .expect("handle_init_subcommand should succeed for global init");
    assert!(handled, "Expected init subcommand to be handled");

    // Determine expected config path using XDG config_dir (fallback to $HOME/.config).
    let cfg_dir =
        dirs::config_dir().unwrap_or_else(|| temp_home.path().to_path_buf().join(".config"));
    let cfg_path = cfg_dir.join("fyai.yaml");
    assert!(
        cfg_path.exists(),
        "Expected global fyai.yaml to be created at {}",
        cfg_path.display()
    );

    let content = fs::read_to_string(&cfg_path).expect("read created fyai.yaml");
    assert!(
        content.contains("# fyai.yaml - Configuration file for fyai"),
        "Global template content not found"
    );
}

#[test]
fn test_init_already_exists_without_force_errors() {
    // Ensure local file exists and that calling init without --force returns AlreadyExists
    let _serial = lock_tests();
    let temp = TempDir::new().expect("create tempdir");
    let _cwd_guard = CwdGuard::new();
    env::set_current_dir(temp.path()).expect("set cwd to temp");

    let file_path = temp.path().join("fyai.yaml");
    fs::write(&file_path, "existing").expect("create existing file");

    let matches = create_commands().get_matches_from(vec!["fyai", "init"]);
    let res = crate::handle_init_subcommand(&matches);
    assert!(
        res.is_err(),
        "Expected error when config exists and --force not provided"
    );
    let err = res.unwrap_err();
    assert_eq!(err.kind(), io::ErrorKind::AlreadyExists);
}

#[test]
fn test_init_force_overwrites_existing() {
    // Create an existing fyai.yaml and call init --force to overwrite it.
    let _serial = lock_tests();
    let temp = TempDir::new().expect("create tempdir");
    let _cwd_guard = CwdGuard::new();
    env::set_current_dir(temp.path()).expect("set cwd to temp");

    let file_path = temp.path().join("fyai.yaml");
    fs::write(&file_path, "old content").expect("create existing file");

    let matches = create_commands().get_matches_from(vec!["fyai", "init", "--force"]);
    let handled = crate::handle_init_subcommand(&matches)
        .expect("handle_init_subcommand should succeed with --force");
    assert!(handled, "Expected init to be handled even with --force");

    let content = fs::read_to_string(&file_path).expect("read overwritten fyai.yaml");
    assert!(
        content.contains("# fyai.yaml - Configuration file for fyai"),
        "Expected template content after force overwrite"
    );
}
