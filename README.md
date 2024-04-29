# control-cli

## Usage
```text
A CLI for code controls

Usage: control <COMMAND>

Commands:
  config   Set configuration variables
  control  Run control commands
  help     Print this message or the help of the given subcommand(s)

Options:
  -h, --help     Print help
  -V, --version  Print version
```

## Development
See the following:
- [Learn Rust](https://www.rust-lang.org/learn)
- [Command Line Applications in Rust](https://rust-cli.github.io/book/index.html)

## Dependencies
- [Parser](https://github.com/tree-sitter/tree-sitter/blob/master/lib/binding_rust/README.md)
  - [Example: A tool from mozilla that uses tree_sitter](https://docs.rs/rust-code-analysis/latest/rust_code_analysis/index.html)
- [Command Line Argument Parser](https://docs.rs/clap/latest/clap/)
  - [Examples](https://github.com/clap-rs/clap/tree/master/examples)

## Prerequisites
### Install Rust
See [documentation](https://www.rust-lang.org/learn/get-started).
```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```
### Install Cargo Edit
See [documentation](https://github.com/killercup/cargo-edit).
```bash
cargo install cargo-edit
```
