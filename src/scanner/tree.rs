//! Renders the directory tree collected by [`super::collect::collect_entries`],
//! either as a minimal indented list (the default) or as a `tree`-style
//! connector diagram when `config.human` is set.

use std::fmt::Write;
use std::path::Path;

use super::collect::Entry;

/// A directory tree node, built from a flat [`Entry`] list. Only needed for
/// the connector-style ([`render_glyph_tree`]) renderer, which has to know
/// each node's siblings to draw `├──` vs `└──`.
struct Node<'e> {
    /// The entry's file name.
    name: &'e str,
    /// Whether this node is a directory rather than a file.
    is_dir: bool,
    /// Direct children, in walk order; always empty for files.
    children: Vec<Node<'e>>,
}

/// Renders `entries` (already walked and filtered) as a tree, prefixed with
/// a `- Tree Structure` header.
///
/// Uses connector-style glyphs (`├──`, `└──`, `│`) when `human` is set, or a
/// minimal two-space indent otherwise (the default: fewer bytes, just as
/// easy for an LLM to parse from depth alone).
pub(crate) fn render_tree(entries: &[Entry], root: &Path, human: bool) -> String {
    let mut structure = String::new();
    structure.push_str("- Tree Structure\n\n");

    let root_label = root
        .file_name()
        .and_then(|name| name.to_str())
        .filter(|name| !name.is_empty())
        .unwrap_or(".");

    // `fmt::Write` on `String` never actually fails (no allocation can
    // realistically be exhausted here), so these are infallible in
    // practice.
    if human {
        writeln!(structure, "{root_label}").expect("String write is infallible");
        render_glyph_tree(&build_tree(entries), "", &mut structure);
    } else {
        writeln!(structure, "{root_label}/").expect("String write is infallible");
        render_indent_tree(entries, &mut structure);
    }

    structure.push('\n');
    structure
}

/// Returns `entry`'s file name, or an empty string if it has none.
fn entry_name(entry: &Entry) -> &str {
    entry
        .path
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or_default()
}

/// Appends `entries` to `output` as a minimal two-space-per-depth indented
/// list, suffixing directories with `/`.
fn render_indent_tree(entries: &[Entry], output: &mut String) {
    for entry in entries {
        let indent = "  ".repeat(entry.depth);
        let marker = if entry.is_dir { "/" } else { "" };
        writeln!(output, "{indent}{}{marker}", entry_name(entry))
            .expect("String write is infallible");
    }
}

/// Rebuilds the tree structure implied by `entries`' depths (a preorder
/// sequence, one depth increase per level) into nested [`Node`]s.
fn build_tree(entries: &[Entry]) -> Vec<Node<'_>> {
    build_children(&mut entries.iter().peekable(), 1)
}

fn build_children<'e>(
    entries: &mut std::iter::Peekable<std::slice::Iter<'e, Entry>>,
    depth: usize,
) -> Vec<Node<'e>> {
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
            name: entry_name(entry),
            is_dir: entry.is_dir,
            children: node_children,
        });
    }

    children
}

/// Appends `nodes` to `output` using `tree`-style connectors (`├── `,
/// `└── `, `│   `), recursing into directories with an extended prefix.
fn render_glyph_tree(nodes: &[Node<'_>], prefix: &str, output: &mut String) {
    for (index, node) in nodes.iter().enumerate() {
        let is_last = index == nodes.len() - 1;
        let connector = if is_last { "└── " } else { "├── " };
        let marker = if node.is_dir { "/" } else { "" };
        writeln!(output, "{prefix}{connector}{}{marker}", node.name)
            .expect("String write is infallible");

        if !node.children.is_empty() {
            let child_prefix = format!("{prefix}{}", if is_last { "    " } else { "│   " });
            render_glyph_tree(&node.children, &child_prefix, output);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    /// Builds a small [`Entry`] fixture. `size` is `None` for directories,
    /// `Some(0)` for files (the tree renderer never reads it).
    fn entry(path: &str, depth: usize, is_dir: bool) -> Entry {
        Entry {
            path: PathBuf::from(path),
            depth,
            is_dir,
            size: if is_dir { None } else { Some(0) },
        }
    }

    /// A 3-level fixture with both a non-last and a last node at every
    /// nested level, so both `├──`/`│   ` and `└──`/`    ` connectors get
    /// exercised:
    ///
    /// ```text
    /// a/       (dir, not last)
    ///   a1     (file, not last)
    ///   a2/    (dir, last)
    ///     a2a  (file, only/last child)
    /// b        (file, last)
    /// ```
    fn deep_entries() -> Vec<Entry> {
        vec![
            entry("a", 1, true),
            entry("a/a1", 2, false),
            entry("a/a2", 2, true),
            entry("a/a2/a2a", 3, false),
            entry("b", 1, false),
        ]
    }

    #[test]
    fn renders_indent_tree_for_nested_entries() {
        let out = render_tree(&deep_entries(), Path::new("myproj"), false);
        assert_eq!(
            out,
            "- Tree Structure\n\nmyproj/\n  a/\n    a1\n    a2/\n      a2a\n  b\n\n"
        );
    }

    #[test]
    fn renders_glyph_tree_for_nested_entries() {
        let out = render_tree(&deep_entries(), Path::new("myproj"), true);
        assert_eq!(
            out,
            "- Tree Structure\n\nmyproj\n├── a/\n│   ├── a1\n│   └── a2/\n│       └── a2a\n└── b\n\n"
        );
    }

    #[test]
    fn glyph_tree_handles_directories_with_no_children() {
        // `x` has a sibling at the same depth right after it (no children),
        // and `y` is last with nothing after it at all (peek() is None).
        // Both should hit the `_ => Vec::new()` arm in `build_children`.
        let entries = vec![entry("x", 1, true), entry("y", 1, true)];
        let out = render_tree(&entries, Path::new("myproj"), true);
        assert_eq!(out, "- Tree Structure\n\nmyproj\n├── x/\n└── y/\n\n");
    }

    #[test]
    fn renders_empty_entries_as_header_and_root_only_indent() {
        let out = render_tree(&[], Path::new("myproj"), false);
        assert_eq!(out, "- Tree Structure\n\nmyproj/\n\n");
    }

    #[test]
    fn renders_empty_entries_as_header_and_root_only_glyph() {
        let out = render_tree(&[], Path::new("myproj"), true);
        assert_eq!(out, "- Tree Structure\n\nmyproj\n\n");
    }

    #[test]
    fn root_label_falls_back_to_dot_when_file_name_is_none() {
        // Empirically verified: `Path::new(".")`, `Path::new("/")`, and
        // `Path::new("")` all return `None` from `file_name()`.
        for root in [Path::new("."), Path::new("/"), Path::new("")] {
            let out = render_tree(&[], root, false);
            assert_eq!(out, "- Tree Structure\n\n./\n\n", "root = {root:?}");

            let out_human = render_tree(&[], root, true);
            assert_eq!(out_human, "- Tree Structure\n\n.\n\n", "root = {root:?}");
        }
    }

    #[test]
    fn entry_name_returns_file_name_when_present() {
        let e = entry("some/dir/file.rs", 2, false);
        assert_eq!(entry_name(&e), "file.rs");
    }

    #[test]
    fn entry_name_returns_empty_string_when_no_file_name() {
        let e = Entry {
            path: PathBuf::from("/"),
            depth: 1,
            is_dir: false,
            size: Some(0),
        };
        assert_eq!(entry_name(&e), "");
    }
}
