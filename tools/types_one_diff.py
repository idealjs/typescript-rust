#!/usr/bin/env python3
"""单例 .types 与 Go 参考逐行比对。

用法: python3 tools/types_one_diff.py <rust_types_file> <stem>
输出（stdout）：
  首行 TYPES_MATCH 或 TYPES_MISMATCH
  MISMATCH 时随后输出首个分歧区（两侧上下文 + 行号），供退回 prompt 内嵌。
"""
import csv
import sys

ROOT = "/home/cqh/workspace/ts2rust-port"


def go_content(stem):
    with open(f"{ROOT}/corpus_go_types.csv", encoding="utf-8", errors="replace") as f:
        for row in csv.reader(f):
            if row and row[0] == stem:
                c = row[2]
                return c.replace("\\n", "\n").replace("\\\\", "\\")
    return None


def main():
    rust_path, stem = sys.argv[1], sys.argv[2]
    try:
        rust = open(rust_path, encoding="utf-8", errors="replace").read()
    except OSError:
        print("TYPES_MISMATCH")
        print(f"  rust .types 未产出：{rust_path}")
        return
    ref = go_content(stem)
    if ref is None:
        print("TYPES_MISMATCH")
        print("  Go 参考缺失（corpus_go_types.csv 无此例）")
        return
    rl, gl = rust.split("\n"), ref.split("\n")
    for i in range(max(len(rl), len(gl))):
        a = rl[i] if i < len(rl) else "<EOF>"
        b = gl[i] if i < len(gl) else "<EOF>"
        if a != b:
            print("TYPES_MISMATCH")
            lo = max(0, i - 2)
            for j in range(lo, min(i + 4, max(len(rl), len(gl)))):
                ra = rl[j] if j < len(rl) else "<EOF>"
                rb = gl[j] if j < len(gl) else "<EOF>"
                if ra != rb:
                    print(f"  行{j+1} L: {ra[:160]}")
                    print(f"  行{j+1} R: {rb[:160]}")
            return
    print("TYPES_MATCH")


if __name__ == "__main__":
    main()
