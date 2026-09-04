#!/bin/bash
# Full-sweep r22 (2026-08-25): verify the post-r21 tree — r21 record was
# written 08-25 01:39 plus a late inference.rs edit (01:41, bare type-param
# contra-candidate optimism). Unit tests green (1344/921/2/15).
cd /home/cqh/workspace/typescript-rust || exit 1
echo "=== full sweep r22 start: $(date) ==="
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
echo "=== full sweep r22 end: $(date) ==="
