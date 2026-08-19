#!/usr/bin/env python3
"""Seed conformance and transpile baselines from the official TypeScript baselines.

conformance: for every official conformance test case (sorted), create
`tests/baselines/reference/conformance/<stem>.errors.txt` in the compact
one-diagnostic-per-line format from the official flat
`_submodules/TypeScript/tests/baselines/reference/<stem>.errors.txt`
(same convention as the compiler-suite seeding in seed_from_official.py).

transpile: copy the official output baselines from
`_submodules/TypeScript/tests/baselines/reference/transpile/` into
`tests/baselines/reference/transpile/`, normalizing the
`//// [Diagnostics reported]` sections to the compact one-line-per-diagnostic
convention: within such a section only the marker line and
`file(l,c): error TSxxxx: …` lines are kept; the `==== file (N errors) ====`
source-excerpt blocks (excerpts, squiggles, `!!!` lines) are dropped. This
matches what the Rust transpile runner assembles.

Usage: python3 tests/baselines/seed_suites.py [--suite conformance,transpile] [--dry] [--force]
"""

from __future__ import annotations

import re
import sys
from pathlib import Path

REPO = Path(__file__).resolve().parents[2]
OFFICIAL = REPO / "_submodules/TypeScript/tests/baselines/reference"
CONFORMANCE_CASES = REPO / "_submodules/TypeScript/tests/cases/conformance"

DIAG_RE = re.compile(r"^\S+\(\d+,\d+\): error TS\d+: ")
SUFFIXED_RE = re.compile(r"^(?P<stem>[^(]+)\([^)]+=[^)]*\)$")
DIAG_SECTION_MARKER = "//// [Diagnostics reported]"
SECTION_RE = re.compile(r"^//// \[")


def seed_conformance(dry: bool, force: bool) -> None:
    ours = REPO / "tests/baselines/reference/conformance"
    cases = sorted(
        str(p.relative_to(CONFORMANCE_CASES))
        for p in CONFORMANCE_CASES.rglob("*")
        if p.suffix in (".ts", ".tsx")
    )
    stems = {Path(rel).stem for rel in cases}
    created = kept = no_official = 0

    def seed_one(official: Path, target: Path) -> str:
        text = official.read_text(encoding="utf-8-sig", errors="replace")
        lines = [line.rstrip("\r") for line in text.splitlines() if DIAG_RE.match(line)]
        if not lines:
            return "empty"
        content = "\n".join(lines) + "\n"
        if target.exists() and (
            not force or target.read_text(encoding="utf-8", errors="replace") == content
        ):
            return "kept"
        if not dry:
            target.parent.mkdir(parents=True, exist_ok=True)
            target.write_text(content, encoding="utf-8")
        return "created"

    for rel in cases:
        stem = Path(rel).stem
        target = ours / f"{stem}.errors.txt"
        official = OFFICIAL / f"{stem}.errors.txt"
        if not official.is_file():
            no_official += 1
            continue
        result = seed_one(official, target)
        created += result == "created"
        kept += result == "kept"
        no_official += result == "empty"

    for official in sorted(OFFICIAL.glob("*.errors.txt")):
        m = SUFFIXED_RE.match(official.name[: -len(".errors.txt")])
        if not m or m.group("stem") not in stems:
            continue
        target = ours / official.name
        result = seed_one(official, target)
        created += result == "created"
        kept += result == "kept"
        no_official += result == "empty"

    print(
        f"conformance: {len(cases)} cases: {created} refs seeded, "
        f"{kept} already official-exact, {no_official} with no official errors baseline"
    )


def normalize_transpile_text(text: str) -> str:
    """Keep output sections verbatim; compact the diagnostics sections.

    Line endings normalize to LF (split on `\\n`, strip `\\r`) — matching the
    runner's CRLF→LF comparison-time normalization. Only `\\n` splits (not
    `str.splitlines`) so U+2028/vertical-tab characters inside emitted string
    literals stay intact.
    """
    out: list[str] = []
    in_diagnostics = False
    for line in text.split("\n"):
        stripped = line.rstrip("\r")
        if stripped == DIAG_SECTION_MARKER:
            in_diagnostics = True
            out.append(stripped)
            continue
        if SECTION_RE.match(stripped):
            # A new output section resumes verbatim content.
            in_diagnostics = False
        if in_diagnostics:
            if DIAG_RE.match(stripped):
                out.append(stripped)
            continue
        out.append(stripped)
    return "\n".join(out)


def seed_transpile(dry: bool, force: bool) -> None:
    official_dir = OFFICIAL / "transpile"
    ours = REPO / "tests/baselines/reference/transpile"
    created = kept = 0
    for official in sorted(official_dir.iterdir()):
        if not official.is_file():
            continue
        content = normalize_transpile_text(
            official.read_text(encoding="utf-8-sig", errors="replace")
        )
        target = ours / official.name
        if target.exists() and (
            not force or target.read_text(encoding="utf-8", errors="replace") == content
        ):
            kept += 1
            continue
        if not dry:
            ours.mkdir(parents=True, exist_ok=True)
            target.write_text(content, encoding="utf-8")
        created += 1
    print(f"transpile: {created} refs seeded, {kept} already normalized-exact")


def main() -> None:
    dry = "--dry" in sys.argv
    force = "--force" in sys.argv
    suite_spec = "conformance,transpile"
    for arg in sys.argv[1:]:
        if arg.startswith("--suite="):
            suite_spec = arg.split("=", 1)[1]
    suites = [s.strip() for s in suite_spec.split(",") if s.strip()]
    if "conformance" in suites:
        seed_conformance(dry, force)
    if "transpile" in suites:
        seed_transpile(dry, force)


if __name__ == "__main__":
    main()
