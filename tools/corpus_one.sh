#!/bin/bash
# 单用例调试：tools/corpus_one.sh <basename 不带扩展名或带 .ts> [suite]
# 输出 PASS/FAIL/SKIP + 基线 diff
set -eu
name="$1"
SUITE=${2:-compiler}
ROOT="$(cd "$(dirname "$0")/.." && pwd)"
BIN=${BIN:-$(ls -t "$ROOT"/target/release/deps/corpus-* | grep -v '\.d$' | head -1)}
cd "$ROOT/crates/tsox" || exit 1
stem="${name%.*}"
rm -f "tests/corpus/baselines/local/$SUITE/$stem.errors.txt" "tests/corpus/baselines/local/$SUITE/$stem.ts.errors.txt" "tests/corpus/baselines/local/$SUITE/$stem.tsx.errors.txt"
# 内存限制强制：与全量口径一致（RLIMIT_AS 4GiB），二进制内守卫会拒绝无限制运行
out=$(TSOX_SUBMODULE_SUITE=$SUITE TSOX_SUBMODULE_LIMIT=0 TSOX_SUBMODULE_FILTER="$name" timeout 60 nice -n 10 bash -c 'ulimit -v 4194304; exec "$0" --exact submodule_compiler::submodule_compiler_cases --nocapture' "$BIN" 2>&1 | grep -E '^\[w0\]|panicked' | head -5)
echo "$out"
for f in "tests/corpus/baselines/local/$SUITE/$stem.errors.txt" "tests/corpus/baselines/local/$SUITE/$stem.ts.errors.txt" "tests/corpus/baselines/local/$SUITE/$stem.tsx.errors.txt"; do
  ref="tests/corpus/testdata/baselines/reference/$SUITE/$(basename "$f")"
  if [[ ! -f "$f" && -f "$ref" ]]; then
    echo "--- local 零输出（无基线文件产生），期望的参考基线存在：$(basename "$ref") ---"
  fi
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
