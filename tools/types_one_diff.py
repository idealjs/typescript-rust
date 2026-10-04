#!/usr/bin/env python3
"""单例 .types 与 Go 参考逐行比对（支持多 target 变体名）。

用法: python3 tools/types_one_diff.py <rust_types_dir> <stem>
  先取 <dir>/<stem>.types；不存在则回落 glob <dir>/<stem>(target=*).types
  （runner 对多 target 用例写变体名，submodule_compiler.rs:451-455）。
  每个产出文件按其文件名 key 对应 corpus_go_types.csv 行比对，全部一致才 MATCH。
输出（stdout）：
  首行 TYPES_MATCH 或 TYPES_MISMATCH
  MISMATCH 时随后输出各变体的首个分歧区（两侧上下文 + 行号）。
"""
import csv
import glob
import os
import sys

csv.field_size_limit(sys.maxsize)

ROOT = "/home/cqh/workspace/ts2rust-port"


def go_content(key):
    with open(f"{ROOT}/corpus_go_types.csv", encoding="utf-8", errors="replace") as f:
        for row in csv.reader(f):
            if row and row[0] == key:
                c = row[2]
                return c.replace("\\n", "\n").replace("\\\\", "\\")
    return None


def first_diff(rust, ref):
    rl, gl = rust.split("\n"), ref.split("\n")
    out = []
    for i in range(max(len(rl), len(gl))):
        a = rl[i] if i < len(rl) else "<EOF>"
        b = gl[i] if i < len(gl) else "<EOF>"
        if a != b:
            lo = max(0, i - 2)
            for j in range(lo, min(i + 4, max(len(rl), len(gl)))):
                ra = rl[j] if j < len(rl) else "<EOF>"
                rb = gl[j] if j < len(gl) else "<EOF>"
                if ra != rb:
                    out.append(f"  行{j+1} L: {ra[:160]}")
                    out.append(f"  行{j+1} R: {rb[:160]}")
            return out
    return None


def main():
    tdir, stem = sys.argv[1], sys.argv[2]
    files = []
    exact = os.path.join(tdir, f"{stem}.types")
    if os.path.isfile(exact):
        files.append(exact)
    else:
        files = sorted(glob.glob(os.path.join(tdir, f"{stem}(target=*).types")))
    if not files:
        print("TYPES_MISMATCH")
        print(f"  rust .types 未产出：期望 {stem}.types 或 {stem}(target=*).types，目录 {tdir} 无匹配")
        return
    ok = True
    lines = []
    for fp in files:
        key = os.path.basename(fp)[: -len(".types")]
        rust = open(fp, encoding="utf-8", errors="replace").read().replace("\r\n", "\n")
        ref = go_content(key)
        if ref is None:
            ok = False
            lines.append(f"  [{key}] Go 参考缺失（corpus_go_types.csv 无此 key）")
            continue
        d = first_diff(rust, ref)
        if d is not None:
            ok = False
            lines.append(f"  [{key}]")
            lines.extend(d)
    print("TYPES_MATCH" if ok else "TYPES_MISMATCH")
    for l in lines[:40]:
        print(l)


if __name__ == "__main__":
    main()
