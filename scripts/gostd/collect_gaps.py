#!/usr/bin/env python3
"""Re-run our worker on the 715 gap cases; save actual vs expected + unified diff."""
import os, re, json, csv, glob, shutil, subprocess, difflib
from concurrent.futures import ProcessPoolExecutor

EXE = sorted(glob.glob("/home/cqh/workspace/typescript-rust/target/release/deps/submodule_compiler-*"),
             key=os.path.getmtime)
EXE = [e for e in EXE if not e.endswith(".d")][0]
ROOT = "/home/cqh/workspace/typescript-rust/_submodules/TypeScript/tests/cases"
OWN_REF = os.path.expanduser("~/workspace/typescript-go/tsc/testdata/baselines/reference")
OUTDIR = "/tmp/gostd/gaps"

def norm(t): return t.replace("\r\n", "\n").strip()
def flat(t):
    i = t.find("\n==== ")
    return t[:i+1] if i >= 0 else t

def own_baseline(suite, configured):
    p = os.path.join(OWN_REF, suite, configured + ".errors.txt")
    if not os.path.exists(p): return ""
    return norm(flat(open(p, encoding="utf-8", errors="replace").read()))

def run_case(job):
    suite, rel, suffix = job
    base = os.path.basename(rel)
    path = os.path.join(ROOT, suite, rel)
    stem = base
    for ext in (".ts", ".tsx"):
        if stem.endswith(ext): stem = stem[:-len(ext)]
    configured = f"{stem}({suffix})" if suffix else stem
    out_path = f"/tmp/gostd/gap_{os.getpid()}.json"
    env = dict(os.environ, TSOX_SUBMODULE_WORKER=path, TSOX_SUBMODULE_OUT=out_path)
    try:
        subprocess.run([EXE, "--exact", "submodule_compiler_cases", "--nocapture"],
                       env=env, capture_output=True, timeout=180)
    except subprocess.TimeoutExpired:
        return (suite, rel, suffix, "TIMEOUT", "", "")
    try:
        entries = json.load(open(out_path))
    except Exception:
        return (suite, rel, suffix, "NO-PAYLOAD", "", "")
    text = ""
    for e in entries:
        if e.get("suffix", "") == suffix:
            text = e.get("text", "") or ""
            if text == "<no content>": text = ""
    expect = own_baseline(suite, configured)
    slug = rel.replace("/", "_")
    dpath = os.path.join(OUTDIR, f"{suite}__{slug}__{suffix or 'default'}.diff")
    actual_lines = text.split("\n") if text else []
    expected_lines = expect.split("\n") if expect else []
    diff = "\n".join(difflib.unified_diff(expected_lines, actual_lines,
                                          "tsgo-baseline", "rust-actual", lineterm=""))
    open(dpath, "w").write(diff)
    tag = "both-empty" if not expect and not text else ("extra-only" if expect and not text else ("missing-only" if text and not expect else "mixed"))
    return (suite, rel, suffix, tag, len(expected_lines), len(actual_lines))

def main():
    os.makedirs(OUTDIR, exist_ok=True)
    jobs = []
    for r in csv.DictReader(open('/tmp/gostd/mismatch_worklist.csv')):
        if r['go_status'] == 'go-pass' and r['rust_status'] == 'rust-diff':
            suffix = r['config'].strip("[]'\"")
            suffix = suffix.replace("'", "").replace('"', "")
            jobs.append((r['suite'], r['case'], suffix))
    print(f"{len(jobs)} gap configs", flush=True)
    tags = {}
    done = 0
    with ProcessPoolExecutor(max_workers=6) as ex:
        for r in ex.map(run_case, jobs, chunksize=4):
            tags[(r[0], r[1], r[2])] = (r[3], r[4], r[5])
            done += 1
            if done % 200 == 0: print(done, flush=True)
    import collections
    c = collections.Counter(v[0] for v in tags.values())
    print("tags:", dict(c))
    json.dump({f"{k[0]}|{k[1]}|{k[2]}": v for k, v in tags.items()},
              open("/tmp/gostd/gaps_tags.json", "w"))

if __name__ == "__main__":
    main()
