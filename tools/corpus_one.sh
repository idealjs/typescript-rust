#!/bin/bash
# 单用例调试：tools/corpus_one.sh <basename 不带扩展名或带 .ts> [suite]
# 输出 PASS/FAIL/SKIP + 基线 diff
set -eu
name="$1"
SUITE=${2:-compiler}
ROOT="$(cd "$(dirname "$0")/.." && pwd)"
BIN=${BIN:-$(ls -t "$ROOT"/target/release/deps/corpus-* | grep -v '\.d$' | head -1)}
cd "$ROOT/crates/tsox" || exit 1
rm -f "tests/corpus/baselines/local/$SUITE/$name.errors.txt" "tests/corpus/baselines/local/$SUITE/$name.ts.errors.txt"
out=$(TSOX_SUBMODULE_SUITE=$SUITE TSOX_SUBMODULE_LIMIT=0 TSOX_SUBMODULE_FILTER="$name" timeout 60 "$BIN" --exact submodule_compiler::submodule_compiler_cases --nocapture 2>&1 | grep -E '^\[w0\]|panicked' | head -5)
echo "$out"
for f in "tests/corpus/baselines/local/$SUITE/$name.errors.txt" "tests/corpus/baselines/local/$SUITE/$name.ts.errors.txt"; do
  if [[ -f "$f" ]]; then
    ref="tests/corpus/testdata/baselines/reference/$SUITE/$(basename "$f")"
    if [[ -f "$ref" ]]; then
      echo "--- diff ref vs local ($(basename "$f")) ---"
      diff "$ref" "$f" | head -60
    else
      echo "--- 无参考基线（期望零错误），local 输出: ---"
      head -20 "$f"
    fi
  fi
done
