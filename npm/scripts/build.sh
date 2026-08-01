#!/usr/bin/env bash
# Build the Rust binary and copy it to the npm package's bin directory.
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
PKG_DIR="$(dirname "$SCRIPT_DIR")"
ROOT_DIR="$(dirname "$PKG_DIR")"

echo "Building Rust binary..."
cd "$ROOT_DIR"
cargo build --release

echo "Copying binary to npm/bin/..."
cp target/release/tsox "$PKG_DIR/bin/tsgo"
chmod +x "$PKG_DIR/bin/tsgo"

echo "Done. Binary at $PKG_DIR/bin/tsgo"
