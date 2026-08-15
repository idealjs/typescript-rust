#!/usr/bin/env python3
"""Seed submodule errors-baseline references from the official TypeScript baselines.

For every official compiler test case (sorted, mirroring the Rust harness's
`collect_ts_files` + `sort`), create `tests/baselines/reference/compiler/
<stem>.errors.txt` in the compact one-diagnostic-per-line format when:

  - no reference exists yet (the first 1000 cases keep their audited refs), and
  - the official `_submodules/TypeScript/tests/baselines/reference/<stem>
    .errors.txt` exists.

Official error lines (`file.ts(line,col): error TSxxxx: message`) are copied
verbatim (CRLF stripped); the `==== file (N errors) ====` source-excerpt
sections are dropped. Cases whose official baseline is absent get no
reference file — the harness then expects `NO_CONTENT` (zero errors), which
is exactly the official expectation.

Usage: python3 tests/baselines/seed_from_official.py [--dry]
"""

from __future__ import annotations

import re
import sys
from pathlib import Path

REPO = Path(__file__).resolve().parents[2]
CASES_DIR = REPO / "_submodules/TypeScript/tests/cases/compiler"
OFFICIAL = REPO / "_submodules/TypeScript/tests/baselines/reference"
OURS = REPO / "tests/baselines/reference/compiler"

# `file.ts(12,5): error TS2345: ...` — the per-line diagnostic format shared
# by official baselines and our compact renderer (no space before the paren).
DIAG_RE = re.compile(r"^\S+\(\d+,\d+\): error TS\d+: ")


def main() -> None:
    dry = "--dry" in sys.argv
    cases = sorted(
        str(p.relative_to(CASES_DIR))
        for p in CASES_DIR.rglob("*")
        if p.suffix in (".ts", ".tsx")
    )
    created = kept = no_official = 0
    for rel in cases:
        stem = Path(rel).stem
        ours = OURS / f"{stem}.errors.txt"
        if ours.exists():
            kept += 1
            continue
        official = OFFICIAL / f"{stem}.errors.txt"
        if not official.is_file():
            no_official += 1
            continue
        text = official.read_text(encoding="utf-8-sig", errors="replace")
        lines = [
            line.rstrip("\r")
            for line in text.splitlines()
            if DIAG_RE.match(line)
        ]
        if not lines:
            # Official file exists but holds no parseable error lines (e.g. a
            # baseline for a runner mode we don't model). Treat as no-errors.
            no_official += 1
            continue
        if not dry:
            OURS.mkdir(parents=True, exist_ok=True)
            ours.write_text("\n".join(lines) + "\n", encoding="utf-8")
        created += 1
    print(
        f"{len(cases)} cases: {created} refs seeded, {kept} refs kept, "
        f"{no_official} with no official errors baseline"
    )


if __name__ == "__main__":
    main()
