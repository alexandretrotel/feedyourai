//! Implementation of the `init` subcommand.

use std::path::PathBuf;

use super::{Cli, Command};
use color_eyre::eyre::{OptionExt, Result, bail};

/// If `cli` carries an `init` subcommand, writes a starter `fyai.yaml` and
/// returns `Ok(true)`; otherwise returns `Ok(false)` so the caller proceeds
/// with a normal combine run.
///
/// Fails if the target config file already exists and `--force` wasn't
/// passed.
pub fn handle_init_subcommand(cli: &Cli) -> Result<bool> {
    if let Some(Command::Init { global, force }) = &cli.command {
        let global = *global;
        let force = *force;

        let (path, display_path) = if global {
            let cfg_dir = feedyourai::config::system_config_dir()
                .ok_or_eyre("could not determine config directory")?;
            std::fs::create_dir_all(&cfg_dir)?;
            let cfg_path = cfg_dir.join("fyai.yaml");
            (cfg_path.clone(), cfg_path.display().to_string())
        } else {
            let local = PathBuf::from("./fyai.yaml");
            (local.clone(), local.display().to_string())
        };

        if path.exists() && !force {
            bail!("config file already exists at {display_path}. Use --force to overwrite.");
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
human: false
"#;

        std::fs::write(&path, template)?;
        println!("Template config file written to {}", display_path);
        return Ok(true);
    }
    Ok(false)
}
