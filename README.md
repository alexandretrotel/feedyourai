# feedyourai

A command-line tool to combine files from a directory into a single file for LLM processing, with flexible filtering options.

![Demo: fyai combining files in a terminal](https://raw.githubusercontent.com/alexandretrotel/feedyourai/main/assets/fyai.gif)

## Features

- Combines multiple text files into one output file
- Can process a remote git repository in a temporary directory
- Supports configuration via CLI options and config files (TOML)
- Filters files by:
  - Size
  - File extensions (e.g., `.txt`, `.md`)
  - Directory inclusion/exclusion
  - File inclusion/exclusion
  - Optionally respects `.gitignore`/`.ignore` rules and skips hidden files/directories (dot-files); `--no-gitignore` disables both, walking hidden entries too
  - Always respects a `.fyaiignore` file (gitignore syntax), regardless of `--no-gitignore`
- Preserves file boundaries with headers showing filename and size
- Customizable input directory and output file

## Installation

### Install via Cargo

```bash
cargo install feedyourai
```

Or,

```bash
cargo install --git https://github.com/alexandretrotel/feedyourai.git
```

This installs the `feedyourai` binary (and its `fyai` alias) to `~/.cargo/bin/`. Ensure this directory is in your `PATH`.

## Configuration

### Config File

You can specify options in a config file (TOML format):

- **Local config:** `./fyai.toml` (used if present in current directory)
- **Global config:** System config directory, used if no local config found — `$XDG_CONFIG_HOME` if set to an absolute path (any platform), otherwise the platform default (e.g. `~/.config` on Linux, `~/Library/Application Support` on macOS)
- **Precedence:** Local config overrides global config. CLI options override both config files.

To see the exact global config path on your system, run:

```bash
fyai init --global
```

#### Example `fyai.toml`

```toml
directory = "./src"
output = "combined.txt"
include_ext = ["md", "txt"]
exclude_dirs = ["node_modules", "dist"]
min_size = 10240
max_size = 512000
respect_gitignore = true
tree_only = false
human = false
```

All CLI options can be set in the config file. CLI flags always take precedence.

### Path Exclusion via `.fyaiignore`

Drop a `.fyaiignore` file (gitignore syntax) anywhere under the scanned directory to exclude matching paths, as an alternative or complement to `exclude_dirs`/`exclude_files`. Unlike `.gitignore`, it's always respected — `--no-gitignore`/`respect_gitignore: false` has no effect on it, since it's fyai's own dedicated exclude mechanism rather than a git one.

## Usage

### Basic Usage

```bash
fyai            # combine everything in the current directory into fyai.txt
fyai --help     # show all options
```

### Examples

| Goal                                    | Command                                                             |
| ---------------------------------------- | -------------------------------------------------------------------- |
| Only `.txt`/`.md` files, from `./docs`   | `fyai -i ./docs --include-ext txt,md`                                 |
| Exclude `.log`/`.tmp` files              | `fyai --exclude-ext log,tmp`                                          |
| Only specific files, from specific dirs  | `fyai --include-dirs src,docs --include-files README.md,main.rs`      |
| Exclude specific files everywhere        | `fyai --exclude-files LICENSE,config.json`                            |
| Size window: 10KB–500KB, custom output   | `fyai -n 10240 -m 512000 -o ai_input.txt -x dist,node_modules`         |
| Tree only, no file contents              | `fyai --tree-only -o tree.txt`                                        |
| Tree with `tree`-style connector glyphs  | `fyai --tree-only --human -o tree.txt`                                |
| Ignore `.gitignore` and hidden-file rules (include everything) | `fyai --respect-gitignore false`                        |
| Remote repo, specific branch             | `fyai --repo https://github.com/owner/repo.git --repo-branch main`    |
| Remote repo, specific commit             | `fyai --repo https://github.com/owner/repo.git --repo-commit 1234abcd` |
| Generate a config template               | `fyai init`                                                            |

## Output Format

Every run starts with a `- Tree Structure` section. By default it uses a minimal two-space indent:

```
- Tree Structure

src/
  main.rs
  utils/
    helper.rs
```

Pass `--human` (or `human = true` in `fyai.toml`) for `tree`-style connector glyphs instead:

```
- Tree Structure

src
├── main.rs
└── utils/
    └── helper.rs
```

Each source file follows as a heading plus a language-tagged, fenced code block, with a human-readable size:

````
### src/main.rs (1.2 KB)

```rust
fn main() {}
```

### notes.md (66.3 KB)

```markdown
[contents of notes.md]
```
````

The fence widens to four backticks for any file whose own content contains a triple backtick, so the block's end is never ambiguous.

## Performance

The directory is walked once, in parallel, and every file is read and UTF-8-checked in parallel too; output is written through a single buffered writer. Nothing to configure — it's just how `fyai` scans.

## License

GPL-3.0 or later. See [LICENSE](LICENSE) for more details.
