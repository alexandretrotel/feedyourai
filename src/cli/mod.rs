use clap::{ArgAction, Parser, Subcommand, parser::ValueSource};

use eyre::{OptionExt, Result, eyre};
use feedyourai::config::{Config, ExplicitFlags};
pub mod init;

#[derive(Parser, Debug)]
#[command(
    name = "fyai",
    version = env!("CARGO_PKG_VERSION"),
    about = "A tool to combine text files for AI processing with flexible filtering options.\n\nCONFIG FILE SUPPORT:\n  - You can specify options in a config file (YAML format).\n  - Local config: ./fyai.yaml (used if present in current directory)\n  - Global config: ~/.config/fyai.yaml (used if no local config found)\n  - CLI options override config file values.\n  - See README for details and examples."
)]
pub struct Cli {
    #[arg(
        short = 'i',
        long = "input",
        value_name = "DIR",
        default_value = ".",
        help = "Sets the input directory"
    )]
    pub input: String,

    #[arg(
        short = 'o',
        long = "output",
        value_name = "FILE",
        default_value = "fyai.txt",
        help = "Sets the output file"
    )]
    pub output: String,

    #[arg(
        long = "repo",
        value_name = "URL",
        conflicts_with = "directory",
        help = "Sets the git repository URL"
    )]
    pub repo: Option<String>,

    #[arg(
        long = "repo-branch",
        value_name = "BRANCH",
        requires = "repo",
        help = "Sets the git repository branch or tag"
    )]
    pub repo_branch: Option<String>,

    #[arg(
        long = "repo-commit",
        value_name = "COMMIT",
        requires = "repo",
        help = "Sets the git repository commit SHA"
    )]
    pub repo_commit: Option<String>,

    #[arg(
        long = "include-dirs",
        value_name = "DIRS",
        help = "Sets the directories to include (e.g., src,tests)"
    )]
    pub include_dirs: Option<String>,

    #[arg(
        long = "exclude-dirs",
        value_name = "DIRS",
        help = "Sets the directories to exclude (e.g., node_modules,target)"
    )]
    pub exclude_dirs: Option<String>,

    #[arg(
        long = "include-ext",
        value_name = "EXT",
        help = "Sets the file extensions to include (e.g., .json,.toml)"
    )]
    pub include_ext: Option<String>,

    #[arg(
        long = "exclude-ext",
        value_name = "EXT",
        help = "Sets the file extensions to exclude (e.g., .json,.toml)"
    )]
    pub exclude_ext: Option<String>,

    #[arg(
        long = "include-files",
        value_name = "FILES",
        help = "Sets the file names to include (e.g., README.md,main.rs)"
    )]
    pub include_files: Option<String>,

    #[arg(
        long = "exclude-files",
        value_name = "FILES",
        help = "Sets the file names to exclude (e.g., LICENSE,config.json)"
    )]
    pub exclude_files: Option<String>,

    #[arg(
        short = 'n',
        long = "min-size",
        value_name = "BYTES",
        help = "Exclude files smaller than this size in bytes"
    )]
    pub min_size: Option<u64>,

    #[arg(
        short = 'm',
        long = "max-size",
        value_name = "BYTES",
        help = "Exclude files larger than this size in bytes"
    )]
    pub max_size: Option<u64>,

    #[arg(
        long = "no-gitignore",
        action = ArgAction::SetTrue,
        help = "Sets whether to respect ignore files (gitignore, .ignore, etc.) [default: true]"
    )]
    pub no_gitignore: bool,

    #[arg(long = "tree-only", action = ArgAction::SetTrue, help = "Only output the project directory tree, no file contents")]
    pub tree_only: bool,

    #[arg(short = 't', long = "test", action = ArgAction::SetTrue, help = "Run in test mode")]
    pub test: bool,

    #[command(subcommand)]
    pub command: Option<Commands>,
}

#[derive(Subcommand, Debug)]
pub enum Commands {
    Init {
        #[arg(long = "global", action = ArgAction::SetTrue, help = "Generate config in ~/.config/fyai.yaml")]
        global: bool,

        #[arg(long = "force", action = ArgAction::SetTrue, help = "Overwrite existing config file if present")]
        force: bool,
    },
}

pub fn config_from_matches(matches: clap::ArgMatches) -> Result<(Config, ExplicitFlags)> {
    let directory_set = matches.value_source("input") == Some(ValueSource::CommandLine);
    let output_set = matches.value_source("output") == Some(ValueSource::CommandLine);
    let respect_gitignore_set =
        matches.value_source("respect_gitignore") == Some(ValueSource::CommandLine);
    let tree_only_set = matches.value_source("tree_only") == Some(ValueSource::CommandLine);

    let directory = matches
        .try_get_one::<String>("input")
        .map_err(|_| eyre!("missing directory"))?
        .ok_or_eyre("missing directory")?
        .into();

    let output = matches
        .try_get_one::<String>("output")
        .map_err(|_| eyre!("missing output"))?
        .ok_or_eyre("missing output")?
        .into();

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

    let respect_gitignore = match matches.try_get_one::<bool>("respect_gitignore") {
        Ok(Some(flag)) => *flag,
        Ok(None) | Err(_) => match matches.try_get_one::<String>("respect_gitignore") {
            Ok(Some(s)) => s == "true" || s == "1",
            Ok(None) | Err(_) => true,
        },
    };

    let tree_only = match matches.try_get_one::<bool>("tree_only") {
        Ok(Some(b)) => *b,
        Ok(None) => false,
        Err(_) => false,
    };

    Ok((
        Config {
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
        },
        ExplicitFlags {
            directory: directory_set,
            output: output_set,
            respect_gitignore: respect_gitignore_set,
            tree_only: tree_only_set,
        },
    ))
}
