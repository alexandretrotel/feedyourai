//! Orchestrates a single combine run: one parallel walk over
//! `config.directory` builds the directory tree and, unless
//! `config.tree_only`, every matching file's contents, written to
//! `config.output` through a single buffered writer.

use std::fs::File;
use std::io::{self, BufWriter, Write};

use crate::config::Config;

use super::collect::collect_entries;
use super::process::write_file_contents;
use super::tree::render_tree;

/// Writes `config.directory`'s (filtered) tree, and optionally its files'
/// contents, to `config.output`.
pub fn scan(config: &Config) -> io::Result<()> {
    let mut output = BufWriter::new(File::create(&config.output)?);

    if config.directory.read_dir()?.count() == 0 {
        write!(output, "- Tree Structure\n\nThe directory is empty.\n\n")?;
        return output.flush();
    }

    let entries = collect_entries(config)?;
    write!(
        output,
        "{}",
        render_tree(&entries, &config.directory, config.human)
    )?;

    if !config.tree_only {
        write_file_contents(&entries, config, &mut output)?;
    }

    output.flush()
}
