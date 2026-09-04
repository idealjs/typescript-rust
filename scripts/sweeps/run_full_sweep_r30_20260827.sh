#!/bin/bash
# Full-sweep r30 (2026-08-27): verification round for the r29-regression
# fixes (signature/container type-parameter scoping: call/construct-sig
# push_scope, method push, ancestry chain for class type params + signature
# kinds, check_type_annotation declaring context, TS2708 string-module gate,
# type-parameter/value coexistence merge, mapped-type open-keyset deferral,
# name-frame symbol-keyed substitution, interface fork instantiation,
# new-expression member-only target parse) + D3-sig. Unit tests green
# (1353/972/2/15).
cd /home/cqh/workspace/typescript-rust || exit 1
echo "=== full sweep r30 start: $(date) ==="
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
echo "=== full sweep r30 end: $(date) ==="
