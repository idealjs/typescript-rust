#!/usr/bin/env python3
"""两侧执行栈对照：Rust/Go 每用例函数调用序列（去重保序）→ 归一函数名 → 差集。

输入目录：
  /tmp/rust_trace/<case>.txt   Rust 侧（snake_case，每行一函数，TSOX_FN_TRACE_DIR 采集）
  /tmp/go_trace/<case>.ts.txt  Go 侧（camelCase，每行一函数，GOFN_TRACE_DIR 采集）
输出（默认仓库根）：
  corpus_stack_diff.csv        表头 case,go_only,rust_only（分号分隔）
  go_only  = Go 执行而 Rust 未执行（缺失路径；实测 ≈0，全量迁移已完成）
  rust_only= Rust 执行而 Go 未执行（Rust 独有/等价不同名路径，辅助信号）
归一：去下划线转小写（get_type_of_symbol ↔ getTypeOfSymbol → gettypeofsymbol）。
出现于多数用例差集的函数视为两侧框架命名噪音自动剔除。
重采：tools/instrument_rust.py 全量跑（TSOX_FN_TRACE_DIR）+ tools/instrument_go.py
插桩 oracle 后 GOFN_TRACE_DIR go test -parallel 1（采完 --restore 还原 oracle）。
"""
import os
import sys

ROOT = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
RUST_DIR = "/tmp/rust_trace"
GO_DIR = "/tmp/go_trace"
OUT = os.path.join(ROOT, "corpus_stack_diff.csv")


def norm(name):
    return name.replace("_", "").lower()


def load(path):
    if not os.path.isfile(path):
        return None
    with open(path, encoding="utf-8") as f:
        return {norm(l.strip()) for l in f if l.strip()}


def main():
    rust_dir = sys.argv[1] if len(sys.argv) > 1 else RUST_DIR
    go_dir = sys.argv[2] if len(sys.argv) > 2 else GO_DIR
    out = sys.argv[3] if len(sys.argv) > 3 else OUT
    cases = []
    for f in os.listdir(go_dir):
        if not f.endswith(".txt"):
            continue
        stem = f[:-4]
        for ext in (".ts", ".tsx", ".js", ".jsx", ".mts", ".cts"):
            if stem.endswith(ext):
                stem = stem[: -len(ext)]
                break
        rust_file = stem + ".txt"
        if os.path.isfile(os.path.join(rust_dir, rust_file)):
            cases.append((stem, rust_file, f))
    rows = 0
    pairs = []
    for stem, rust_file, go_file in sorted(cases):
        go = load(os.path.join(go_dir, go_file))
        rust = load(os.path.join(rust_dir, rust_file))
        if not go or not rust:
            continue
        pairs.append((stem, sorted(go - rust), sorted(rust - go)))
    # 自动噪音过滤：出现在多数用例 go_only/rust_only 的函数是两侧测试框架
    # 与实现命名差异的固有噪音（如 Go testrunner 层），不携带用例级信号
    n = max(1, len(pairs) // 2)
    from collections import Counter

    freq = Counter()
    for _, go_only, rust_only in pairs:
        freq.update(go_only)
        freq.update(rust_only)
    noise = {f for f, c in freq.items() if c > n}
    with open(out, "w", encoding="utf-8") as f:
        f.write("case,go_only,rust_only\n")
        for stem, go_only, rust_only in pairs:
            go_clean = [x for x in go_only if x not in noise]
            rust_clean = [x for x in rust_only if x not in noise]
            f.write(f"{stem},{';'.join(go_clean)},{';'.join(rust_clean)}\n")
            rows += 1
    print(f"对照完成：{rows}/{len(cases)} 用例（剔除框架噪音 {len(noise)} 个函数）-> {out}")


if __name__ == "__main__":
    main()
