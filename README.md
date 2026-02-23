# Word Guessing Game

A word is randomly chosen from [words.txt](./src/words.txt) and you must guess it in 6 tries.

## Building

`cargo build -r`

## Running

`cargo run`

## Dependencies

See [Cargo.toml](./Cargo.toml) for details

This project uses:
- [ratatui](https://ratatui.rs/) for TUI
- [rand](https://docs.rs/rand/latest/rand/) for choosing random word
- [clap](https://docs.rs/clap/latest/clap/) for argument parsing
