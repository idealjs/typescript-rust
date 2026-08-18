#!/usr/bin/env python3
"""Secondary triage: diff triaged.txt entries against fresh local artifacts.

Reads tests/baselines/reference/triaged.txt, and for every entry whose local
artifact exists under tests/baselines/local/, computes a diff signature
(missing/extra TSxxxx code sets) and groups entries by signature.
Pure file processing — never runs the compiler.
"""
import os
import re
import sys
from collections import defaultdict

REF = "tests/baselines/reference"
LOC = "tests/baselines/local"
CODE = re.compile(r"error TS(\d+):")

def load(path):
    try:
        with open(path, encoding="utf-8", errors="replace") as f:
            return f.read()
    except FileNotFoundError:
        return None

def main():
    triaged = []
    with open(os.path.join(REF, "triaged.txt"), encoding="utf-8") as f:
        for line in f:
            line = line.strip()
            if line.startswith("compiler/"):
                triaged.append(line)

    groups = defaultdict(list)
    pending = 0
    identical = []
    for rel in triaged:
        ref = load(os.path.join(REF, rel))
        loc = load(os.path.join(LOC, rel))
        loc_del = os.path.exists(os.path.join(LOC, rel + ".delete"))
        if loc is None and not loc_del:
            pending += 1  # not yet run / skipped
            continue
        if loc_del and (loc is None):
            # official has errors, we emit none
            ref_codes = sorted(set(CODE.findall(ref or "")))
            groups[("UNDER", (), tuple(ref_codes), ())].append(rel)
            continue
        if loc == ref:
            identical.append(rel)  # now passing → remove from triaged.txt
            continue
        ref_codes = sorted(set(CODE.findall(ref or "")))
        loc_codes = sorted(set(CODE.findall(loc or "")))
        missing = tuple(c for c in ref_codes if c not in loc_codes)
        extra = tuple(c for c in loc_codes if c not in ref_codes)
        # first real diff line for flavor
        rl, ll = (ref or "").splitlines(), (loc or "").splitlines()
        flavor = ""
        for a, b in zip(rl, ll):
            if a != b:
                flavor = "REF: " + a[:90] + " || LOC: " + b[:90]
                break
        if not flavor and len(rl) != len(ll):
            src = rl[len(ll):] if len(rl) > len(ll) else ll[len(rl):]
            flavor = "LEN%+d %s" % (len(rl) - len(ll), src[0][:100] if src else "")
        groups[(missing and "MISS" or "", extra and "EXTRA" or "",
                missing, extra)].append((rel, flavor))

    print(f"triaged total={len(triaged)} pending/notrun={pending} "
          f"identical(now-pass)={len(identical)} diffgroups={len(groups)}")
    print()
    ranked = sorted(groups.items(), key=lambda kv: -len(kv[1]))
    for key, items in ranked:
        missing, extra = key[2], key[3]
        print(f"== {len(items)} cases | missing TS: {','.join(missing) or '-'} "
              f"| extra TS: {','.join(extra) or '-'}")
        for it in items[:6]:
            rel, flavor = it if isinstance(it, tuple) else (it, "")
            print(f"   {rel}")
            if flavor:
                print(f"      {flavor}")
        if len(items) > 6:
            print(f"   ... +{len(items)-6} more")
    if identical:
        print(f"\n-- now identical ({len(identical)}), candidates to untriage:")
        for r in identical[:20]:
            print(f"   {r}")
    out = sys.stdout

if __name__ == "__main__":
    os.chdir(os.path.dirname(os.path.dirname(os.path.abspath(__file__))))
    main()
