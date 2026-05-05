---
name: seg-lcd-rust
description: Render seven-segment LCD text in terminal, SVG, or PNG using the seg-lcd-rust CLI.
version: 1.0.0
author: Hermes Agent
license: MIT
metadata:
  hermes:
    tags: [seven-segment, lcd, svg, terminal, cli, rust]
---

# seg-lcd-rust

Use this skill when the user asks to render seven-segment/LCD-style digits or text, generate SVG previews, test the `seg-lcd-rust` CLI, or work on the `forjd/seg-lcd-rust` repository.

## Installed CLI

The release installer installs binaries to `~/.local/bin`:

- `seg-lcd-rust` — terminal renderer, SVG exporter, and native PNG exporter
- `seg-lcd-rust-gui` — native egui desktop GUI; requires a graphical display (`DISPLAY`, Wayland, etc.)

Install/update from GitHub Releases:

```bash
curl -fsSL https://raw.githubusercontent.com/Forjd/seg-lcd-rust/main/install.sh | sh
```

Install to a custom directory:

```bash
curl -fsSL https://raw.githubusercontent.com/Forjd/seg-lcd-rust/main/install.sh | sh -s -- --dir ~/.local/bin
```

Install a specific version:

```bash
curl -fsSL https://raw.githubusercontent.com/Forjd/seg-lcd-rust/main/install.sh | sh -s -- --version v0.6.0
```

## CLI Usage

Basic terminal render:

```bash
seg-lcd-rust '10:58.42'
```

Labels mode:

```bash
seg-lcd-rust --labels HELP
```

Print segment masks:

```bash
seg-lcd-rust '12:34.5' --masks
```

Custom segment masks:

```bash
seg-lcd-rust --mask ABDEG --mask BCG --labels
```

Inverse terminal render:

```bash
seg-lcd-rust --inverse '10:58.42'
```

SVG export:

```bash
seg-lcd-rust --svg display.svg --theme amber '10:58.42'
seg-lcd-rust --svg blue.svg --theme blue --glow '88:88.88'
seg-lcd-rust --svg custom.svg --on 102418 --off 6b7a62 --bg dbe5d2 --panel c3d0ba --inactive-opacity 0.18 1234
```

PNG export:

```bash
seg-lcd-rust --png blue.png --theme blue --glow '88:88.88'
seg-lcd-rust --png amber.png --theme amber '12:34.5'
seg-lcd-rust --png custom.png --on b86cff --off 2b1748 --bg 08030f --panel 160820 --glow '42.0'
```

Native PNG output rasterizes the existing SVG renderer, so SVG and PNG output should stay visually consistent.

Available themes observed in help: `classic`, `green`, `amber`, `blue`, `negative`.

## PNG Output

Use native `--png PATH` when working from this branch or any release that includes PNG export:

```bash
png="$HOME/.hermes/tmp/seg-lcd-example.png"
mkdir -p "$(dirname "$png")"
seg-lcd-rust --png "$png" --theme blue --glow '12:34.5'
file "$png"
```

For older releases such as v0.6.0 that lack `--png`, first generate SVG, then rasterize it with `scripts/svg-to-png.sh INPUT.svg OUTPUT.png` or a headless browser. The helper currently accepts exactly two arguments; do not pass a size like `1200x400` unless the script has been updated.

## Verification Commands

```bash
command -v seg-lcd-rust
seg-lcd-rust --help
seg-lcd-rust '10:58.42'
out=$(mktemp --suffix=.svg)
seg-lcd-rust --svg "$out" --theme amber '10:58.42'
test -s "$out"
png=$(mktemp --suffix=.png)
seg-lcd-rust --png "$png" --theme blue --glow '10:58.42'
file "$png" | grep 'PNG image data'
```

## Known Behaviors / Pitfalls

- `seg-lcd-rust --version` is not supported in v0.6.0; it exits `2` with `unknown option: --version`.
- `--help` text currently says `Usage: cargo run -- [OPTIONS] [TEXT]` even for the installed binary; use `seg-lcd-rust [OPTIONS] [TEXT]` when explaining installed usage.
- SVG and PNG export also print a terminal preview and `wrote PATH` to stdout.
- `seg-lcd-rust-gui` cannot run in headless environments without `DISPLAY`, `WAYLAND_DISPLAY`, or `WAYLAND_SOCKET`.
- Support script: `scripts/svg-to-png.sh` converts generated SVGs to PNG using `rsvg-convert`, ImageMagick, Google Chrome, or Chromium, whichever is available.

## Repository Development Notes

Session reference: `references/png-export-pr-notes.md` captures the completed local PNG export implementation, verification commands/results, and GitHub PR/auth state from the original work session.

Repository: https://github.com/forjd/seg-lcd-rust

### Native PNG export implementation pattern

If adding or revisiting native PNG support, prefer reusing the SVG rendering pipeline rather than duplicating geometry/rasterization:

1. Add dependencies for non-WASM builds or general native builds: `resvg`, `usvg`, and `tiny-skia`.
2. Add library helpers around existing SVG output:
   - `render_png(text, style) -> Result<Vec<u8>, String>`
   - `render_cells_png(cells, style) -> Result<Vec<u8>, String>`
   - `render_svg_to_png(svg) -> Result<Vec<u8>, String>`
3. Implement `render_svg_to_png` with:
   - `usvg::Tree::from_str(svg, &usvg::Options::default())`
   - `tree.size().to_int_size()`
   - `tiny_skia::Pixmap::new(width, height)`
   - `resvg::render(&tree, tiny_skia::Transform::identity(), &mut pixmap.as_mut())`
   - `pixmap.encode_png()`
4. Add `--png PATH` parsing in `src/main.rs`, write bytes with `fs::write`, and print `wrote PATH` matching SVG behavior.
5. Add integration tests under `tests/` using `env!("CARGO_BIN_EXE_seg-lcd-rust")`; assert the output starts with the PNG signature `\x89PNG\r\n\x1a\n` and that missing `--png` path reports `--png requires an output path`.

If GitHub auth is absent, commit locally and export `git format-patch origin/main..HEAD --stdout` so the user can apply/push the work.

If editing the repo, follow `AGENTS.md`:

- CLI binary: `src/main.rs`
- GUI binary: `src/bin/gui.rs`
- Shared parser, segment model, themes, geometry, terminal rendering, and SVG rendering: `src/lib.rs`
- Keep core display behavior in `src/lib.rs` so CLI and GUI stay consistent.
- Use `cargo run -- ...` for local CLI development; `Cargo.toml` sets `default-run = "seg-lcd-rust"`.
- Release Please manages releases from Conventional Commits on `main`.

Verification from `AGENTS.md` expects `rtk` in that environment:

```bash
rtk cargo fmt --check
rtk cargo test
rtk cargo clippy --all-targets -- -D warnings
```

For GUI changes also run:

```bash
rtk cargo check --bin seg-lcd-rust-gui
```

If `rtk` or `cargo` is not installed in the current environment, report that clearly rather than guessing test results.
