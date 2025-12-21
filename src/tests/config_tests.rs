use std::fs;
use std::io;
use std::path::PathBuf;
use std::sync::{Mutex, OnceLock};

use clap::{Arg, ArgAction, Command};

use crate::config::{Config, FileConfig, config_from_matches, discover_config_file, merge_config};

static TEST_LOCK: OnceLock<Mutex<()>> = OnceLock::new();

fn test_lock() -> std::sync::MutexGuard<'static, ()> {
    TEST_LOCK.get_or_init(|| Mutex::new(())).lock().unwrap()
}

#[test]
fn test_fileconfig_from_path_valid() {
    // create a temporary yaml file
    let yaml = r#"
directory: "mydir"
output: "outdir"
include_dirs:
  - a
  - b
min_size: 10
respect_gitignore: false
"#;

    let path = "./test_fyai_config.yaml";
    fs::write(path, yaml).expect("write yaml");

    let cfg = FileConfig::from_path(path).expect("load config");

    // cleanup
    let _ = fs::remove_file(path);

    assert_eq!(cfg.directory.unwrap(), "mydir");
    assert_eq!(cfg.output.unwrap(), "outdir");
    assert_eq!(
        cfg.include_dirs.unwrap(),
        vec!["a".to_string(), "b".to_string()]
    );
    assert_eq!(cfg.min_size.unwrap(), 10);
    assert!(!cfg.respect_gitignore.unwrap());
}

#[test]
fn test_fileconfig_from_path_invalid_yaml() {
    // invalid YAML should produce an io::Error with InvalidData
    let path = "./bad_fyai_config.yaml";
    fs::write(path, "not: [valid").expect("write bad yaml");

    let res = FileConfig::from_path(path);

    // cleanup
    let _ = fs::remove_file(path);

    assert!(res.is_err());
    let err = res.err().unwrap();
    assert_eq!(err.kind(), io::ErrorKind::InvalidData);
}

#[test]
fn test_discover_config_file_local() {
    let _lock = test_lock();
    // ensure local ./fyai.yaml presence is detected
    let path = "./fyai.yaml";
    fs::write(path, "directory: test").expect("write fyai");

    let found = discover_config_file();

    // cleanup
    let _ = fs::remove_file(path);

    assert!(found.is_some());
    assert_eq!(found.unwrap(), PathBuf::from("./fyai.yaml"));
}

#[test]
fn test_discover_config_file_global() {
    let _lock = test_lock();
    // Use the system config dir returned by `dirs::config_dir()` instead of modifying env vars.
    // If the system doesn't provide one, skip the test.
    if let Some(config_dir) = dirs::config_dir() {
        let cfg_path = config_dir.join("fyai.yaml");

        // Ensure parent exists
        if let Some(parent) = cfg_path.parent() {
            fs::create_dir_all(parent).expect("create config dir");
        }

        // If a file already exists at that location, back it up so we can restore it.
        let backup = if cfg_path.exists() {
            let bak = cfg_path.with_extension("fyai.bak");
            fs::rename(&cfg_path, &bak).expect("backup existing global config");
            Some(bak)
        } else {
            None
        };

        // Write our test config
        fs::write(&cfg_path, "directory: global").expect("write global fyai");

        // Ensure local file does not exist so discover_config_file prefers the global location
        let _ = fs::remove_file("./fyai.yaml");

        let found = discover_config_file();

        // cleanup: remove our test file
        let _ = fs::remove_file(&cfg_path);

        // restore backup if present
        if let Some(bak) = backup {
            let _ = fs::rename(&bak, &cfg_path);
        }

        assert!(found.is_some());
        assert_eq!(found.unwrap(), cfg_path);
    } else {
        // Cannot run this test on platforms without a config dir; skip gracefully.
        eprintln!("dirs::config_dir() returned None; skipping global discover test");
    }
}

#[test]
fn test_merge_config_precedence() {
    let file = FileConfig {
        include_dirs: Some(vec!["from_file".to_string()]),
        min_size: Some(1),
        max_size: Some(100),
        ..Default::default()
    };

    let cli = Config {
        directory: PathBuf::from("d"),
        output: PathBuf::from("o"),
        include_dirs: Some(vec!["from_cli".to_string()]),
        exclude_dirs: None,
        include_ext: None,
        exclude_ext: None,
        include_files: None,
        exclude_files: None,
        min_size: None,
        max_size: None,
        respect_gitignore: true,
        tree_only: false,
    };

    let explicit = crate::config::ExplicitFlags {
        directory: false,
        output: false,
        respect_gitignore: true,
        tree_only: false,
    };
    let merged = merge_config(file.clone(), cli.clone(), explicit);

    // cli.include_dirs should take precedence
    assert_eq!(merged.include_dirs.unwrap(), vec!["from_cli".to_string()]);
    // cli.min_size None so file's min_size should be used
    assert_eq!(merged.min_size.unwrap(), 1);
    // file.max_size should be used as cli had None
    assert_eq!(merged.max_size.unwrap(), 100);

    // now test when cli doesn't provide include_dirs
    let cli2 = Config {
        include_dirs: None,
        ..cli
    };
    let merged2 = merge_config(file, cli2, explicit);
    assert_eq!(merged2.include_dirs.unwrap(), vec!["from_file".to_string()]);
}

#[test]
fn test_config_from_matches_parsing() {
    let app = Command::new("test")
        .arg(Arg::new("directory").long("directory").num_args(1))
        .arg(Arg::new("output").long("output").num_args(1))
        .arg(Arg::new("include_dirs").long("include_dirs").num_args(1))
        .arg(Arg::new("min_size").long("min_size").num_args(1))
        .arg(
            Arg::new("respect_gitignore")
                .long("respect_gitignore")
                .num_args(1),
        )
        .arg(
            Arg::new("tree_only")
                .long("tree_only")
                .action(ArgAction::SetTrue),
        );

    let matches = app.get_matches_from(vec![
        "prog",
        "--directory",
        "dir",
        "--output",
        "out",
        "--include_dirs",
        "A,B",
        "--min_size",
        "42",
        "--respect_gitignore",
        "false",
        "--tree_only",
    ]);

    let (cfg, _explicit) = config_from_matches(matches).expect("create config");

    assert_eq!(cfg.directory, PathBuf::from("dir"));
    assert_eq!(cfg.output, PathBuf::from("out"));
    assert_eq!(
        cfg.include_dirs.unwrap(),
        vec!["a".to_string(), "b".to_string()]
    );
    assert_eq!(cfg.min_size.unwrap(), 42);
    assert!(!cfg.respect_gitignore);
    assert!(cfg.tree_only);
}

#[test]
fn test_config_from_matches_invalid_min_size() {
    let app = Command::new("test")
        .arg(Arg::new("directory").long("directory").num_args(1))
        .arg(Arg::new("output").long("output").num_args(1))
        .arg(Arg::new("min_size").long("min_size").num_args(1));

    let matches = app.get_matches_from(vec![
        "prog",
        "--directory",
        "d",
        "--output",
        "o",
        "--min_size",
        "nope",
    ]);

    let res = config_from_matches(matches);
    assert!(res.is_err());
}

// ---- New tests covering additional branches and error cases ----

#[test]
fn test_respect_gitignore_true_values() {
    let app = Command::new("test")
        .arg(Arg::new("directory").long("directory").num_args(1))
        .arg(Arg::new("output").long("output").num_args(1))
        .arg(
            Arg::new("respect_gitignore")
                .long("respect_gitignore")
                .num_args(1),
        );

    // clone the Command before using it multiple times so we don't move the original
    let matches = app.clone().get_matches_from(vec![
        "prog",
        "--directory",
        "d",
        "--output",
        "o",
        "--respect_gitignore",
        "1",
    ]);

    let (cfg, _explicit) = config_from_matches(matches).expect("create config");
    assert!(cfg.respect_gitignore);

    // also accept "true" - use the cloned original again
    let matches2 = app.get_matches_from(vec![
        "prog",
        "--directory",
        "d",
        "--output",
        "o",
        "--respect_gitignore",
        "true",
    ]);
    let (cfg2, _explicit) = config_from_matches(matches2).expect("create config");
    assert!(cfg2.respect_gitignore);
}

#[test]
fn test_respect_gitignore_default_when_arg_absent() {
    // If the arg is not registered at all, config_from_matches should treat it as Err(_) => true
    let app = Command::new("test")
        .arg(Arg::new("directory").long("directory").num_args(1))
        .arg(Arg::new("output").long("output").num_args(1));
    let matches = app.get_matches_from(vec!["prog", "--directory", "d", "--output", "o"]);
    let (cfg, _explicit) = config_from_matches(matches).expect("create config");
    assert!(cfg.respect_gitignore);
}

#[test]
fn test_tree_only_absent_arg_definition() {
    // If the tree_only arg is not registered, the code should take the else branch and set false
    let app = Command::new("test")
        .arg(Arg::new("directory").long("directory").num_args(1))
        .arg(Arg::new("output").long("output").num_args(1));
    let matches = app.get_matches_from(vec!["prog", "--directory", "d", "--output", "o"]);
    let (cfg, _explicit) = config_from_matches(matches).expect("create config");
    assert!(!cfg.tree_only);
}

#[test]
fn test_include_ext_parsing_trims_and_lowercases_and_filters_empty() {
    let app = Command::new("test")
        .arg(Arg::new("directory").long("directory").num_args(1))
        .arg(Arg::new("output").long("output").num_args(1))
        .arg(Arg::new("include_ext").long("include_ext").num_args(1));

    let matches = app.get_matches_from(vec![
        "prog",
        "--directory",
        "d",
        "--output",
        "o",
        "--include_ext",
        ".RS, .Md, , ",
    ]);

    let (cfg, _explicit) = config_from_matches(matches).expect("create config");
    let exts = cfg.include_ext.unwrap();
    assert_eq!(exts, vec![".rs".to_string(), ".md".to_string()]);
}

#[test]
fn test_exclude_files_parsing_trims_and_lowercases_and_filters_empty() {
    let app = Command::new("test")
        .arg(Arg::new("directory").long("directory").num_args(1))
        .arg(Arg::new("output").long("output").num_args(1))
        .arg(Arg::new("exclude_files").long("exclude_files").num_args(1));

    let matches = app.get_matches_from(vec![
        "prog",
        "--directory",
        "d",
        "--output",
        "o",
        "--exclude_files",
        " README.md , Cargo.TOML, , ",
    ]);

    let (cfg, _explicit) = config_from_matches(matches).expect("create config");
    let files = cfg.exclude_files.unwrap();
    assert_eq!(
        files,
        vec!["readme.md".to_string(), "cargo.toml".to_string()]
    );
}

#[test]
fn test_missing_directory_error_message() {
    // directory arg is registered but not provided => Ok(None) path -> should cause the ok_or_else message
    let app = Command::new("test")
        .arg(Arg::new("directory").long("directory").num_args(1))
        .arg(Arg::new("output").long("output").num_args(1));
    let matches = app.get_matches_from(vec!["prog", "--output", "o"]);
    let res = config_from_matches(matches);
    assert!(res.is_err());
    let err = res.unwrap_err();
    // The error message was constructed with "Missing directory"
    assert!(err.to_string().to_lowercase().contains("missing directory"));
}

#[test]
fn test_missing_output_error_message() {
    // output arg is registered but not provided => Ok(None) path -> should cause the ok_or_else message
    let app = Command::new("test")
        .arg(Arg::new("directory").long("directory").num_args(1))
        .arg(Arg::new("output").long("output").num_args(1));
    let matches = app.get_matches_from(vec!["prog", "--directory", "d"]);
    let res = config_from_matches(matches);
    assert!(res.is_err());
    let err = res.unwrap_err();
    assert!(err.to_string().to_lowercase().contains("missing output"));
}

/// Additional tests to cover branches where args are registered-but-not-provided
/// and where string-based args are not registered at all.

#[test]
fn test_respect_gitignore_registered_but_not_provided() {
    // respect_gitignore is registered but not supplied -> should take Ok(None) => true
    let app = Command::new("test")
        .arg(Arg::new("directory").long("directory").num_args(1))
        .arg(Arg::new("output").long("output").num_args(1))
        .arg(
            Arg::new("respect_gitignore")
                .long("respect_gitignore")
                .num_args(1),
        );

    let matches = app.get_matches_from(vec!["prog", "--directory", "d", "--output", "o"]);
    let (cfg, _explicit) = config_from_matches(matches).expect("create config");
    assert!(cfg.respect_gitignore);
}

#[test]
fn test_tree_only_registered_but_not_provided() {
    // tree_only is registered as a flag but not present in args -> Ok(None) => false
    let app = Command::new("test")
        .arg(Arg::new("directory").long("directory").num_args(1))
        .arg(Arg::new("output").long("output").num_args(1))
        .arg(
            Arg::new("tree_only")
                .long("tree_only")
                .action(ArgAction::SetTrue),
        );

    let matches = app.get_matches_from(vec!["prog", "--directory", "d", "--output", "o"]);
    let (cfg, _explicit) = config_from_matches(matches).expect("create config");
    assert!(!cfg.tree_only);
}

#[test]
fn test_unregistered_string_args_return_none() {
    // Several string args are not registered at all; try_get_one should return Err(_) and code maps those to None
    let app = Command::new("test")
        .arg(Arg::new("directory").long("directory").num_args(1))
        .arg(Arg::new("output").long("output").num_args(1));

    let matches = app.get_matches_from(vec!["prog", "--directory", "d", "--output", "o"]);
    let (cfg, _explicit) = config_from_matches(matches).expect("create config");

    assert!(cfg.include_dirs.is_none());
    assert!(cfg.exclude_dirs.is_none());
    assert!(cfg.include_ext.is_none());
    assert!(cfg.exclude_ext.is_none());
    assert!(cfg.include_files.is_none());
    assert!(cfg.exclude_files.is_none());
    assert!(cfg.min_size.is_none());
    assert!(cfg.max_size.is_none());
}
