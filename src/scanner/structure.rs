use std::fmt::Write;
use std::io;
use std::path::Path;

use crate::config::Config;

use super::filter::PathFilter;
use super::walker::build_walker;

pub fn get_directory_structure(
    root: &Path,
    ignored_files: &[&str],
    ignored_dirs: &[&str],
    config: &Config,
) -> io::Result<String> {
    let mut structure = String::new();
    structure.push_str("- Tree Structure\n\n");

    if root.read_dir()?.count() == 0 {
        structure.push_str("The directory is empty.\n\n");
        return Ok(structure);
    }

    let filter = PathFilter::new(config, ignored_dirs);
    let walker = build_walker(root, ignored_files, ignored_dirs, config)?;
    write_directory_structure(&mut structure, &filter, walker)?;

    structure.push('\n');
    Ok(structure)
}

fn write_directory_structure(
    output: &mut String,
    filter: &PathFilter<'_>,
    walker: ignore::Walk,
) -> io::Result<()> {
    for entry in walker {
        let entry = entry.map_err(io::Error::other)?;
        let path = entry.path();
        let is_dir = entry
            .file_type()
            .map(|file_type| file_type.is_dir())
            .unwrap_or_else(|| path.is_dir());

        if !filter.allows_entry(path, is_dir) {
            continue;
        }

        let depth = entry.depth();
        let indent = "  ".repeat(depth);
        if let Some(name) = path.file_name() {
            let marker = if is_dir { "/" } else { "" };
            writeln!(output, "{}{}{}", indent, name.to_string_lossy(), marker)
                .map_err(io::Error::other)?;
        }
    }

    Ok(())
}
