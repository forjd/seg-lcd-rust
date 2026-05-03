# seg-lcd-rust

[![CI](https://github.com/Forjd/seg-lcd-rust/actions/workflows/ci.yml/badge.svg)](https://github.com/Forjd/seg-lcd-rust/actions/workflows/ci.yml)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](LICENSE)

A Rust simulator for seven-segment LCD displays like the digits used in basic
calculators, digital clocks, and classic digital watches.

![Seven-segment LCD preview](docs/assets/preview.svg)

It includes:

- a terminal renderer
- a browser-viewable SVG exporter
- a native `egui` desktop GUI
- a shared library for parsing text, segment masks, themes, geometry, terminal
  rendering, and SVG rendering

## Status

Early development. The CLI, SVG exporter, and native GUI are usable, but the API
is not yet stable.

## Usage

```bash
cargo run -- 0123456789
cargo run --bin gui
cargo run -- --labels HELP
cargo run -- --inverse 10:58.42
cargo run -- 12:34.5 --masks
cargo run -- --svg display.svg 0123456789
cargo run -- --svg amber.svg --theme amber 10:58.42
cargo run -- --svg blue.svg --theme blue --glow 88:88.88
cargo run -- --svg custom.svg --on 102418 --off 6b7a62 --bg dbe5d2 --panel c3d0ba --inactive-opacity 0.18 1234
```

`cargo run -- ...` runs the CLI by default. Use `cargo run --bin gui` for the
desktop app.

## GUI

Run the desktop GUI with:

```bash
cargo run --bin gui
```

The GUI provides a live LCD preview, editable display text, theme selection,
color controls, inactive-segment opacity, glow and glass toggles, and an SVG
export button that writes `gui-display.svg`.

## Library

The shared library can parse display text and render terminal or SVG output:

```rust
use seg_lcd_rust::{LcdStyle, TerminalStyle, render_svg, render_text};

let text = "12:34.5";
let terminal = render_text(text, TerminalStyle::default());
let svg = render_svg(text, LcdStyle::default());
```

## Structure

- `src/lib.rs` contains the shared segment model, parser, themes, terminal
  renderer, SVG renderer, and segment geometry.
- `src/main.rs` is the CLI wrapper.
- `src/bin/gui.rs` is the native `egui` desktop app.

## Checks

```bash
cargo fmt --check
cargo test
cargo clippy --all-targets -- -D warnings
```

## Releases

Releases are automated with Release Please from Conventional Commits. Changes
merged to `main` update a release PR; merging that PR creates the GitHub Release,
updates `CHANGELOG.md`, and bumps the crate version.

Pull requests should use a Conventional Commit title such as `feat: add segment
editor` or `fix(svg): preserve decimal points`.

## Options

- `--labels` renders segment names (`A` through `G`) instead of filled cells.
- `--inverse` renders active LCD segments as clear space against inactive blocks.
- `--masks` prints each character's seven-bit segment mask.
- `--svg <path>` writes a browser-viewable SVG rendering with faint inactive
  LCD segments.
- `--theme <name>` applies an SVG theme: `classic`, `green`, `amber`, `blue`,
  or `negative`.
- `--on <hex>`, `--off <hex>`, `--bg <hex>`, and `--panel <hex>` override SVG
  colors. Hex values can be written with or without `#`.
- `--inactive-opacity <number>` controls how visible inactive SVG segments are,
  from `0.0` to `1.0`.
- `--glow`, `--no-glow`, and `--no-glass` toggle SVG display effects.

Supported characters are digits, a small set of seven-segment-friendly letters,
space, `-`, `_`, `.`, and `:`.

## License

MIT. See [LICENSE](LICENSE).
