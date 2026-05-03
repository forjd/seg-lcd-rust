# Contributing

Thanks for helping improve `seg-lcd-rust`.

## Setup

Install the Rust toolchain from <https://rustup.rs/>.

```bash
git config core.hooksPath scripts/git-hooks
cargo run -- 0123456789
cargo run --bin seg-lcd-rust-gui
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
cargo check --bin seg-lcd-rust-gui
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

This repo includes a versioned `commit-msg` hook. Enable it with:

```bash
git config core.hooksPath scripts/git-hooks
```

Pull requests also check that the PR title follows Conventional Commits, which
keeps squash-merged commits release-friendly.

## Releases

Releases are automated with Release Please. Conventional Commits merged to
`main` are used to open or update a release PR. Merging that release PR creates
the GitHub Release, updates `CHANGELOG.md`, and bumps `Cargo.toml`.

Common release-driving commit types:

- `fix:` creates a patch release.
- `feat:` creates a minor release.
- `feat!:` or a `BREAKING CHANGE:` footer creates a major release.
- `docs:`, `test:`, `refactor:`, and `chore:` are included in history but do
  not normally trigger a release by themselves.

## Generated Files

Root-level SVG exports such as `display.svg` and `gui-display.svg` are ignored.
Commit generated assets only when they are intentional fixtures or docs assets,
such as files under `docs/assets/`.
