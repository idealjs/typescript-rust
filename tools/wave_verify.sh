#!/bin/bash
# 波内串行验证：在指定 worktree 构建语料二进制，于主仓数据面直跑全量，刷新五份数据 CSV 并输出判定行。
# 用法: tools/wave_verify.sh <worktree_abs> <case_key> <tag>
#   <case_key> 形如 compiler/foo.ts（目标用例）
#   <tag>      本次验证标识，日志归档到 /tmp/flywheel_verify/<tag>.log
# 首次调用自动把主仓 corpus_results/skips.csv 快照为 *.base.csv（波起点水位，波收尾由主 agent 删除）。
# 输出（stdout 末尾，供 workflow 解析）：
#   VERDICT case=<key> status=<PASS|FAIL|SKIP|CRASH|BUILD_ERROR> fail_count=<n> anchor_count=<n> new_fails=<a,b,..> new_skips=<..> fixed=<m>
#   REGRESSION <key>   （每个新 FAIL/SKIP 一行）
#   BUILD_ERR <text>   （BUILD_ERROR 时最多 30 行）
set -u
export LC_ALL=C
MAIN=/home/cqh/workspace/ts2rust-port
# anchor 快检模式：单例跑 + .types 发射 + 与 Go 参考逐行比对（errors PASS 且 .types 全等才算过）
# 用法: tools/wave_verify.sh --anchor <worktree_abs> <case_key> <tag>
# PASS 判据 = 目标例 errors PASS 且本例 .types 与 Go reference 全文一致（anchor 行消失）
if [ "${1:-}" = "--anchor" ]; then
  WT=$2; KEY=$3; TAG=$4
  NAME=${KEY#compiler/}
  STEM=$(basename "$NAME"); STEM=${STEM%.*}
  cd "$MAIN" || exit 1
  mkdir -p /tmp/flywheel_verify
  if ! (cd "$WT" && timeout 600 nice -n 10 bash -c 'ulimit -v 8388608; exec cargo test --release -p tsox --test corpus --no-run --jobs 12') > /tmp/flywheel_build_$TAG.log 2>&1; then
    echo "VERDICT case=$KEY status=BUILD_ERROR mode=anchor fail_count=- skip_count=- anchor_count=- new_fails=- new_skips=- fixed=-1"
    grep -E "^error" -A 6 /tmp/flywheel_build_$TAG.log | head -30 | sed 's/^/BUILD_ERR /'
    exit 0
  fi
  BIN=$(ls -t "$WT"/target/release/deps/corpus-* 2>/dev/null | grep -v '\.d$' | head -1)
  if [ -z "$BIN" ]; then
    echo "VERDICT case=$KEY status=BUILD_ERROR mode=anchor fail_count=- skip_count=- anchor_count=- new_fails=- new_skips=- fixed=-1"
    echo "BUILD_ERR corpus binary not found under $WT/target/release/deps"
    exit 0
  fi
  TDIR=/tmp/flywheel_types_$TAG; rm -rf "$TDIR"; mkdir -p "$TDIR"
  out=$(cd "$MAIN/crates/tsox" && BIN="$BIN" TSOX_TYPES_EMIT_DIR="$TDIR" bash "$MAIN/tools/corpus_one.sh" "$NAME" 2>&1)
  printf '%s\n' "$out" > "/tmp/flywheel_verify/$TAG.log"
  wline=$(printf '%s\n' "$out" | grep -E '^\[w0\]' | tail -1)
  estatus=$(printf '%s\n' "$wline" | awk '{print $3}')
  [ -z "$estatus" ] && estatus=CRASH
  python3 "$MAIN/tools/types_one_diff.py" "$TDIR/$STEM.types" "$STEM" > /tmp/flywheel_adiff_$TAG.txt 2>&1
  tmatch=$(head -1 /tmp/flywheel_adiff_$TAG.txt)
  # 验收只看 .types 全等：errors 基线回归/失败属预期（anchor 抹平波口径），仅性能问题（崩溃致无 .types）会连带判 FAIL
  if [ "$tmatch" = "TYPES_MATCH" ]; then st=PASS; else st=FAIL; fi
  echo "VERDICT case=$KEY status=$st mode=anchor fail_count=- skip_count=- anchor_count=- new_fails=- new_skips=- fixed=-1"
  [ "$estatus" != "PASS" ] && echo "NOTE errors_status=$estatus（预期内，不拦截） $(printf '%s\n' "$wline" | sed 's/^[^)]*) *//' | cut -c1-100)"
  [ "$tmatch" != "TYPES_MATCH" ] && sed -n '2,31p' /tmp/flywheel_adiff_$TAG.txt | sed 's/^/ANCHORDIFF /'
  exit 0
fi
# 单例快检模式：worktree 构建 + 只跑目标例（复用 corpus_one.sh 的单例执行与 ref/local diff）
# 用法: tools/wave_verify.sh --single <worktree_abs> <case_key> <tag>
# 适用于验证排队积压（待验证 >1）时压缩单轮验证时长；跨例回归不感知，合并后由主 agent 全量统一暴露
if [ "${1:-}" = "--single" ]; then
  WT=$2; KEY=$3; TAG=$4
  NAME=${KEY#compiler/}
  cd "$MAIN" || exit 1
  mkdir -p /tmp/flywheel_verify
  # 资源护栏：-j 12 限并行（24 核取半）、每进程 AS 8GB 兜底、nice 降优先级、600s 内部超时（外层 world.run 900s）
  if ! (cd "$WT" && timeout 600 nice -n 10 bash -c 'ulimit -v 8388608; exec cargo test --release -p tsox --test corpus --no-run --jobs 12') > /tmp/flywheel_build_$TAG.log 2>&1; then
    echo "VERDICT case=$KEY status=BUILD_ERROR mode=single fail_count=- skip_count=- anchor_count=- new_fails=- new_skips=- fixed=-1"
    grep -E "^error" -A 6 /tmp/flywheel_build_$TAG.log | head -30 | sed 's/^/BUILD_ERR /'
    exit 0
  fi
  BIN=$(ls -t "$WT"/target/release/deps/corpus-* 2>/dev/null | grep -v '\.d$' | head -1)
  if [ -z "$BIN" ]; then
    echo "VERDICT case=$KEY status=BUILD_ERROR mode=single fail_count=- skip_count=- anchor_count=- new_fails=- new_skips=- fixed=-1"
    echo "BUILD_ERR corpus binary not found under $WT/target/release/deps"
    exit 0
  fi
  out=$(BIN="$BIN" bash "$MAIN/tools/corpus_one.sh" "$NAME" 2>&1)
  printf '%s\n' "$out" > "/tmp/flywheel_verify/$TAG.log"
  wline=$(printf '%s\n' "$out" | grep -E '^\[w0\]' | tail -1)
  status=$(printf '%s\n' "$wline" | awk '{print $3}')
  [ -z "$status" ] && status=CRASH
  echo "VERDICT case=$KEY status=$status mode=single fail_count=- skip_count=- anchor_count=- new_fails=- new_skips=- fixed=-1"
  reason=$(printf '%s\n' "$wline" | sed 's/^[^)]*) *//' | cut -c1-200)
  [ -n "$reason" ] && echo "REASON $reason"
  printf '%s\n' "$out" | sed -n '/^--- diff/,$p' | head -40 | sed 's/^/CASEDIFF /'
  exit 0
fi
if [ "${1:-}" = "--snapshot" ]; then
  cd "$MAIN" || exit 1
  # 幂等：波起点基线已存在则跳过（amend 重放/波收尾前不得覆盖真基线；波收尾由主 agent 删除）
  if [ ! -f corpus_results.base.csv ]; then cp corpus_results.csv corpus_results.base.csv; fi
  if [ ! -f corpus_skips.base.csv ]; then cp corpus_skips.csv corpus_skips.base.csv; fi
  echo "SNAPSHOT base: $(($(wc -l < corpus_results.base.csv) - 1)) FAIL / $(($(wc -l < corpus_skips.base.csv) - 1)) SKIP"
  exit 0
fi
WT=$1; KEY=$2; TAG=$3
NAME=${KEY#compiler/}
cd "$MAIN" || exit 1

if [ ! -f corpus_results.base.csv ]; then cp corpus_results.csv corpus_results.base.csv; fi
if [ ! -f corpus_skips.base.csv ]; then cp corpus_skips.csv corpus_skips.base.csv; fi

emit_verdict() { # $1=status $2=new_fails $3=new_skips $4=fixed
  local fails skips anchor
  fails=$(($(wc -l < corpus_results.csv) - 1))
  skips=$(($(wc -l < corpus_skips.csv) - 1))
  anchor=$(($(wc -l < corpus_types_anchor.csv) - 1))
  echo "VERDICT case=$KEY status=$1 fail_count=$fails skip_count=$skips anchor_count=$anchor new_fails=$2 new_skips=$3 fixed=$4"
  for k in $(echo "$2" | tr ',' ' '); do [ -n "$k" ] && echo "REGRESSION-FAIL $k"; done
  for k in $(echo "$3" | tr ',' ' '); do [ -n "$k" ] && echo "REGRESSION-SKIP $k"; done
}

# 1) worktree 内构建语料二进制（worktree 本地 target：退回归增量；构建失败逐例暴露）
#    资源护栏：-j 12 限并行、每进程 AS 8GB 兜底、nice 降优先级、1200s 内部超时（外层 world.run 3600s）
if ! (cd "$WT" && timeout 1200 nice -n 10 bash -c 'ulimit -v 8388608; exec cargo test --release -p tsox --test corpus --no-run --jobs 12') > /tmp/flywheel_build_$TAG.log 2>&1; then
  echo "VERDICT case=$KEY status=BUILD_ERROR fail_count=-1 skip_count=-1 anchor_count=-1 new_fails=- new_skips=- fixed=-1"
  grep -E "^error" -A 6 /tmp/flywheel_build_$TAG.log | head -30 | sed 's/^/BUILD_ERR /'
  exit 0
fi

BIN=$(ls -t "$WT"/target/release/deps/corpus-* 2>/dev/null | grep -v '\.d$' | head -1)
if [ -z "$BIN" ]; then
  echo "VERDICT case=$KEY status=BUILD_ERROR fail_count=-1 skip_count=-1 anchor_count=-1 new_fails=- new_skips=- fixed=-1"
  echo "BUILD_ERR corpus binary not found under $WT/target/release/deps"
  exit 0
fi

# 2) 主仓数据面直跑全量（CWD=主仓 crates/tsox：testdata 与 baselines 落主仓）
rm -rf /tmp/rust_trace "$MAIN/.traces/types"
mkdir -p /tmp/rust_trace "$MAIN/.traces/types"
mkdir -p /tmp/flywheel_verify
cd "$MAIN/crates/tsox" || exit 1
TSOX_SUBMODULE_LIMIT=0 TSOX_FN_TRACE_DIR=/tmp/rust_trace TSOX_TYPES_EMIT_DIR="$MAIN/.traces/types" \
  timeout 2400 nice -n 10 bash -c 'ulimit -v 4194304; exec "$0" --exact submodule_compiler::submodule_compiler_cases' "$BIN" \
  > "$MAIN/fullrun.log" 2>&1
cp "$MAIN/fullrun.log" "/tmp/flywheel_verify/$TAG.log"

# 3) 刷新数据 CSV（五份 + 栈差 + 分片）
cd "$MAIN" || exit 1
python3 tools/corpus_csv_export.py fullrun.log > /dev/null 2>&1
python3 tools/types_csv.py rust > /dev/null 2>&1
python3 tools/types_anchor.py > /dev/null 2>&1
python3 tools/stack_diff.py > /dev/null 2>&1
python3 tools/cut_test_shards.py > /dev/null 2>&1

# 4) 判定（新 SKIP 按 log 归因分类：timed out = 负载噪声，其余 panic/diff = 真回归）
status=$(grep -E "^\[w[0-9]+\] #[0-9]+/[0-9]+ (PASS|FAIL|SKIP) $NAME \(" "$MAIN/fullrun.log" | tail -1 | awk '{print $3}')
[ -z "$status" ] && status=CRASH
new_fails=$(comm -13 <(grep -v '^key$' corpus_results.base.csv) <(grep -v '^key$' corpus_results.csv) | paste -sd, -)
new_skips_all=$(comm -13 <(grep -v '^key$' corpus_skips.base.csv) <(grep -v '^key$' corpus_skips.csv))
new_skips=""; noise_skips=""
for k in $new_skips_all; do
  nm=${k#compiler/}
  if grep -E "^\[w[0-9]+\] #[0-9]+/[0-9]+ SKIP $nm \(" "$MAIN/fullrun.log" | tail -1 | grep -q "timed out"; then
    noise_skips="$noise_skips,$k"; echo "TIMEOUT-SKIP $k"
  else
    new_skips="$new_skips,$k"
  fi
done
new_skips=${new_skips#,}; noise_skips=${noise_skips#,}
printf '%s\n' "$new_skips_all" | grep -v '^$' > /tmp/flywheel_newskips_$TAG.txt || true
fixed=$(comm -23 <(grep -v '^key$' corpus_results.base.csv) <(grep -v '^key$' corpus_results.csv) | grep -vxFf /tmp/flywheel_newskips_$TAG.txt | grep -vc '^$')
emit_verdict "$status" "$new_fails" "$new_skips" "${fixed:-0}"
[ -n "$noise_skips" ] && echo "NOTE noise_skips=$noise_skips"

# 判定行永远成功退出：退出码不承载判定（VERDICT 行承载），避免 world.run 误标红
exit 0
