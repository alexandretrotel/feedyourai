use clap::{ArgAction, Parser, Subcommand};

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
