#!/usr/bin/env bash
# Build the Rust binary and copy it to the npm package's bin directory.
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
PKG_DIR="$(dirname "$SCRIPT_DIR")"
ROOT_DIR="$(dirname "$PKG_DIR")"

# Detect current platform
OS="$(uname -s)"
ARCH="$(uname -m)"

case "$OS" in
    Darwin) PLATFORM="darwin" ;;
    Linux)  PLATFORM="linux" ;;
    MINGW*|MSYS*|CYGWIN*) PLATFORM="win32" ;;
    *) echo "Unknown OS: $OS"; exit 1 ;;
esac

case "$ARCH" in
    x86_64|amd64) ARCH="x64" ;;
    arm64|aarch64) ARCH="arm64" ;;
    *) echo "Unknown arch: $ARCH"; exit 1 ;;
esac

echo "Building Rust binary (platform: $PLATFORM-$ARCH)..."
cd "$ROOT_DIR"
cargo build --release

# Copy with platform-specific name
SOURCE="$ROOT_DIR/target/release/tsox"
if [ "$PLATFORM" = "win32" ]; then
    SOURCE="$ROOT_DIR/target/release/tsox.exe"
fi

PLATFORM_BIN="$PKG_DIR/bin/tsox-$PLATFORM-$ARCH"
mkdir -p "$PKG_DIR/bin"
cp "$SOURCE" "$PLATFORM_BIN"
chmod +x "$PLATFORM_BIN"

# Also copy as the generic name (for single-platform builds)
cp "$SOURCE" "$PKG_DIR/bin/tsox"
chmod +x "$PKG_DIR/bin/tsox"

echo "Done. Binary at:"
echo "  $PLATFORM_BIN"
echo "  $PKG_DIR/bin/tsox"
