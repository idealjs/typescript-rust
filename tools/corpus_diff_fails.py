#!/usr/bin/env python3
"""对比两次全量语料 runlog 的失败集合，报告修复/回归/仍失败。

用法: python3 tools/corpus_diff_fails.py <old_runlog> <new_runlog>
"""
import re
import sys

FAIL_RE = re.compile(r"FAIL ([A-Za-z0-9_()+=,.-]+\.tsx?) \(")


def fails_of(path):
    out = set()
    with open(path, encoding="utf-8", errors="replace") as f:
        for line in f:
            m = FAIL_RE.search(line)
            if m:
                out.add(m.group(1))
    return out


def main():
    old, new = fails_of(sys.argv[1]), fails_of(sys.argv[2])
    fixed = old - new
    regressed = new - old
    print(f"old={len(old)} new={len(new)} fixed={len(fixed)} regressed={len(regressed)}")
    if regressed:
        print("--- regressions ---")
        for name in sorted(regressed):
            print(name)
    if fixed:
        print(f"--- fixed ({len(fixed)}) ---")
        for name in sorted(fixed):
            print(name)


if __name__ == "__main__":
    main()
