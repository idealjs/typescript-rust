#!/bin/bash
# 语料修复批工作流：
#   corpus_batch.sh fail-list <runlog> <out.txt> [suite]  # 从跑批日志提取失败清单
#   corpus_batch.sh run <batch.txt> [suite]               # 只跑清单内用例
# 环境约定：ulimit 内存限制 +（批量验证时）--release
set -e
SUITE=${3:-compiler}
BIN=$(ls -t target/debug/deps/corpus-* | grep -v '\.d$' | head -1)

case "$1" in
  fail-list)
    grep -oE "FAIL [A-Za-z0-9_()+=,.-]+\.tsx? \(" "$2" | sed -E 's/FAIL (.+) \(/\1/' | sort -u > "$3"
    echo "fail list -> $3 ($(wc -l < "$3") cases)"
    ;;
  run)
    ( ulimit -v 8388608; TSOX_SUBMODULE_SUITE=$SUITE TSOX_SUBMODULE_LIMIT=0 \
      TSOX_SUBMODULE_CASES_FILE="$2" cargo test -p tsox --test corpus \
      submodule_compiler_cases 2>&1 )
    ;;
  *) echo "usage: $0 fail-list <runlog> <out.txt> [suite] | run <batch.txt> [suite]"; exit 1;;
esac
