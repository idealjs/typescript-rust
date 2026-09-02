#!/bin/bash
# Full-sweep r35 (2026-08-28): TS2590 cross-product union cap (template /
# variadic tuple / intersection), TS1338 infer placement, TS1479/TS1471
# CJS<->ESM format checks, TS2883 declaration nameability (approximation),
# mapped-type written-form display, imported-variable types + re-export
# clause chasing + bare package-name resolution in type_of_imported_symbol.
# Unit tests green (1353/999/2/15).
cd /home/cqh/workspace/typescript-rust || exit 1
echo "=== full sweep r35 start: $(date) ==="
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
echo "=== full sweep r35 end: $(date) ==="
