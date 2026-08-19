#!/usr/bin/env python3
"""Append verify-sweep FAIL entries to triaged.txt (post-fix leftovers).

Reads the per-suite run logs, extracts every FAILED case (with its
configuration suffix), maps it to its local baseline artifact name, and
appends a dated triaged.txt section. Skips entries already present.
Pure file processing.

Usage: python3 _scripts/triage_remaining.py [--dry]
"""
import re
import sys

REF = "tests/baselines/reference/triaged.txt"
LOGS = {
    "conformance": "tests/baselines/local/submodule_conformance_run.log",
    "compiler": "tests/baselines/local/submodule_run.log",
    "transpile": "tests/baselines/local/submodule_transpile_run.log",
}
# `name → stem/suffix/ext` extraction from FAILED lines like
# `foo.ts → foo(target=es5).errors.txt` (transpile) or bare case names
# resolved from FAIL lines `foo.ts (1.2s) — detail` / multi-config notes.
DATE = "2026-08-19"


def conformance_or_compiler_entries(log_path: str) -> list[str]:
    """Collect FAILED case display names from a compiler-style run log."""
    fails = []
    try:
        f = open(log_path, encoding="utf-8", errors="replace")
    except FileNotFoundError:
        return fails
    with f as f:
        for line in f:
            if " FAILED: " not in line:
                continue
            name = line.split(" FAILED: ", 1)[1].strip()
            # Multi-config failures carry the suffix AFTER the extension:
            # `deleteExpressionMustBeOptional.ts(strict=false)`.
            m = re.match(r"^(\S+?)\.(ts|tsx)(\([^)]*\))?$", name)
            if not m:
                continue
            stem, _, suffix = m.groups()
            fails.append(f"{stem}{suffix or ''}.errors.txt")
    return fails


def transpile_entries(log_path: str) -> list[str]:
    fails = []
    try:
        f = open(log_path, encoding="utf-8", errors="replace")
    except FileNotFoundError:
        return fails
    with f as f:
        for line in f:
            if " FAILED: " not in line:
                continue
            name = line.split(" FAILED: ", 1)[1].strip()
            # `case.ts → stem(suffix).ext` — baseline artifact after `→`.
            if " → " in name:
                fails.append(name.split(" → ", 1)[1].strip())
    return fails


def main():
    dry = "--dry" in sys.argv
    with open(REF, encoding="utf-8") as f:
        existing = set(
            ln.strip() for ln in f if ln.strip() and not ln.startswith("#")
        )
    new_entries: list[str] = []
    for suite, log in LOGS.items():
        entries = (
            transpile_entries(log)
            if suite == "transpile"
            else conformance_or_compiler_entries(log)
        )
        for e in entries:
            key = f"{suite}/{e}"
            if key not in existing:
                existing.add(key)
                new_entries.append(key)
    print(f"{len(new_entries)} new triaged entries")
    if dry or not new_entries:
        for e in new_entries[:20]:
            print(" ", e)
        return
    with open(REF, "a", encoding="utf-8") as f:
        f.write(
            f"\n## {DATE} 三套件验证跑后补登: 修复暴露的新差异与首轮排除过宽族（JSX 底层 2339/for-of 位置/parser 行数差等，下轮按类修） ##\n"
        )
        for e in new_entries:
            f.write(e + "\n")


if __name__ == "__main__":
    main()
