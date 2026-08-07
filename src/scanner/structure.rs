//! Renders the filtered directory tree, either as a minimal indented list
//! (the default) or as a `tree`-style connector diagram when
//! `config.human` is set.

use std::fmt::Write;
use std::io;

use crate::config::Config;

use super::filter::PathFilter;
use super::walker::build_walker;

/// One filtered walk entry, flattened to just what rendering needs.
struct FlatEntry {
    depth: usize,
    name: String,
    is_dir: bool,
}

/// A directory tree node, built from a [`FlatEntry`] list. Only needed for
/// the connector-style ([`render_glyph_tree`]) renderer, which has to know
/// each node's siblings to draw `├──` vs `└──`.
struct Node {
    name: String,
    is_dir: bool,
    children: Vec<Node>,
}

/// Walks `config.directory` and renders every entry that passes the
/// configured filters as a tree, prefixed with a `- Tree Structure` header.
///
/// Uses connector-style glyphs (`├──`, `└──`, `│`) when `config.human` is
/// set, or a minimal two-space indent otherwise (the default: fewer bytes,
/// just as easy for an LLM to parse from depth alone).
///
/// Returns a single-line "The directory is empty." body if the root has no
/// entries at all (before filtering).
pub fn get_directory_tree(config: &Config) -> io::Result<String> {
    let root = &config.directory;
    let mut structure = String::new();
    structure.push_str("- Tree Structure\n\n");

    if root.read_dir()?.count() == 0 {
        structure.push_str("The directory is empty.\n\n");
        return Ok(structure);
    }

    let filter = PathFilter::new(config);
    let walker = build_walker(config)?;
    let entries = collect_entries(&filter, walker)?;

    let root_label = root
        .file_name()
        .and_then(|name| name.to_str())
        .filter(|name| !name.is_empty())
        .unwrap_or(".");

    if config.human {
        writeln!(structure, "{root_label}").map_err(io::Error::other)?;
        render_glyph_tree(&build_tree(entries), "", &mut structure).map_err(io::Error::other)?;
    } else {
        writeln!(structure, "{root_label}/").map_err(io::Error::other)?;
        render_indent_tree(&entries, &mut structure).map_err(io::Error::other)?;
    }

    structure.push('\n');
    Ok(structure)
}

/// Walks `walker`, keeping only entries the filter allows, and flattens each
/// into a [`FlatEntry`] (dropping the root entry itself, at depth 0).
fn collect_entries(filter: &PathFilter<'_>, walker: ignore::Walk) -> io::Result<Vec<FlatEntry>> {
    let mut entries = Vec::new();

    for entry in walker {
        let entry = entry.map_err(io::Error::other)?;
        let path = entry.path();
        let depth = entry.depth();
        if depth == 0 {
            continue;
        }

        let is_dir = entry
            .file_type()
            .map(|file_type| file_type.is_dir())
            .unwrap_or_else(|| path.is_dir());

        if !filter.allows_entry(path, is_dir) {
            continue;
        }

        let Some(name) = path.file_name().and_then(|name| name.to_str()) else {
            continue;
        };
        entries.push(FlatEntry {
            depth,
            name: name.to_string(),
            is_dir,
        });
    }

    Ok(entries)
}

/// Appends `entries` to `output` as a minimal two-space-per-depth indented
/// list, suffixing directories with `/`.
fn render_indent_tree(entries: &[FlatEntry], output: &mut String) -> std::fmt::Result {
    for entry in entries {
        let indent = "  ".repeat(entry.depth);
        let marker = if entry.is_dir { "/" } else { "" };
        writeln!(output, "{indent}{}{marker}", entry.name)?;
    }
    Ok(())
}

/// Rebuilds the tree structure implied by `entries`' depths (a preorder
/// walk, one depth increase per level) into nested [`Node`]s.
fn build_tree(entries: Vec<FlatEntry>) -> Vec<Node> {
    build_children(&mut entries.into_iter().peekable(), 1)
}

fn build_children(
    entries: &mut std::iter::Peekable<std::vec::IntoIter<FlatEntry>>,
    depth: usize,
) -> Vec<Node> {
    let mut children = Vec::new();

    while let Some(next) = entries.peek() {
        if next.depth != depth {
            break;
        }
        let entry = entries.next().expect("just peeked");

        let node_children = if entry.is_dir {
            match entries.peek() {
                Some(next) if next.depth > depth => build_children(entries, depth + 1),
                _ => Vec::new(),
            }
        } else {
            Vec::new()
        };

        children.push(Node {
            name: entry.name,
            is_dir: entry.is_dir,
            children: node_children,
        });
    }

    children
}

/// Appends `nodes` to `output` using `tree`-style connectors (`├── `,
/// `└── `, `│   `), recursing into directories with an extended prefix.
fn render_glyph_tree(nodes: &[Node], prefix: &str, output: &mut String) -> std::fmt::Result {
    for (index, node) in nodes.iter().enumerate() {
        let is_last = index == nodes.len() - 1;
        let connector = if is_last { "└── " } else { "├── " };
        let marker = if node.is_dir { "/" } else { "" };
        writeln!(output, "{prefix}{connector}{}{marker}", node.name)?;

        if !node.children.is_empty() {
            let child_prefix = format!("{prefix}{}", if is_last { "    " } else { "│   " });
            render_glyph_tree(&node.children, &child_prefix, output)?;
        }
    }

    Ok(())
}
