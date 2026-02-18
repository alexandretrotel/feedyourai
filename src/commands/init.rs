use std::path::PathBuf;

use crate::cli::{Cli, Commands};
use crate::errors::{AppError, AppResult};

pub fn handle_init_subcommand(cli: &Cli) -> AppResult<bool> {
    if let Some(Commands::Init { global, force }) = &cli.command {
        let global = *global;
        let force = *force;

        let (path, display_path) = if global {
            let cfg_dir =
                crate::config::system_config_dir().expect("Could not determine config directory");
            std::fs::create_dir_all(&cfg_dir)?;
            let cfg_path = cfg_dir.join("fyai.yaml");
            (cfg_path.clone(), cfg_path.display().to_string())
        } else {
            let local = PathBuf::from("./fyai.yaml");
            (local.clone(), local.display().to_string())
        };

        if path.exists() && !force {
            return Err(AppError::ConfigAlreadyExists { path: display_path });
        }

        let template = r#"# fyai.yaml - Configuration file for fyai
# All options are optional. CLI flags override config values.
# See README.md for details.

directory: .
output: fyai.txt
include_dirs:
  - src
  - docs
exclude_dirs:
  - node_modules
  - dist
include_ext:
  - md
  - txt
exclude_ext:
  - log
  - tmp
include_files:
  - README.md
  - main.rs
exclude_files:
  - LICENSE
  - config.json
min_size: 10240
max_size: 512000
respect_gitignore: true
tree_only: false
"#;

        std::fs::write(&path, template)?;
        println!("Template config file written to {}", display_path);
        return Ok(true);
    }
    Ok(false)
}
