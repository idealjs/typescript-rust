#!/bin/bash
# 从 Go 版 TypeScript 仓库整目录同步官方语料（用例 + reference 基线）。
# 用法: tools/corpus_sync.sh [GO_REPO_DIR]   # 默认 /home/cqh/workspace/TypeScript
set -e
GO_REPO=${1:-/home/cqh/workspace/TypeScript}
SRC="$GO_REPO/tsc/testdata"
DST="$(dirname "$0")/../crates/tsox/tests/corpus/testdata"

if [ ! -d "$SRC/tests/cases" ]; then
  echo "source not found: $SRC/tests/cases" >&2
  exit 1
fi

mkdir -p "$DST"
# 整目录覆盖复制（--delete 保持与源一致）
rsync -a --delete "$SRC/tests/cases/" "$DST/tests/cases/"
rsync -a --delete "$SRC/baselines/reference/" "$DST/baselines/reference/"

echo "cases:      $(find "$DST/tests/cases" -type f | wc -l) files"
echo "references: $(find "$DST/baselines/reference" -type f | wc -l) files"
diff -rq "$SRC/tests/cases" "$DST/tests/cases" && echo "cases verify: OK"
diff -rq "$SRC/baselines/reference" "$DST/baselines/reference" && echo "reference verify: OK"
