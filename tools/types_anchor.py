#!/usr/bin/env python3
"""从 .traces/types/ 本地 .types 与 reference .types 提取每例首分歧锚点。

输出（默认仓库根）：
  corpus_types_anchor.csv   case,line_no,context
  line_no 为首个分歧行号（本地行），context 为两侧该行的摘要。
已知渲染偏差模式跳过（typeof 侧/裸 any 值行），避免噪声锚点。
重采：全量 TSOX_TYPES_EMIT_DIR=.traces/types 跑 corpus 后执行本脚本。
"""
import os
import re
import sys

ROOT = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
LOCAL_DIR = os.path.join(ROOT, ".traces", "types")
REF_DIR = "/home/cqh/workspace/typescript-go/tsc/testdata/baselines/reference/compiler"
OUT = os.path.join(ROOT, "corpus_types_anchor.csv")

TYPEOF_RE = re.compile(r"^>(\S+) : typeof \1$")
ANY_RE = re.compile(r"^>\S+ : any$")


def norm(path):
    with open(path, encoding="utf-8", errors="replace") as f:
        return [l.rstrip("\r\n") for l in f]


def is_known_noise(local_line, ref_line):
    if TYPEOF_RE.match(local_line) and ref_line.startswith(">"):
        return True
    if ANY_RE.match(local_line):
        return True
    return False


def main():
    out = OUT if len(sys.argv) < 2 else sys.argv[1]
    rows = 0
    with open(out, "w", encoding="utf-8") as fo:
        fo.write("case,line_no,context\n")
        for fn in sorted(os.listdir(LOCAL_DIR)):
            if not fn.endswith(".types"):
                continue
            stem = fn[: -len(".types")]
            for ext in (".ts", ".tsx"):
                if stem.endswith(ext):
                    stem = stem[: -len(ext)]
                    break
            ref = os.path.join(REF_DIR, fn)
            if not os.path.isfile(ref):
                continue
            local = norm(os.path.join(LOCAL_DIR, fn))
            expect = norm(ref)
            i = j = 0
            found = None
            while i < len(local) or j < len(expect):
                a = local[i] if i < len(local) else "<EOF>"
                b = expect[j] if j < len(expect) else "<EOF>"
                if a == b:
                    i += 1
                    j += 1
                    continue
                if is_known_noise(a, b):
                    i += 1
                    j += 1
                    continue
                if a.startswith(">") and b.startswith(">"):
                    found = (i + 1, a, b)
                    break
                if i + 1 < len(local) and local[i + 1] == b:
                    i += 1
                    continue
                if j + 1 < len(expect) and expect[j + 1] == a:
                    j += 1
                    continue
                found = (i + 1, a, b)
                break
            if found:
                ctx = f"L:{found[1]} || R:{found[2]}"
                fo.write(f'{stem},{found[0]},"{ctx.replace(chr(34), chr(39))}"\n')
                rows += 1
    print(f"锚点提取：{rows} 例 -> {out}")


if __name__ == "__main__":
    main()
