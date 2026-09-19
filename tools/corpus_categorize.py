#!/usr/bin/env python3
"""按参考/本地基线错误码差异对失败用例聚类，产出每簇用例清单。

用法: python3 tools/corpus_categorize.py <runlog> <outdir>
"""
import os
import re
import subprocess
import sys
from collections import defaultdict

ROOT = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
REF = os.path.join(ROOT, "crates/tsox/tests/corpus/testdata/baselines/reference/compiler")
LOC = os.path.join(ROOT, "crates/tsox/tests/corpus/baselines/local/compiler")

FAIL_RE = re.compile(r"FAIL ([A-Za-z0-9_()+=,.-]+\.tsx?) \(")
CODE_RE = re.compile(r"error TS(\d+):")


def codes_of(path):
    try:
        with open(path, encoding="utf-8", errors="replace") as f:
            return set(CODE_RE.findall(f.read()))
    except FileNotFoundError:
        return None


def main():
    runlog, outdir = sys.argv[1], sys.argv[2]
    os.makedirs(outdir, exist_ok=True)
    fails = []
    with open(runlog, encoding="utf-8", errors="replace") as f:
        for line in f:
            m = FAIL_RE.search(line)
            if m and m.group(1) not in fails:
                fails.append(m.group(1))

    clusters = defaultdict(list)  # key -> [case, ...]
    nocat = []
    for name in fails:
        stem = name[:-3] if name.endswith(".ts") else name[:-4]
        ref_err = codes_of(os.path.join(REF, stem + ".errors.txt"))
        loc_err = codes_of(os.path.join(LOC, stem + ".errors.txt"))
        if ref_err is None and loc_err is None:
            nocat.append(name)
            continue
        missing = (ref_err or set()) - (loc_err or set())
        extra = (loc_err or set()) - (ref_err or set())
        if missing:
            key = "missing-TS" + ",".join(sorted(missing, key=int)[:4])
        elif extra:
            key = "extra-TS" + ",".join(sorted(extra, key=int)[:4])
        elif ref_err is None or loc_err is None:
            key = "baseline-file-missing"
        else:
            key = "text-diff-only"
        clusters[key].append(name)

    total = 0
    with open(os.path.join(outdir, "_summary.txt"), "w") as sumf:
        for key in sorted(clusters, key=lambda k: -len(clusters[k])):
            cases = clusters[key]
            total += len(cases)
            sumf.write(f"{len(cases):5d}  {key}\n")
            with open(os.path.join(outdir, re.sub(r"[^A-Za-z0-9_,-]", "_", key) + ".txt"), "w") as f:
                f.write("\n".join(cases) + "\n")
        sumf.write(f"{len(nocat):5d}  uncategorized(no baselines)\n")
    with open(os.path.join(outdir, "_all_fails.txt"), "w") as f:
        f.write("\n".join(fails) + "\n")
    print(f"{len(fails)} fails, {total} clustered, {len(nocat)} no-baseline -> {outdir}")


if __name__ == "__main__":
    main()
