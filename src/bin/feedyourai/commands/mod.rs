//! Argument parsing: the [`Cli`] struct, its `init` subcommand, and
//! conversion of parsed `clap` matches into a library
//! [`PartialConfig`](feedyourai::config::PartialConfig).

use clap::{ArgAction, Parser, Subcommand, parser::ValueSource};

use color_eyre::eyre::{Result, eyre};
use feedyourai::config::PartialConfig;

/// The `init` subcommand: writes a starter `fyai.toml`.
pub mod init;

/// Top-level command-line arguments.
#[derive(Parser, Debug)]
#[command(
    name = "fyai",
    version = env!("CARGO_PKG_VERSION"),
    about = "A tool to combine text files for LLM processing with flexible filtering options.\n\nCONFIG FILE SUPPORT:\n  - You can specify options in a config file (TOML format).\n  - Local config: ./fyai.toml (used if present in current directory)\n  - Global config: system config directory (used if no local config found).\n    Honors $XDG_CONFIG_HOME (any platform, if set to an absolute path),\n    else the platform default. Run `fyai init --global` to see the exact path.\n  - CLI options override config file values.\n  - You can also drop a .fyaiignore file (gitignore syntax) to exclude paths.\n  - See README for details and examples."
)]
pub struct Cli {
    /// Sets the input directory.
    #[arg(
        short = 'i',
        long = "input",
        value_name = "DIR",
        default_value = ".",
        help = "Sets the input directory"
    )]
    pub input: String,

    /// Sets the output file.
    #[arg(
        short = 'o',
        long = "output",
        value_name = "FILE",
        default_value = "fyai.txt",
        help = "Sets the output file"
    )]
    pub output: String,

    /// Sets the git repository URL to clone and scan instead of a local
    /// directory.
    #[arg(
        long = "repo",
        value_name = "URL",
        conflicts_with = "input",
        help = "Sets the git repository URL"
    )]
    pub repo: Option<String>,

    /// Sets the git repository branch or tag to check out. Requires `--repo`.
    #[arg(
        long = "repo-branch",
        value_name = "BRANCH",
        requires = "repo",
        help = "Sets the git repository branch or tag"
    )]
    pub repo_branch: Option<String>,

    /// Sets the git repository commit SHA to check out. Requires `--repo`.
    #[arg(
        long = "repo-commit",
        value_name = "COMMIT",
        requires = "repo",
        help = "Sets the git repository commit SHA"
    )]
    pub repo_commit: Option<String>,

    /// Sets the directories to include (e.g., `src,tests`).
    #[arg(
        long = "include-dirs",
        value_name = "DIRS",
        help = "Sets the directories to include (e.g., src,tests)"
    )]
    pub include_dirs: Option<String>,

    /// Sets the directories to exclude (e.g., `node_modules,target`).
    #[arg(
        long = "exclude-dirs",
        value_name = "DIRS",
        help = "Sets the directories to exclude (e.g., node_modules,target)"
    )]
    pub exclude_dirs: Option<String>,

    /// Sets the file extensions to include (e.g., `.json,.toml`).
    #[arg(
        long = "include-ext",
        value_name = "EXT",
        help = "Sets the file extensions to include (e.g., .json,.toml)"
    )]
    pub include_ext: Option<String>,

    /// Sets the file extensions to exclude (e.g., `.json,.toml`).
    #[arg(
        long = "exclude-ext",
        value_name = "EXT",
        help = "Sets the file extensions to exclude (e.g., .json,.toml)"
    )]
    pub exclude_ext: Option<String>,

    /// Sets the file names to include (e.g., `README.md,main.rs`).
    #[arg(
        long = "include-files",
        value_name = "FILES",
        help = "Sets the file names to include (e.g., README.md,main.rs)"
    )]
    pub include_files: Option<String>,

    /// Sets the file names to exclude (e.g., `LICENSE,config.json`).
    #[arg(
        long = "exclude-files",
        value_name = "FILES",
        help = "Sets the file names to exclude (e.g., LICENSE,config.json)"
    )]
    pub exclude_files: Option<String>,

    /// Excludes files smaller than this size in bytes.
    #[arg(
        short = 'n',
        long = "min-size",
        value_name = "BYTES",
        help = "Exclude files smaller than this size in bytes"
    )]
    pub min_size: Option<u64>,

    /// Excludes files larger than this size in bytes.
    #[arg(
        short = 'm',
        long = "max-size",
        value_name = "BYTES",
        help = "Exclude files larger than this size in bytes"
    )]
    pub max_size: Option<u64>,

    /// Sets whether to respect .gitignore/.ignore and friends \[default:
    /// true\]. `.fyaiignore` is always respected regardless of this flag.
    #[arg(
        long = "no-gitignore",
        action = ArgAction::SetTrue,
        help = "Sets whether to respect .gitignore/.ignore and friends [default: true] (.fyaiignore is always respected)"
    )]
    pub no_gitignore: bool,

    /// Only outputs the project directory tree, no file contents.
    #[arg(long = "tree-only", action = ArgAction::SetTrue, help = "Only output the project directory tree, no file contents")]
    pub tree_only: bool,

    /// Renders the directory tree with `tree`-style connector glyphs instead
    /// of the minimal two-space indent.
    #[arg(long = "human", action = ArgAction::SetTrue, help = "Render the directory tree with tree-style connector glyphs")]
    pub human: bool,

    /// Runs in test mode.
    #[arg(short = 't', long = "test", action = ArgAction::SetTrue, help = "Run in test mode")]
    pub test: bool,

    /// Optional subcommand (currently only `init`).
    #[command(subcommand)]
    pub command: Option<Command>,
}

/// Subcommands available alongside the default combine behavior.
#[derive(Subcommand, Debug)]
pub enum Command {
    /// Writes a starter `fyai.toml` config file.
    Init {
        /// Generates the config in the system config directory instead of
        /// the current directory.
        #[arg(
            long = "global",
            action = ArgAction::SetTrue,
            help = "Generate config in the system config directory (see `fyai --help`)"
        )]
        global: bool,

        /// Overwrites an existing config file if present.
        #[arg(long = "force", action = ArgAction::SetTrue, help = "Overwrite existing config file if present")]
        force: bool,
    },
}

/// Converts parsed `clap` matches into a [`PartialConfig`], leaving a field
/// `None` unless it was explicitly set on the command line — so an unset
/// flag can't shadow a `fyai.toml` value when [`merge_config`] reconciles
/// the two.
///
/// [`merge_config`]: feedyourai::config::merge_config
///
/// Comma-separated list options (`include_dirs`, `exclude_ext`, ...) are
/// split, trimmed, lower-cased, and emptied entries dropped.
pub fn config_from_matches(matches: clap::ArgMatches) -> Result<PartialConfig> {
    let directory = explicit_string(&matches, "input");
    let output = explicit_string(&matches, "output");

    let include_dirs = match matches.try_get_one::<String>("include_dirs") {
        Ok(opt) => opt.map(|dirs| {
            dirs.split(',')
                .map(|s| s.trim().to_lowercase())
                .filter(|s| !s.is_empty())
                .collect::<Vec<_>>()
        }),
        Err(_) => None,
    };

    let exclude_dirs = match matches.try_get_one::<String>("exclude_dirs") {
        Ok(opt) => opt.map(|dirs| {
            dirs.split(',')
                .map(|s| s.trim().to_lowercase())
                .filter(|s| !s.is_empty())
                .collect::<Vec<_>>()
        }),
        Err(_) => None,
    };

    let include_ext = match matches.try_get_one::<String>("include_ext") {
        Ok(opt) => opt.map(|ext| {
            ext.split(',')
                .map(|s| s.trim().to_lowercase())
                .filter(|s| !s.is_empty())
                .collect::<Vec<_>>()
        }),
        Err(_) => None,
    };

    let exclude_ext = match matches.try_get_one::<String>("exclude_ext") {
        Ok(opt) => opt.map(|ext| {
            ext.split(',')
                .map(|s| s.trim().to_lowercase())
                .filter(|s| !s.is_empty())
                .collect::<Vec<_>>()
        }),
        Err(_) => None,
    };

    let include_files = match matches.try_get_one::<String>("include_files") {
        Ok(opt) => opt.map(|files| {
            files
                .split(',')
                .map(|s| s.trim().to_lowercase())
                .filter(|s| !s.is_empty())
                .collect::<Vec<_>>()
        }),
        Err(_) => None,
    };

    let exclude_files = match matches.try_get_one::<String>("exclude_files") {
        Ok(opt) => opt.map(|files| {
            files
                .split(',')
                .map(|s| s.trim().to_lowercase())
                .filter(|s| !s.is_empty())
                .collect::<Vec<_>>()
        }),
        Err(_) => None,
    };

    let min_size = match matches.try_get_one::<u64>("min_size") {
        Ok(Some(value)) => Some(*value),
        Ok(None) | Err(_) => match matches.try_get_one::<String>("min_size") {
            Ok(Some(s)) => Some(s.parse::<u64>().map_err(|_| eyre!("invalid min-size"))?),
            Ok(None) | Err(_) => None,
        },
    };

    let max_size = match matches.try_get_one::<u64>("max_size") {
        Ok(Some(value)) => Some(*value),
        Ok(None) | Err(_) => match matches.try_get_one::<String>("max_size") {
            Ok(Some(s)) => Some(s.parse::<u64>().map_err(|_| eyre!("invalid max-size"))?),
            Ok(None) | Err(_) => None,
        },
    };

    // `--no-gitignore` is a negated flag: respect_gitignore is the opposite
    // of whatever was passed.
    let respect_gitignore =
        explicit_flag(&matches, "no_gitignore").map(|no_gitignore| !no_gitignore);
    let tree_only = explicit_flag(&matches, "tree_only");
    let human = explicit_flag(&matches, "human");

    Ok(PartialConfig {
        directory,
        output,
        include_dirs,
        exclude_dirs,
        include_ext,
        exclude_ext,
        include_files,
        exclude_files,
        min_size,
        max_size,
        respect_gitignore,
        tree_only,
        human,
    })
}

/// Returns `matches`' string value for `id`, but only if it was passed
/// explicitly on the command line — a `clap` `default_value` doesn't count.
fn explicit_string(matches: &clap::ArgMatches, id: &str) -> Option<String> {
    if matches.value_source(id) != Some(ValueSource::CommandLine) {
        return None;
    }
    matches.get_one::<String>(id).cloned()
}

/// Returns `matches`' `SetTrue` flag value for `id`, but only if it was
/// passed explicitly on the command line.
fn explicit_flag(matches: &clap::ArgMatches, id: &str) -> Option<bool> {
    if matches.value_source(id) != Some(ValueSource::CommandLine) {
        return None;
    }
    Some(matches.get_flag(id))
}
