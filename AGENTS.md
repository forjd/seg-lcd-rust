# AGENTS.md

## Project

`seg-lcd-rust` is a Rust seven-segment LCD simulator. It has:

- a CLI binary at `src/main.rs`
- a native `egui` GUI binary at `src/bin/gui.rs`
- shared parser, segment model, themes, geometry, terminal rendering, and SVG
  rendering in `src/lib.rs`

Keep core display behavior in `src/lib.rs` so the CLI and GUI stay consistent.

## Commands

In this environment, prefix shell commands with `rtk`.

```bash
rtk cargo fmt --check
rtk cargo test
rtk cargo clippy --all-targets -- -D warnings
rtk cargo run -- 0123456789
rtk cargo run -- --svg display.svg --theme amber 10:58.42
rtk cargo run --bin seg-lcd-rust-gui
rtk git config core.hooksPath scripts/git-hooks
```

Use `cargo run -- ...` for the CLI. `Cargo.toml` sets `default-run =
"seg-lcd-rust"` so the CLI remains the default even though the GUI binary also
exists.

## Development Notes

- Do not duplicate segment masks, text parsing, theme definitions, SVG rendering,
  or segment geometry in binaries.
- Keep `src/main.rs` focused on CLI argument parsing and IO.
- Keep `src/bin/gui.rs` focused on GUI state and `egui` painting.
- SVG export should use `render_svg` from the shared library.
- If adding supported characters, update `segment_mask` and tests in
  `src/lib.rs`.
- Generated SVGs such as `display.svg` or `gui-display.svg` are local artifacts
  unless intentionally committed as fixtures.
- Commit messages should follow Conventional Commits. The versioned
  `scripts/git-hooks/commit-msg` hook enforces this locally when
  `core.hooksPath` is configured.
- Releases are managed by Release Please from Conventional Commits on `main`.
  Do not hand-edit release PR changes unless the release automation needs a
  targeted fix.

## Verification

Before handing off changes, run:

```bash
rtk cargo fmt --check
rtk cargo test
rtk cargo clippy --all-targets -- -D warnings
```

For GUI changes, also run:

```bash
rtk cargo check --bin seg-lcd-rust-gui
```
