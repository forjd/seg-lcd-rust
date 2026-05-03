#!/bin/sh
set -eu

REPO="Forjd/seg-lcd-rust"
INSTALL_DIR="${INSTALL_DIR:-$HOME/.local/bin}"
VERSION="${VERSION:-latest}"

usage() {
  cat <<'EOF'
Install seg-lcd-rust binaries from GitHub Releases.

Usage:
  install.sh [options]

Options:
  --dir DIR          Install directory (default: ~/.local/bin)
  --version VERSION  Release version, such as v0.2.0 (default: latest)
  --help            Show this help

Environment:
  INSTALL_DIR       Install directory
  VERSION           Release version
EOF
}

while [ "$#" -gt 0 ]; do
  case "$1" in
    --dir)
      [ "$#" -ge 2 ] || {
        echo "error: --dir requires a value" >&2
        exit 2
      }
      INSTALL_DIR="$2"
      shift 2
      ;;
    --version)
      [ "$#" -ge 2 ] || {
        echo "error: --version requires a value" >&2
        exit 2
      }
      VERSION="$2"
      shift 2
      ;;
    --help|-h)
      usage
      exit 0
      ;;
    *)
      echo "error: unknown option: $1" >&2
      usage >&2
      exit 2
      ;;
  esac
done

need_cmd() {
  if ! command -v "$1" >/dev/null 2>&1; then
    echo "error: required command not found: $1" >&2
    exit 1
  fi
}

need_cmd uname
need_cmd mktemp
need_cmd tar
need_cmd chmod

if command -v curl >/dev/null 2>&1; then
  download() {
    curl -fsSL "$1" -o "$2"
  }
elif command -v wget >/dev/null 2>&1; then
  download() {
    wget -qO "$2" "$1"
  }
else
  echo "error: curl or wget is required" >&2
  exit 1
fi

case "$(uname -s)" in
  Linux)
    os="linux"
    ;;
  Darwin)
    os="macos"
    ;;
  *)
    echo "error: unsupported OS: $(uname -s). Use a release archive from GitHub instead." >&2
    exit 1
    ;;
esac

case "$(uname -m)" in
  x86_64|amd64)
    arch="x86_64"
    ;;
  arm64|aarch64)
    arch="aarch64"
    ;;
  *)
    echo "error: unsupported CPU architecture: $(uname -m)" >&2
    exit 1
    ;;
esac

if [ "$os" = "linux" ] && [ "$arch" != "x86_64" ]; then
  echo "error: no Linux $arch binary release is currently published" >&2
  exit 1
fi

if [ "$VERSION" = "latest" ]; then
  release_url="https://github.com/$REPO/releases/latest/download"
  version_label="latest"
else
  release_url="https://github.com/$REPO/releases/download/$VERSION"
  version_label="$VERSION"
fi

asset="seg-lcd-rust-${version_label}-${os}-${arch}.tar.gz"
url="$release_url/$asset"
tmp_dir="$(mktemp -d)"
archive="$tmp_dir/$asset"

cleanup() {
  rm -rf "$tmp_dir"
}
trap cleanup EXIT INT TERM

echo "Downloading $url"
download "$url" "$archive"

mkdir -p "$INSTALL_DIR"
tar -xzf "$archive" -C "$tmp_dir"

package_dir="$(find "$tmp_dir" -type d -name "seg-lcd-rust-*-${os}-${arch}" | head -n 1)"
if [ -z "$package_dir" ]; then
  echo "error: could not find unpacked package directory" >&2
  exit 1
fi

cp "$package_dir/seg-lcd-rust" "$INSTALL_DIR/seg-lcd-rust"
cp "$package_dir/seg-lcd-rust-gui" "$INSTALL_DIR/seg-lcd-rust-gui"
chmod +x "$INSTALL_DIR/seg-lcd-rust" "$INSTALL_DIR/seg-lcd-rust-gui"

cat <<EOF
Installed:
  $INSTALL_DIR/seg-lcd-rust
  $INSTALL_DIR/seg-lcd-rust-gui
EOF

case ":$PATH:" in
  *":$INSTALL_DIR:"*) ;;
  *)
    cat <<EOF

Note: $INSTALL_DIR is not on PATH.
Add this to your shell profile:
  export PATH="$INSTALL_DIR:\$PATH"
EOF
    ;;
esac
