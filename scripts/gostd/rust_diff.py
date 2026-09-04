#!/usr/bin/env python3
"""Rust side of the differential: run the tsox worker per case (its own config
expansion), classify each config's rendered output against the SAME tsgo own
baselines the Go side uses."""
import os, re, json, csv, glob, subprocess
from concurrent.futures import ProcessPoolExecutor

EXE = sorted(glob.glob("/home/cqh/workspace/typescript-rust/target/release/deps/submodule_compiler-*"),
             key=os.path.getmtime)
EXE = [e for e in EXE if not e.endswith(".d")][0]
ROOT = "/home/cqh/workspace/typescript-rust/_submodules/TypeScript/tests/cases"
OWN_REF = os.path.expanduser("~/workspace/typescript-go/tsc/testdata/baselines/reference")
SKIPPED = json.load(open("/tmp/gostd/skipped.json"))

def norm(t): return t.replace("\r\n", "\n").strip()
def flat(t):
    i = t.find("\n==== ")
    return t[:i+1] if i >= 0 else t

def own_baseline(suite, configured):
    p = os.path.join(OWN_REF, suite, configured + ".errors.txt")
    if not os.path.exists(p): return ""
    return norm(flat(open(p, encoding="utf-8", errors="replace").read()))

def run_case(job):
    suite, rel = job
    base = os.path.basename(rel)
    if base in SKIPPED:
        return [("skip-case", suite, rel, "", "tsgo skippedTests")]
    path = os.path.join(ROOT, suite, rel)
    out_path = f"/tmp/gostd/rust_out_{os.getpid()}.json"
    env = dict(os.environ, TSOX_SUBMODULE_WORKER=path, TSOX_SUBMODULE_OUT=out_path)
    try:
        subprocess.run([EXE, "--exact", "submodule_compiler_cases", "--nocapture"],
                       env=env, capture_output=True, timeout=120)
    except subprocess.TimeoutExpired:
        return [("timeout-case", suite, rel, "", "worker timeout")]
    try:
        entries = json.load(open(out_path))
    except Exception:
        return [("error-case", suite, rel, "", "no payload")]
    stem = base
    for ext in (".ts", ".tsx"):
        if stem.endswith(ext): stem = stem[:-len(ext)]
    rows = []
    for e in entries:
        suffix = e.get("suffix", "")
        configured = f"{stem}({suffix})" if suffix else stem
        if "skip" in e and e["skip"]:
            rows.append(("rust-skip", suite, rel, suffix, e["skip"]))
            continue
        text = e.get("text", "") or ""
        if text == "<no content>":
            text = ""          # harness sentinel == empty baseline
        expect = own_baseline(suite, configured)
        status = "rust-pass" if norm(text) == expect else "rust-diff"
        rows.append((status, suite, rel, suffix, ""))
    return rows

def main():
    jobs = []
    for suite in ("compiler", "conformance"):
        for dp, _, ns in os.walk(os.path.join(ROOT, suite)):
            for n in sorted(ns):
                if n.endswith((".ts", ".tsx")):
                    jobs.append((suite, os.path.relpath(os.path.join(dp, n), os.path.join(ROOT, suite))))
    jobs = [j for j in jobs if os.path.basename(j[1]) not in SKIPPED]
    print(f"{len(jobs)} cases", flush=True)
    done = 0
    with open("/tmp/gostd/rust_configs.csv", "w", newline="") as f:
        w = csv.writer(f); w.writerow(["status","suite","case","config","note"])
        with ProcessPoolExecutor(max_workers=6) as ex:
            for rows in ex.map(run_case, jobs, chunksize=4):
                for r in rows: w.writerow(r)
                done += 1
                if done % 500 == 0: f.flush(); print(done, flush=True)
    print("RUST-DIFF-DONE", flush=True)

if __name__ == "__main__":
    main()
