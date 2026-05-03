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
  download_optional() {
    curl -fsSL "$1" -o "$2" >/dev/null 2>&1
  }
elif command -v wget >/dev/null 2>&1; then
  download() {
    wget -qO "$2" "$1"
  }
  download_optional() {
    wget -qO "$2" "$1" >/dev/null 2>&1
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
  metadata="$(mktemp)"
  download "https://api.github.com/repos/$REPO/releases/latest" "$metadata"
  VERSION="$(sed -n 's/.*"tag_name":[[:space:]]*"\([^"]*\)".*/\1/p' "$metadata" | head -n 1)"
  rm -f "$metadata"
  if [ -z "$VERSION" ]; then
    echo "error: could not resolve latest release version" >&2
    exit 1
  fi
fi

release_url="https://github.com/$REPO/releases/download/$VERSION"
asset="seg-lcd-rust-${VERSION}-${os}-${arch}.tar.gz"
url="$release_url/$asset"
tmp_dir="$(mktemp -d)"
archive="$tmp_dir/$asset"
checksums="$tmp_dir/SHA256SUMS.txt"

cleanup() {
  rm -rf "$tmp_dir"
}
trap cleanup EXIT INT TERM

echo "Downloading $url"
download "$url" "$archive"

if download_optional "$release_url/SHA256SUMS.txt" "$checksums"; then
  expected="$(grep "[[:space:]]$asset$" "$checksums" | awk '{print $1}' | head -n 1)"
  if [ -z "$expected" ]; then
    echo "error: SHA256SUMS.txt does not contain $asset" >&2
    exit 1
  fi

  if command -v sha256sum >/dev/null 2>&1; then
    actual="$(sha256sum "$archive" | awk '{print $1}')"
  elif command -v shasum >/dev/null 2>&1; then
    actual="$(shasum -a 256 "$archive" | awk '{print $1}')"
  else
    echo "error: SHA256SUMS.txt is available, but sha256sum or shasum was not found" >&2
    exit 1
  fi

  if [ "$actual" != "$expected" ]; then
    echo "error: checksum verification failed for $asset" >&2
    exit 1
  fi
  echo "Verified checksum for $asset"
else
  echo "Warning: no SHA256SUMS.txt found for $VERSION; skipping checksum verification" >&2
fi

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
