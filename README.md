# FeedYourAI

A command-line tool to combine files from a directory into a single file for AI processing, with flexible filtering options.

## Features

- Combines multiple text files into one output file
- Filters files by:
  - Size
  - File extensions (e.g., `.txt`, `.md`)
- Preserves file boundaries with headers showing filename and size
- Customizable input directory and output file

## Installation

### Prerequisites

- [Rust](https://www.rust-lang.org/tools/install) (latest stable version recommended)

### Install via Cargo

```bash
cargo install feedyourai
```

Or,

```bash
cargo install --git https://github.com/alexandretrotel/feedyourai.git
```

This installs the `fyai` binary to `~/.cargo/bin/`. Ensure this directory is in your `PATH`.

## Usage

Run `fyai` in your terminal to combine files:

### Basic Usage

```bash
fyai
```

- Combines all files from the current directory into `feedyourai.txt`

### Options

```
USAGE:
    fyai [OPTIONS]

OPTIONS:
    -d, --dir <DIR>          Sets the input directory [default: .]
    
    -o, --output <FILE>      Sets the output file [default: feedyourai.txt]

    -e, --ext <EXT>          Comma-separated list of file extensions to exclude (e.g., txt,md)
    -x, --exclude-dirs <DIRS> Comma-separated list of directories to exclude (e.g., src,tests)
    -n, --min-size <BYTES>   Exclude files smaller than this size in bytes (default: 51200)
    -m, --max-size <BYTES>   Exclude files larger than this size in bytes

    --tree-only              Only output the project directory tree, no file contents

    -h, --help               Print help information
    -V, --version            Print version information
```

### Examples

- Combine `.txt` and `.md` files from a specific directory:

  ```bash
  fyai -d ./docs -e txt,md
  ```

- Include all files (no size minimum) up to 1MB:

  ```bash
  fyai -n 0 -m 1048576
  ```

- Custom output file with files between 10KB and 500KB, excluding `list` and `node_modules`:
  ```bash
  fyai -n 10240 -m 512000 -o ai_input.txt -x dist, node_modules
  ```

- Output only the project directory structure (no file contents):
  ```bash
  fyai --tree-only -o tree.txt
  ```

## Output Format

The combined file includes headers for each source file:

```
=== File: example.txt (12345 bytes) ===
[contents of example.txt]

=== File: notes.md (67890 bytes) ===
[contents of notes.md]
```

## Contributing

Contributions are welcome! Please:

1. Fork the repository
2. Create a feature branch
3. Submit a pull request

## License

This project is licensed under the MIT License.

## Acknowledgments

- Built with [Rust](https://www.rust-lang.org/)
- Uses [clap](https://crates.io/crates/clap) for command-line parsing
