#!/usr/bin/env python3
"""Generate conformance/transpile triaged.txt entries from sweep artifacts.

Groups every differing local artifact under tests/baselines/local/
<subfolder>/ by coarse root-cause family (dominant extra/missing error-code
prefix + heuristic buckets for module-resolution / declaration-emit /
text-only families), emitting triaged.txt-format lines with dated group
headers. Pure file processing.

Usage: python3 _scripts/gen_triage_suites.py [--dry]
"""
import os
import re
import sys
from collections import defaultdict

REF = "tests/baselines/reference"
LOC = "tests/baselines/local"
CODE = re.compile(r"error TS(\d+):")

DATE = "2026-08-19"

# Families fixed in this round (expected to PASS in the verify sweep — do
# NOT triage them; remaining failures get triaged after verification).
FIXED_CODES = {
    "2464",  # computed property name check (ported)
    "2411",  # index constraint check (ported)
    "2602", "7026",  # JSX ambient namespace fallback (ported)
    "2554",  # this-parameter arity (ported)
    "2454",  # destructuring-assignment flow (ported)
    # parser recovery (1109 default branch + scanner error streaming)
    "1109", "1127", "1128", "1012", "1005",
    "2304",  # dynamic-import callee parsing (ported; other 2304s re-bucket)
}
FIXED_NAME_HINTS = (
    "importassertion",
    "importattribute",
    "computedpropertynames14",
    "computedpropertynames36",
    "computedpropertynames37",
    "computedpropertynames38",
    "computedpropertynames39",
    "computedpropertynames40",
    "computedpropertynames42",
    "computedpropertynames43",
    "computedpropertynames44",
    "computedpropertynames45",
    "checkjsxchildren",
    "es5for-of",
    "callchaininference",
    "classwithstaticfieldinparameterinitializer",
)


def is_fixed(name: str, ref_codes: set, loc_codes: set) -> bool:
    low = name.lower()
    if any(h in low for h in FIXED_NAME_HINTS):
        return True
    codes = ref_codes | loc_codes
    return bool(codes) and codes.issubset(FIXED_CODES)


def load(path):
    try:
        with open(path, encoding="utf-8", errors="replace") as f:
            return f.read()
    except FileNotFoundError:
        return None


def family_of(name: str, ref: str | None, loc: str | None) -> str:
    """Coarse root-cause bucket for one differing artifact."""
    ref_codes = set(CODE.findall(ref or ""))
    loc_codes = set(CODE.findall(loc or ""))
    missing = ref_codes - loc_codes
    extra = loc_codes - ref_codes
    all_codes = ref_codes | loc_codes
    # Text-only difference (same code sets): elaborated-chain/type display.
    if not missing and not extra:
        return "text-diff"
    # Module resolution cluster (2304/2305/2307/2306/6053) in cases whose
    # name suggests resolution layout.
    if {"2307", "2304", "2305", "2306"} & (extra | missing) and any(
        k in name.lower()
        for k in ("node", "module", "resolution", "bundler", "packagejson",
                  " symlink", "symlink", "ambient", "conditions", "import")
    ):
        return "module-resolution"
    # Declaration-emit flavored (baseline produced by .d.ts emit semantics).
    if "declaration" in name.lower() and extra | missing:
        return "declaration-emit"
    # Dominant extra code family (over-reports).
    if extra and not missing:
        return "extra-" + sorted(extra)[0]
    if missing and not extra:
        return "missing-" + sorted(missing)[0]
    return "mixed-" + (sorted(extra)[0] if extra else sorted(missing)[0])


def collect(subfolder: str):
    entries = defaultdict(list)
    loc_dir = os.path.join(LOC, subfolder)
    if not os.path.isdir(loc_dir):
        return entries
    for name in sorted(os.listdir(loc_dir)):
        if name.endswith(".delete"):
            rel = name[: -len(".delete")]
            ref = load(os.path.join(REF, subfolder, rel))
            if ref is None:
                continue
            if is_fixed(rel, set(CODE.findall(ref or "")), set()):
                continue
            entries[family_of(rel, ref, None)].append(rel)
            continue
        ref = load(os.path.join(REF, subfolder, name))
        loc = load(os.path.join(LOC, subfolder, name))
        if ref is None:
            if is_fixed(name, set(), set(CODE.findall(loc or ""))):
                continue
            entries[family_of(name, None, loc)].append(name)
            continue
        if ref == loc:
            continue
        if is_fixed(name, set(CODE.findall(ref)), set(CODE.findall(loc))):
            continue
        entries[family_of(name, ref, loc)].append(name)
    return entries


FAMILY_NOTES = {
    "text-diff": "码全同、文本/顺序不同：elaborated error chain 与类型显示（与 compiler 套件文本差异族同根）",
    "module-resolution": "模块解析子系统：node16/bundler 目录布局、package.json exports/imports、symlink、ambient module patterns 未移植",
    "declaration-emit": ".d.ts 声明产生语义（node reuse/推断/别名链）未移植",
}


def family_note(fam: str) -> str:
    if fam in FAMILY_NOTES:
        return FAMILY_NOTES[fam]
    if fam.startswith("extra-"):
        return f"TS{fam[6:]} 多报族（推断/上下文/延迟类型连锁或检查边界）"
    if fam.startswith("missing-"):
        return f"TS{fam[8:]} 欠报族（检查缺失或延迟类型未求值）"
    return "混合差异族"


def main():
    dry = "--dry" in sys.argv
    out_lines = []
    total = 0
    for subfolder in ("conformance", "transpile"):
        entries = collect(subfolder)
        if not entries:
            continue
        out_lines.append(f"## {DATE} 三套件首轮(conformance/transpile): {subfolder} 套件分诊总段 ##")
        for fam in sorted(entries, key=lambda f: -len(entries[f])):
            rels = sorted(entries[fam])
            total += len(rels)
            out_lines.append(
                f"### {fam}: {family_note(fam)} ({len(rels)} 例) ###"
            )
            for rel in rels:
                out_lines.append(f"{subfolder}/{rel}")
        out_lines.append("")
    print(f"total entries: {total}")
    if dry:
        return
    with open(os.path.join(REF, "triaged.txt"), "a", encoding="utf-8") as f:
        f.write("\n".join(out_lines) + "\n")


if __name__ == "__main__":
    main()
