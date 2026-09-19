#!/usr/bin/env python3
"""全量失败用例按错误码家族归并，产出每类 worktree 的用例清单。

用法: python3 tools/corpus_families.py <runlog> <outdir>
每行: <case> :: missing=[..] extra=[..]
"""
import glob
import os
import re
import sys
from collections import defaultdict

ROOT = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
REF = os.path.join(ROOT, "crates/tsox/tests/corpus/testdata/baselines/reference/compiler")
LOC = os.path.join(ROOT, "crates/tsox/tests/corpus/baselines/local/compiler")

FAIL_RE = re.compile(r"FAIL ([A-Za-z0-9_()+=,.-]+\.tsx?) \(")
CODE_RE = re.compile(r"error TS(\d+):")

FAMILIES = {
    "assign": {2322, 2345, 2353, 2741, 2769, 2416, 2719, 2320, 2321, 2352, 2351,
               2348, 2604, 2354, 2347, 2365, 2367, 2820, 2540, 2454, 2834, 2835,
               18048, 18050, 18046, 18047, 2445, 2446, 2447, 2429, 2430},
    "members": {2339, 2304, 2551, 2552, 2553, 2661, 2663, 2688, 2561, 2564, 2528,
                2341, 2343, 2362, 2559, 2411, 2425, 2560, 2716, 2686, 2687, 2537},
    "typeval": {2693, 2307, 2449, 2749, 2305, 2306, 2303, 2302, 2503, 2694, 2362,
                2338, 2678, 2692, 2672, 2839, 2840},
    "binder": {2300, 2301, 2393, 2394, 2395, 2396, 2397, 2403, 2448, 2451, 2452,
               2453, 2717, 2718, 2261, 2262, 2263, 2264, 2414, 2424},
    "jsany": set(range(7005, 7062)) | {7066, 7067, 7070, 7071, 7074, 7075, 7076, 7077, 7078, 7079},
    "parser": set(range(1000, 1500)) | {5069, 5072, 5074, 5076},
}


def codes_of_dir(base, stem):
    out = set()
    for p in [os.path.join(base, stem + ".errors.txt")] + \
             glob.glob(os.path.join(base, glob.escape(stem) + "(*).errors.txt")):
        try:
            with open(p, encoding="utf-8", errors="replace") as f:
                out |= set(CODE_RE.findall(f.read()))
        except FileNotFoundError:
            continue
    return out


def main():
    runlog, outdir = sys.argv[1], sys.argv[2]
    os.makedirs(outdir, exist_ok=True)
    fails = []
    with open(runlog, encoding="utf-8", errors="replace") as f:
        for line in f:
            m = FAIL_RE.search(line)
            if m and m.group(1) not in fails:
                fails.append(m.group(1))

    fam_cases = defaultdict(list)
    for name in fails:
        stem = name[:-3] if name.endswith(".ts") else name[:-4]
        ref = codes_of_dir(REF, stem)
        loc = codes_of_dir(LOC, stem)
        missing = {int(c) for c in ref - loc}
        extra = {int(c) for c in loc - ref}
        counts = defaultdict(int)
        for c in list(missing) + list(extra):
            for fam, codes in FAMILIES.items():
                if c in codes:
                    counts[fam] += 1
        if not ref and not loc:
            fam = "types"
        elif counts:
            fam = max(counts, key=lambda k: (counts[k], k))
        elif missing or extra:
            fam = "misc"
        else:
            fam = "diag"
        fam_cases[fam].append(
            f"{name} :: missing={sorted(missing)} extra={sorted(extra)}")

    for fam, lines in fam_cases.items():
        with open(os.path.join(outdir, fam + ".txt"), "w") as f:
            f.write("\n".join(sorted(lines)) + "\n")
        print(f"{fam}: {len(lines)}")


if __name__ == "__main__":
    main()
