#!/bin/bash
# Full-sweep r36 (2026-08-28): r35 post-fix verification — ambient-module
# nameability whitelist (TS2883), require-check format refinements (.d.ts
# target never ESM / .d.ts importer CJS), dynamic import() namespace typing
# with re-export clause chasing. Expected: compiler 0F, conformance 1F
# (templateLiteralTypes1, D8-1 on file), transpile 0. Workers 12→8 (host
# core count reduced). Unit tests green (1353/1004/2/15).
cd /home/cqh/workspace/typescript-rust || exit 1
echo "=== full sweep r36 start: $(date) ==="
cargo test --test submodule_compiler --no-run 2>&1 | tail -1
cargo test --test submodule_transpile --no-run 2>&1 | tail -1
echo "--- [1/3] compiler suite ---"
TSOX_SUBMODULE_START=0 TSOX_SUBMODULE_END=6536 TSOX_SUBMODULE_JOBS=8 TSOX_SUBMODULE_TIMEOUT_SECS=30 cargo test --test submodule_compiler 2>&1
echo "compiler exit: $?"
echo "--- [2/3] conformance suite ---"
TSOX_SUBMODULE_SUITE=conformance TSOX_SUBMODULE_START=0 TSOX_SUBMODULE_END=5907 TSOX_SUBMODULE_JOBS=8 TSOX_SUBMODULE_TIMEOUT_SECS=30 cargo test --test submodule_compiler 2>&1
echo "conformance exit: $?"
echo "--- [3/3] transpile suite ---"
TSOX_TRANSPILE_JOBS=8 TSOX_TRANSPILE_TIMEOUT_SECS=30 cargo test --test submodule_transpile 2>&1
echo "transpile exit: $?"
echo "=== full sweep r36 end: $(date) ==="
