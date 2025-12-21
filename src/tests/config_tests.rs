use std::fs;
use std::path::PathBuf;

use clap::{Arg, ArgAction, Command};

use crate::config::{Config, FileConfig, config_from_matches, discover_config_file, merge_config};

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
    assert_eq!(cfg.respect_gitignore.unwrap(), false);
}

#[test]
fn test_discover_config_file_local() {
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

    let merged = merge_config(file.clone(), cli.clone());

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
    let merged2 = merge_config(file, cli2);
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

    let cfg = config_from_matches(matches).expect("create config");

    assert_eq!(cfg.directory, PathBuf::from("dir"));
    assert_eq!(cfg.output, PathBuf::from("out"));
    assert_eq!(
        cfg.include_dirs.unwrap(),
        vec!["a".to_string(), "b".to_string()]
    );
    assert_eq!(cfg.min_size.unwrap(), 42);
    assert_eq!(cfg.respect_gitignore, false);
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
