#!/bin/bash
# Full-sweep run of all three official suites (2026-08-19).
# compiler (6537) → conformance (5907) → transpile (22), sequential, 12 workers.
cd /home/cqh/workspace/typescript-rust || exit 1

echo "=== full sweep start: $(date) ==="

echo "--- [1/3] compiler suite ---"
TSOX_SUBMODULE_START=0 TSOX_SUBMODULE_END=6536 TSOX_SUBMODULE_JOBS=12 \
  TSOX_SUBMODULE_TIMEOUT_SECS=30 \
  cargo test --test submodule_compiler 2>&1
echo "compiler exit: $?"

echo "--- [2/3] conformance suite ---"
TSOX_SUBMODULE_SUITE=conformance TSOX_SUBMODULE_START=0 TSOX_SUBMODULE_END=5907 \
  TSOX_SUBMODULE_JOBS=12 TSOX_SUBMODULE_TIMEOUT_SECS=30 \
  cargo test --test submodule_compiler 2>&1
echo "conformance exit: $?"

echo "--- [3/3] transpile suite ---"
TSOX_TRANSPILE_JOBS=8 TSOX_TRANSPILE_TIMEOUT_SECS=30 \
  cargo test --test submodule_transpile 2>&1
echo "transpile exit: $?"

echo "=== full sweep end: $(date) ==="
