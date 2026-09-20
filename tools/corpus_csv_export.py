#!/usr/bin/env python3
"""全量日志 → corpus_results.csv(仅 FAIL)+ corpus_skips.csv(超出 Go 基准的 SKIP)。

用法: python3 tools/corpus_csv_export.py [日志路径]
日志默认依次尝试 fullrun.log 与 corpus run log。

产出(仓库根):
  corpus_results.csv        本轮失败用例,表头 key,seconds,按 key 字典序
  corpus_skips.csv          SKIP 差异表:超出 Go 基准的 SKIP 用例
  *.prev.csv / *.diff       上一轮副本与两轮机械差异

Go 一致的合法 SKIP 集合记录在 tools/skip_baseline.txt(每行一个 key,
`compiler/<用例名>`,# 开头为注释);文件缺失或为空时所有 SKIP 均视为差异。
SKIP 差异与 FAIL 同流程修复:纳入 subagent 修复循环,直至差集为空。
"""
import os
import re
import shutil
import sys

ROOT = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
LINE_RE = re.compile(
    r"^\[w\d+\] #\d+/\d+ (FAIL|SKIP) (\S+) \(([\d.]+)s\)"
)
BASELINE = os.path.join(ROOT, "tools", "skip_baseline.txt")


def baseline_skips():
    if not os.path.isfile(BASELINE):
        return set()
    with open(BASELINE, encoding="utf-8") as f:
        return {
            l.strip() for l in f
            if l.strip() and not l.startswith("#")
        }


def scan(log_path):
    fails, skips = {}, {}
    with open(log_path, encoding="utf-8", errors="replace") as f:
        for line in f:
            m = LINE_RE.match(line)
            if not m:
                continue
            status, name, secs = m.groups()
            key = f"compiler/{name}"
            (fails if status == "FAIL" else skips)[key] = secs
    return fails, skips


def emit(rows, path, header="key,seconds\n"):
    with open(path, "w", encoding="utf-8") as f:
        f.write(header)
        for key in sorted(rows):
            f.write(f"{key},{rows[key]}\n")


def rotate_and_diff(path):
    prev, diff = path.replace(".csv", ".prev.csv"), path.replace(".csv", ".diff")
    if os.path.isfile(path):
        shutil.copyfile(path, prev)
    if os.path.isfile(prev):
        status = os.system(f"diff -U0 {prev} {path} > {diff} 2>/dev/null")
        added = sum(1 for l in open(diff) if l.startswith("+") and not l.startswith("+++"))
        removed = sum(1 for l in open(diff) if l.startswith("-") and not l.startswith("---"))
        print(f"  diff: +{added} / -{removed} -> {os.path.basename(diff)}")


def main():
    log = sys.argv[1] if len(sys.argv) > 1 else ""
    if not log:
        for cand in ("fullrun.log",
                     "crates/tsox/tests/corpus/baselines/local/submodule_run.log"):
            if os.path.isfile(cand):
                log = cand
                break
    if not log or not os.path.isfile(log):
        sys.exit(f"log not found: {log or '(no candidate)'}")

    fails, skips = scan(log)
    legit = baseline_skips()
    skip_diff = {k: v for k, v in skips.items() if k not in legit}

    results = os.path.join(ROOT, "corpus_results.csv")
    emit(fails, results)
    print(f"corpus_results.csv ({len(fails)} failed cases)")

    skips_csv = os.path.join(ROOT, "corpus_skips.csv")
    emit(skip_diff, skips_csv)
    print(f"corpus_skips.csv ({len(skip_diff)} skip defects; "
          f"{len(skips)} skipped, {len(skips) - len(skip_diff)} in Go baseline)")

    for path in (results, skips_csv):
        rotate_and_diff(path)


if __name__ == "__main__":
    main()
