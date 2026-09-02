#!/bin/bash
# Full-sweep r33 (2026-08-27): D7 recursiveReverseMappedType verification —
# Go-aligned tri-state conditional resolution (permissive/restrictive
# definitely-true/false probes), creation-context snapshots on deferred
# conditionals (type_argument_stack + scope chain), same-root conditional
# inference fast path, and dual deferred-conditional source/target relation
# fallbacks. ExportsSourceTs timeout expected GONE (D7 fixed the pathological
# re-resolution); remaining expected: D6a TS2883 x4, D6b PackagePattern
# node16/18 (+ test.d.* parse mystery), D8 templateLiteralTypes1.
# Unit tests green (1353/982/2/15).
cd /home/cqh/workspace/typescript-rust || exit 1
echo "=== full sweep r33 start: $(date) ==="
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
echo "=== full sweep r33 end: $(date) ==="
