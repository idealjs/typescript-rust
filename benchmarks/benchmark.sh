#!/usr/bin/env bash
# Benchmark: Go oracle (tsgo) vs Rust (tsox) CLI cold run + type check.
#
# Usage: benchmarks/benchmark.sh [file.ts ...]
# If no files given, uses /tmp/parity_test/type_error.ts
set -euo pipefail

TSOX="${TSOX:-/Users/cqh/workspace/typescript-rust/target/release/tsox}"
TSGO="${TSGO:-/Users/cqh/workspace/typescript-go/built/local/tsgo}"
FILES=("$@")
if [ ${#FILES[@]} -eq 0 ]; then
  FILES=(/tmp/parity_test/type_error.ts)
fi

echo "=== tsgo (Go oracle) ==="
time "$TSGO" "${FILES[@]}" --noEmit 2>&1 | tail -1
echo ""
echo "=== tsox (Rust) ==="
time "$TSOX" "${FILES[@]}" --noEmit 2>&1 | tail -1
