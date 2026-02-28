# Repository Guidelines

## Development Commands

- `cargo check`: fast compile check.
- `cargo clippy`: fast style check.
- `cargo +nightly fmt --all`: apply Rust formatting with nightly rustfmt.

## Coding Style

- Rust edition is `2024`; follow idiomatic Rust and keep APIs small and explicit.
  - Use `if let` chains when they improve clarity.
