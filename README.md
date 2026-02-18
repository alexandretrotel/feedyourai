# fyai

A command-line tool to combine files from a directory into a single file for AI processing, with flexible filtering options.

![Demo: fyai combining files in a terminal](./assets/fyai.gif)

## Features

- Combines multiple text files into one output file
- Can process a remote git repository in a temporary directory
- Supports configuration via CLI options and config files (YAML)
- Filters files by:
  - Size
  - File extensions (e.g., `.txt`, `.md`)
  - Directory inclusion/exclusion
  - File inclusion/exclusion
  - Optionally respects `.gitignore` rules (can be disabled)
- Preserves file boundaries with headers showing filename and size
- Customizable input directory and output file

## Installation

### Install via Cargo

```bash
cargo install feedyourai
```

Or,

```bash
cargo install --git https://github.com/atrtde/feedyourai.git
```

This installs the `fyai` binary to `~/.cargo/bin/`. Ensure this directory is in your `PATH`.

## Usage

Run `fyai` in your terminal to combine files:

### Config File Support

You can specify options in a config file (YAML format):

- **Local config:** `./fyai.yaml` (used if present in current directory)
- **Global config:** System config directory (platform-specific), used if no local config found
- **Precedence:** Local config overrides global config. CLI options override both config files.

To see the exact global config path on your system, run:

```bash
fyai init --global
```

#### Example `fyai.yaml`

```yaml
directory: ./src
output: combined.txt
include_ext:
  - md
  - txt
exclude_dirs:
  - node_modules
  - dist
min_size: 10240
max_size: 512000
respect_gitignore: true
tree_only: false
```

All CLI options can be set in the config file. CLI flags always take precedence.

### Basic Usage

```bash
fyai
fyai --help # show help
```

- Combines all files from the current directory into `fyai.txt`

### Examples

- Combine only `.txt` and `.md` files from a specific directory:

  ```bash
  fyai -d ./docs --include-ext txt,md
  ```

- Exclude all `.log` and `.tmp` files from the output:

  ```bash
  fyai --exclude-ext log,tmp
  ```

- Include only files named `README.md` and `main.rs` from the `src` and `docs` directories:

  ```bash
  fyai --include-dirs src,docs --include-files README.md,main.rs
  ```

- Exclude all files named `LICENSE` and `config.json` from any directory:

  ```bash
  fyai --exclude-files LICENSE,config.json
  ```

- Include all files (no size minimum) up to 1MB:

  ```bash
  fyai -n 0 -m 1048576
  ```

- Custom output file with files between 10KB and 500KB, excluding `dist` and `node_modules` directories:

  ```bash
  fyai -n 10240 -m 512000 -o ai_input.txt -x dist,node_modules
  ```

- Output only the project directory structure (no file contents):

  ```bash
  fyai --tree-only -o tree.txt
  ```

- Ignore .gitignore rules and include all files (even those normally excluded):
  ```bash
  fyai --respect-gitignore false
  ```

- Run against a remote GitHub/GitLab repository without leaving the clone on disk:

  ```bash
  fyai --repo https://github.com/owner/repo.git --repo-branch main
  ```

- Run against a specific commit:

  ```bash
  fyai --repo https://github.com/owner/repo.git --repo-commit 1234abcd
  ```

- Generate a config template:

  ```bash
  fyai init
  ```

## Output Format

The combined file includes headers for each source file:

```
- File: example.txt (12345 bytes)
[contents of example.txt]

- File: notes.md (67890 bytes)
[contents of notes.md]
```

## Contributing

Contributions are welcome! Please:

1. Fork the repository
2. Create a feature branch
3. Submit a pull request

## License

MIT. See [LICENSE](LICENSE) for more details.

## Changelog

See `CHANGELOG.md` for release notes.

## Acknowledgments

- Built with [Rust](https://www.rust-lang.org/)
- Uses [clap](https://crates.io/crates/clap) for command-line parsing
