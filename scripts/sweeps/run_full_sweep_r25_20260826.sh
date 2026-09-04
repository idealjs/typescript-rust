#!/bin/bash
# Full-sweep r25 (2026-08-25 night): verify Fix 8 (optional-param |undefined),
# Fix 7a (static this), Fix 7b (qualified-name heritage + boxed heritage
# members), Fix 7c (intersection structural fall-through + Go union/intersection
# order), union-callee call signatures, new-expression explicit type-argument
# substitution, comma-expression typing, logical-assignment RHS frame at first
# traversal. Unit tests green (1350/937/2/15).
cd /home/cqh/workspace/typescript-rust || exit 1
echo "=== full sweep r25 start: $(date) ==="
cargo test --test submodule_compiler --no-run 2>&1 | tail -1
cargo test --test submodule_transpile --no-run 2>&1 | tail -1
echo "--- [1/3] compiler suite ---"
TSOX_SUBMODULE_START=0 TSOX_SUBMODULE_END=6536 TSOX_SUBMODULE_JOBS=12 TSOX_SUBMODULE_TIMEOUT_SECS=30 cargo test --test submodule_compiler 2>&1
echo "compiler exit: $?"
echo "--- [2/3] conformance suite ---"
TSOX_SUBMODULE_SUITE=conformance TSOX_SUBMODULE_START=0 TSOX_SUBMODULE_END=5907 TSOX_SUBMODULE_JOBS=12 TSOX_SUBMODULE_TIMEOUT_SECS=30 cargo test --test submodule_compiler 2>&1
echo "conformance exit: $?"
echo "--- [3/3] transpile suite ---"
TSOX_TRANSPILE_JOBS=8 TSOX_TRANSPILE_TIMEOUT_SECS=30 cargo test --test submodule_transpile 2>&1
echo "transpile exit: $?"
echo "=== full sweep r25 end: $(date) ==="
