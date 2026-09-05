#!/usr/bin/env python3
"""Final numbered test-matrix CSV:
测试编号, 文件位置, go测试结果, go测试预期差异, rust测试结果, rust测试预期差异
- basis: tsgo own baseline (flat segment, CRLF-normalized), byte comparison
- one row per (case, config) in the union of both runners' expansions"""
import csv, os, difflib, collections

OWN_REF = os.path.expanduser("~/workspace/typescript-go/tsc/testdata/baselines/reference")

def norm(t): return t.replace("\r\n", "\n").strip()
def flat(t):
    i = t.find("\n==== ")
    return t[:i+1] if i >= 0 else t

_blc = {}
def baseline(suite, configured):
    k = (suite, configured)
    if k in _blc: return _blc[k]
    p = os.path.join(OWN_REF, suite, configured + ".errors.txt")
    v = norm(flat(open(p, encoding="utf-8", errors="replace").read())) if os.path.exists(p) else ""
    _blc[k] = v
    return v

def ud(expected, actual):
    a = expected.split("\n") if expected else []
    b = actual.split("\n") if actual else []
    d = list(difflib.unified_diff(a, b, "预期(tsgo基线)", "实际输出", lineterm=""))
    return "\n".join(d) if d else "一致"

def ckey(name):
    if not name: return frozenset()
    return frozenset(p.strip().lower() for p in name.split(",") if p.strip())

def stem_of(case):
    b = os.path.basename(case)
    for ext in (".ts", ".tsx"):
        if b.endswith(ext): return b[:-len(ext)]
    return b

# ---- go rows: positional [suite, case, config, status, note, actual] ----
go = {}
for r in csv.reader(open('/tmp/gostd/go_configs.csv')):
    if not r or r[0] == 'suite': continue
    if r[0] == 'skip-case':                     # symlink misaligned row
        go[(r[1], r[2], frozenset())] = ("跳过", "symlink 用例")
        continue
    suite, case, cfg, status, note, actual = r[0], r[1], r[2], r[3], r[4], norm(r[5] if len(r) > 5 else '')
    configured = stem_of(case) + (f"({cfg})" if cfg else "")
    expect = baseline(suite, configured)
    res = "通过" if actual == expect else "不一致"
    if status.startswith('skip-config') or status == 'error-case':
        res, d = "跳过", note
    go[(suite, case, ckey(cfg))] = (res, ud(expect, actual) if res == "不一致" else (d if res == "跳过" else "一致"), actual)

# ---- rust rows: header [status, suite, case, config, note, actual] ----
ru = {}
for r in csv.DictReader(open('/tmp/gostd/rust_configs.csv')):
    st = r['status']
    suite, case = r['suite'], r['case']
    if st in ('skip-case', 'timeout-case', 'error-case'):
        ru[(suite, case, frozenset())] = ("跳过" if st == 'skip-case' else st, r['note'])
        continue
    cfg = r['config']
    configured = stem_of(case) + (f"({cfg})" if cfg else "")
    expect = baseline(suite, configured)
    actual = norm(r.get('actual', '') or '')
    res = "通过" if actual == expect else "不一致"
    if st == 'rust-skip':
        res, d = "跳过", r['note']
    ru[(suite, case, ckey(cfg))] = (res, ud(expect, actual) if res == "不一致" else (d if res == "跳过" else "一致"), actual)

# ---- merge ----
all_keys = sorted(set(go) | set(ru), key=lambda k: (k[0], k[1], sorted(k[2])))
rows = []
gn = rn = both = 0
for (suite, case, ck) in all_keys:
    g = go.get((suite, case, ck), ("未运行", "—", "", ""))
    r = ru.get((suite, case, ck), ("未运行", "—", "", ""))
    if g[0] != "未运行": gn += 1
    if r[0] != "未运行": rn += 1
    if g[0] != "未运行" and r[0] != "未运行": both += 1
    suffix = ",".join(sorted(x.split('=')[0] + '=' + x.split('=', 1)[1] if '=' in x else x for x in ck)) if ck else ""
    loc = f"{suite}/{case}" + (f" [{suffix}]" if suffix else "")
    if g[0] in ("通过", "不一致") and r[0] in ("通过", "不一致"):
        same = "相同" if g[2] == r[2] else "不同"
    else:
        same = "—"
    rows.append([suite, case, suffix, g[0], g[1], r[0], r[1], same])

import collections
c = collections.Counter((rr[3], rr[5], rr[7]) for rr in rows)

out = '/tmp/gostd/test_matrix.csv'
with open(out, 'w', newline='', encoding='utf-8-sig') as f:
    w = csv.writer(f)
    w.writerow(["测试编号", "文件位置", "go测试结果", "go测试预期差异", "rust测试结果", "rust测试预期差异", "双方预期差异是否相同"])
    for i, (suite, case, suffix, gres, gdiff, rres, rdiff, same) in enumerate(rows, 1):
        w.writerow([f"T{i:05d}", loc, gres, gdiff, rres, rdiff, same])

print(f"rows={len(rows)}  go-ran={gn}  rust-ran={rn}  both-ran={both}")
print(f"-> {out}")
for k, v in c.most_common(12): print(" ", k, v)
