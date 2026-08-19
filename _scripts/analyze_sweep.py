#!/usr/bin/env python3
"""Sweep-failure analysis: compare fresh local artifacts against references.

For a given baseline subfolder (compiler/conformance/transpile), walk
tests/baselines/local/<subfolder>/ and classify every artifact that differs
from its reference (or whose .delete marker / new-baseline state differs):
group by (missing TS codes, extra TS codes) signature with a first-diff-line
flavor, and print a histogram of largest groups first.

Pure file processing — never runs the compiler.

Usage: python3 _scripts/analyze_sweep.py [subfolder] [--flavor] [--head N]
  subfolder  default: all of compiler, conformance, transpile
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


def analyze(subfolder: str):
    failures = []  # (rel, kind, ref_codes, loc_codes, flavor)
    loc_dir = os.path.join(LOC, subfolder)
    if not os.path.isdir(loc_dir):
        print(f"{subfolder}: no local artifacts")
        return
    for name in sorted(os.listdir(loc_dir)):
        if name.endswith(".delete"):
            rel = name[: -len(".delete")]
            ref = load(os.path.join(REF, subfolder, rel))
            if ref is None:
                continue  # reference also absent → consistent
            ref_codes = sorted(set(CODE.findall(ref)))
            failures.append((rel, "UNDER", ref_codes, [], "official errors, we emit none"))
            continue
        ref = load(os.path.join(REF, subfolder, name))
        loc = load(os.path.join(LOC, subfolder, name))
        if ref is None:
            loc_codes = sorted(set(CODE.findall(loc or "")))
            failures.append((name, "OVER", [], loc_codes, "no official baseline, we emit errors"))
            continue
        if ref == loc:
            continue  # matching artifact (PASS or triaged-diff with identical text)
        ref_codes = sorted(set(CODE.findall(ref)))
        loc_codes = sorted(set(CODE.findall(loc)))
        rl, ll = (ref or "").splitlines(), (loc or "").splitlines()
        flavor = ""
        for a, b in zip(rl, ll):
            if a != b:
                flavor = "REF: " + a[:100] + " || LOC: " + b[:100]
                break
        if not flavor and len(rl) != len(ll):
            flavor = f"line-count {len(rl)} vs {len(ll)}"
        failures.append((name, "DIFF", ref_codes, loc_codes, flavor))

    groups = defaultdict(list)
    for rel, kind, ref_codes, loc_codes, flavor in failures:
        missing = tuple(c for c in ref_codes if c not in loc_codes)
        extra = tuple(c for c in loc_codes if c not in ref_codes)
        groups[(kind, missing, extra)].append((rel, flavor))

    print(f"=== {subfolder}: {len(failures)} differing artifacts, "
          f"{len(groups)} signature groups ===")
    show_flavor = "--flavor" in sys.argv
    head = 60
    for i, arg in enumerate(sys.argv):
        if arg == "--head" and i + 1 < len(sys.argv):
            head = int(sys.argv[i + 1])
    for (kind, missing, extra), entries in sorted(
        groups.items(), key=lambda kv: -len(kv[1])
    )[:head]:
        print(f"\n[{kind} missing={','.join(missing) or '-'} "
              f"extra={','.join(extra) or '-'}] x{len(entries)}")
        for rel, flavor in entries[: (5 if show_flavor else 12)]:
            print(f"  {rel}")
            if show_flavor and flavor:
                print(f"      {flavor[:200]}")
        if len(entries) > (5 if show_flavor else 12):
            print(f"  … +{len(entries) - (5 if show_flavor else 12)} more")


def main():
    args = [a for a in sys.argv[1:] if not a.startswith("--") and not a.isdigit()]
    folders = args or ["compiler", "conformance", "transpile"]
    for f in folders:
        analyze(f)


if __name__ == "__main__":
    main()
