# seven-seg-lcd

A small Rust terminal simulator for seven-segment LCD displays like the digits used
in basic calculators, digital clocks, and classic digital watches.

## Usage

```bash
cargo run -- 0123456789
cargo run -- --labels HELP
cargo run -- --inverse 10:58.42
cargo run -- 12:34.5 --masks
cargo run -- --svg display.svg 0123456789
cargo run -- --svg amber.svg --theme amber 10:58.42
cargo run -- --svg blue.svg --theme blue --glow 88:88.88
cargo run -- --svg custom.svg --on 102418 --off 6b7a62 --bg dbe5d2 --panel c3d0ba --inactive-opacity 0.18 1234
```

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
