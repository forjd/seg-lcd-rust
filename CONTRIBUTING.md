# Contributing

Thanks for helping improve `seg-lcd-rust`.

## Setup

Install the Rust toolchain from <https://rustup.rs/>.

```bash
cargo run -- 0123456789
cargo run --bin gui
```

## Checks

Run these before opening a pull request:

```bash
cargo fmt --check
cargo test
cargo clippy --all-targets -- -D warnings
```

For GUI-specific changes, also run:

```bash
cargo check --bin gui
```

## Architecture

- Keep shared display behavior in `src/lib.rs`.
- Keep `src/main.rs` focused on CLI argument parsing and IO.
- Keep `src/bin/gui.rs` focused on GUI state and `egui` painting.
- Do not duplicate segment masks, parser behavior, themes, SVG rendering, or
  geometry in binaries.

## Commits

Use Conventional Commits where practical:

```text
feat: add segment editor
fix: preserve decimal points in svg export
docs: clarify gui usage
refactor: share renderer geometry
```

## Generated Files

Root-level SVG exports such as `display.svg` and `gui-display.svg` are ignored.
Commit generated assets only when they are intentional fixtures or docs assets,
such as files under `docs/assets/`.
