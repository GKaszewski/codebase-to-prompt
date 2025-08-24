# Codebase to Prompt

`codebase-to-prompt` is a Rust-based CLI tool designed to bundle files from a directory into a single output file. It supports filtering files by extensions, respecting `.gitignore` rules, and formatting the output in Markdown, plain text, or console-friendly formats.

## Features

- **File Filtering**: Include or exclude files based on their extensions.
- **Output Formats**: Supports Markdown, plain text, and console-friendly formats.
- **Git Integration**: Optionally append the current Git hash to the output file name.
- **Date Integration**: Optionally append the current date to the output file name.
- **Line Numbers**: Include line numbers in the output.
- **Hidden Files**: Optionally ignore hidden files.
- **`.gitignore` Respect**: Optionally respect `.gitignore` rules.

## Installation

1. Ensure you have [Rust](https://www.rust-lang.org/) installed.
2. Clone this repository:
   ```bash
   git clone https://github.com/GKaszewski/codebase-to-prompt
   cd codebase-to-prompt
   ```
3. Build the project:
   ```bash
   cargo build --release
   ```
4. The binary will be available at `target/release/codebase-to-prompt`.

## Usage

Run the tool with the following options:

```bash
codebase-to-prompt [OPTIONS] [DIRECTORY]
```

### Options

- `-o, --output <FILE>`: Specify the output file. Defaults to stdout if not provided.
- `-i, --include <EXTENSIONS>`: Comma-separated list of file extensions to include.
- `-e, --exclude <EXTENSIONS>`: Comma-separated list of file extensions to exclude.
- `--format <FORMAT>`: Output format (`console`, `markdown`, `text`). Defaults to `console`.
- `-d, --append-date`: Append the current date to the output file name.
- `-g, --append-git-hash`: Append the current Git hash to the output file name.
- `-l, --line-numbers`: Include line numbers in the output.
- `-H, --ignore-hidden`: Ignore hidden files.
- `-R, --respect-gitignore`: Respect `.gitignore` rules. Enabled by default.

### Examples

1. Bundle all `.rs` files in the current directory into `output.md` in Markdown format:

   ```bash
   codebase-to-prompt -o output.md -i rs --format markdown
   ```

2. Bundle all files except `.log` files, appending the current date and Git hash to the output file name:

   ```bash
   codebase-to-prompt -o output.txt -e log -d -g
   ```

3. Output all files to the console, including line numbers:
   ```bash
   codebase-to-prompt -l
   ```

## Development

### Prerequisites

- [Rust](https://www.rust-lang.org/) (latest stable version recommended)

### Building

To build the project:

```bash
cargo build --release
```

### Running Tests

To run tests:

```bash
cargo test
```

### Code Structure

- `src/lib.rs`: Core logic for file processing and bundling.
- `src/main.rs`: CLI entry point.

### Adding Dependencies

To add a new dependency, update `Cargo.toml` and run:

```bash
cargo build
```

### Linting and Formatting

To lint the code:

```bash
cargo clippy
```

To format the code:

```bash
cargo fmt
```

## License

This project is licensed under the MIT License. See the [LICENSE](LICENSE) file for details.

## Contributing

Contributions are welcome! Feel free to open issues or submit pull requests.
