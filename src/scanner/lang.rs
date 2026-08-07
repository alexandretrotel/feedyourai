//! Maps a file extension to the language identifier used on its Markdown
//! code fence.

/// Returns the fence language tag for `ext` (lower-cased, no leading dot),
/// or `""` if the extension isn't recognized (a plain, untagged fence is
/// still valid Markdown).
pub(crate) fn fence_language(ext: &str) -> &'static str {
    match ext {
        "rs" => "rust",
        "py" | "pyw" => "python",
        "js" | "mjs" | "cjs" => "javascript",
        "jsx" => "jsx",
        "ts" | "mts" | "cts" => "typescript",
        "tsx" => "tsx",
        "go" => "go",
        "java" => "java",
        "kt" | "kts" => "kotlin",
        "c" | "h" => "c",
        "cpp" | "cc" | "cxx" | "hpp" | "hxx" => "cpp",
        "cs" => "csharp",
        "rb" => "ruby",
        "php" => "php",
        "sh" | "bash" | "zsh" => "bash",
        "ps1" => "powershell",
        "yaml" | "yml" => "yaml",
        "json" | "jsonc" => "json",
        "toml" => "toml",
        "xml" => "xml",
        "md" | "markdown" => "markdown",
        "html" | "htm" => "html",
        "css" => "css",
        "scss" | "sass" => "scss",
        "sql" => "sql",
        "swift" => "swift",
        "dart" => "dart",
        "lua" => "lua",
        "r" => "r",
        "pl" | "pm" => "perl",
        "ex" | "exs" => "elixir",
        "erl" => "erlang",
        "hs" => "haskell",
        "scala" => "scala",
        "zig" => "zig",
        "dockerfile" => "dockerfile",
        "makefile" | "mk" => "makefile",
        "graphql" | "gql" => "graphql",
        "proto" => "protobuf",
        "vue" => "vue",
        "svelte" => "svelte",
        _ => "",
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn maps_known_extensions_to_their_fence_language() {
        let cases: &[(&str, &str)] = &[
            ("rs", "rust"),
            ("py", "python"),
            ("pyw", "python"),
            ("js", "javascript"),
            ("mjs", "javascript"),
            ("cjs", "javascript"),
            ("jsx", "jsx"),
            ("ts", "typescript"),
            ("mts", "typescript"),
            ("cts", "typescript"),
            ("tsx", "tsx"),
            ("go", "go"),
            ("java", "java"),
            ("kt", "kotlin"),
            ("kts", "kotlin"),
            ("c", "c"),
            ("h", "c"),
            ("cpp", "cpp"),
            ("cc", "cpp"),
            ("cxx", "cpp"),
            ("hpp", "cpp"),
            ("hxx", "cpp"),
            ("cs", "csharp"),
            ("rb", "ruby"),
            ("php", "php"),
            ("sh", "bash"),
            ("bash", "bash"),
            ("zsh", "bash"),
            ("ps1", "powershell"),
            ("yaml", "yaml"),
            ("yml", "yaml"),
            ("json", "json"),
            ("jsonc", "json"),
            ("toml", "toml"),
            ("xml", "xml"),
            ("md", "markdown"),
            ("markdown", "markdown"),
            ("html", "html"),
            ("htm", "html"),
            ("css", "css"),
            ("scss", "scss"),
            ("sass", "scss"),
            ("sql", "sql"),
            ("swift", "swift"),
            ("dart", "dart"),
            ("lua", "lua"),
            ("r", "r"),
            ("pl", "perl"),
            ("pm", "perl"),
            ("ex", "elixir"),
            ("exs", "elixir"),
            ("erl", "erlang"),
            ("hs", "haskell"),
            ("scala", "scala"),
            ("zig", "zig"),
            ("dockerfile", "dockerfile"),
            ("makefile", "makefile"),
            ("mk", "makefile"),
            ("graphql", "graphql"),
            ("gql", "graphql"),
            ("proto", "protobuf"),
            ("vue", "vue"),
            ("svelte", "svelte"),
        ];

        for (ext, expected) in cases {
            assert_eq!(
                fence_language(ext),
                *expected,
                "extension {ext:?} should map to {expected:?}"
            );
        }
    }

    #[test]
    fn returns_empty_string_for_unknown_extensions() {
        assert_eq!(fence_language("xyz"), "");
        assert_eq!(fence_language(""), "");
        assert_eq!(fence_language("unknownext"), "");
    }
}
