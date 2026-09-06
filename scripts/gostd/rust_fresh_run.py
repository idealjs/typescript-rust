#!/usr/bin/env python3
"""Full corpus rerun on the current binary; per-config status + actual text on diff.

Output: scripts/gostd/rust_fresh_configs.csv (status,suite,case,config,actual)
Same baseline oracle as rust_diff.py: tsgo own baselines (flat segment, CRLF-normalized),
read from the in-repo mirror tests/baselines/reference-go.
"""
import os, re, json, csv, glob, subprocess
from concurrent.futures import ProcessPoolExecutor

REPO = "/home/cqh/workspace/typescript-rust"
EXE = sorted(glob.glob(f"{REPO}/target/release/deps/submodule_compiler-*"),
             key=os.path.getmtime)
EXE = [e for e in EXE if not e.endswith(".d")][0]
ROOT = f"{REPO}/_submodules/TypeScript/tests/cases"
OWN_REF = f"{REPO}/tests/baselines/reference-go"
OUT_DIR = "/tmp/gostd_fresh"

def load_skipped():
    src = open(f"{REPO}/tests/submodule_compiler.rs", encoding="utf-8").read()
    m = re.search(r"const TSGO_SKIPPED_TESTS: &\[&str\] = &\[(.*?)\];", src, re.S)
    return set(re.findall(r'"([^"]+)"', m.group(1)))

SKIPPED = load_skipped()

def norm(t): return t.replace("\r\n", "\n").strip()
def flat(t):
    i = t.find("\n==== ")
    return t[:i+1] if i >= 0 else t

_blc = {}
def own_baseline(suite, configured):
    k = (suite, configured)
    if k in _blc: return _blc[k]
    p = os.path.join(OWN_REF, suite, configured + ".errors.txt")
    v = norm(flat(open(p, encoding="utf-8", errors="replace").read())) if os.path.exists(p) else ""
    _blc[k] = v
    return v

def run_case(job):
    suite, rel = job
    base = os.path.basename(rel)
    if base in SKIPPED:
        return [("skip-case", suite, rel, "", "")]
    path = os.path.join(ROOT, suite, rel)
    out_path = f"{OUT_DIR}/rust_out_{os.getpid()}.json"
    if os.path.exists(out_path): os.remove(out_path)
    env = dict(os.environ, TSOX_SUBMODULE_WORKER=path, TSOX_SUBMODULE_OUT=out_path)
    try:
        subprocess.run([EXE, "--exact", "submodule_compiler_cases", "--nocapture"],
                       env=env, capture_output=True, timeout=120)
    except subprocess.TimeoutExpired:
        return [("timeout-case", suite, rel, "", "")]
    try:
        entries = json.load(open(out_path))
    except Exception:
        return [("error-case", suite, rel, "", "")]
    stem = base
    for ext in (".ts", ".tsx"):
        if stem.endswith(ext): stem = stem[:-len(ext)]
    rows = []
    for e in entries:
        suffix = e.get("suffix", "")
        configured = f"{stem}({suffix})" if suffix else stem
        if e.get("skip"):
            rows.append(("rust-skip", suite, rel, suffix, ""))
            continue
        text = e.get("text", "") or ""
        if text == "<no content>":
            text = ""
        expect = own_baseline(suite, configured)
        if norm(text) == expect:
            rows.append(("rust-pass", suite, rel, suffix, ""))
        else:
            rows.append(("rust-diff", suite, rel, suffix, text))
    return rows

def main():
    os.makedirs(OUT_DIR, exist_ok=True)
    jobs = []
    for suite in ("compiler", "conformance"):
        for dp, _, ns in os.walk(os.path.join(ROOT, suite)):
            for n in sorted(ns):
                if n.endswith((".ts", ".tsx")):
                    rel = os.path.relpath(os.path.join(dp, n), os.path.join(ROOT, suite))
                    jobs.append((suite, rel))
    jobs = [j for j in jobs if os.path.basename(j[1]) not in SKIPPED]
    print(f"{len(jobs)} cases, skipped-list {len(SKIPPED)}", flush=True)
    done = 0
    with open(f"{REPO}/scripts/gostd/rust_fresh_configs.csv", "w", newline="") as f:
        w = csv.writer(f); w.writerow(["status","suite","case","config","actual"])
        with ProcessPoolExecutor(max_workers=6) as ex:
            for rows in ex.map(run_case, jobs, chunksize=4):
                for r in rows: w.writerow(r)
                done += 1
                if done % 500 == 0:
                    f.flush(); print(done, flush=True)
    print("FRESH-RUN-DONE", flush=True)

if __name__ == "__main__":
    main()
