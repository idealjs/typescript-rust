#!/usr/bin/env python3
"""汇总 .types 原始文件为仓库根全量 CSV（corpus_rust_types / corpus_go_types）。

用法：
  Rust 侧（频繁更新）：python3 tools/types_csv.py rust  <rust_types_dir>  corpus_rust_types.csv
  Go   侧（一次性）  ：python3 tools/types_csv.py go <go_reference_dir>  corpus_go_types.csv

CSV 形态：case,line_count,content（content 为 .types 全文，换行替换为 \\n）。
"""
import os
import sys

ROOT = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))


def summarize(src_dir, out_path):
    n = 0
    with open(out_path, "w", encoding="utf-8") as fo:
        fo.write("case,line_count,content\n")
        for fn in sorted(os.listdir(src_dir)):
            if not fn.endswith(".types"):
                continue
            stem = fn[: -len(".types")]
            with open(os.path.join(src_dir, fn), encoding="utf-8", errors="replace") as fi:
                content = fi.read()
            line_count = content.count("\n")
            escaped = content.replace("\\", "\\\\").replace("\n", "\\n").replace('"', '""')
            fo.write(f'{stem},{line_count},"{escaped}"\n')
            n += 1
    print(f"{out_path}: {n} 例")


def main():
    side = sys.argv[1] if len(sys.argv) > 1 else ""
    if side == "rust":
        src = sys.argv[2] if len(sys.argv) > 2 else os.path.join(ROOT, ".traces", "types")
        out = sys.argv[3] if len(sys.argv) > 3 else os.path.join(ROOT, "corpus_rust_types.csv")
    elif side == "go":
        src = (
            sys.argv[2]
            if len(sys.argv) > 2
            else "/home/cqh/workspace/typescript-go/tsc/testdata/baselines/reference/compiler"
        )
        out = sys.argv[3] if len(sys.argv) > 3 else os.path.join(ROOT, "corpus_go_types.csv")
    else:
        sys.exit("usage: types_csv.py rust|go [src_dir] [out.csv]")
    if not os.path.isdir(src):
        sys.exit(f"src dir not found: {src}")
    summarize(src, out)


if __name__ == "__main__":
    main()
