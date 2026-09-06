#!/usr/bin/env python3
"""差异清单 CSV: tsgo(go) 基线口径下的真实分歧 = go-pass ∧ rust-diff(当前实跑).

输入: go_configs.csv (Go 侧快照状态, 快照未变), rust_fresh_configs.csv (当前二进制实跑)
输出: scripts/gostd/divergence_worklist.csv
  测试用例文件路径, 原结果(=tsgo基线, go-pass 故与 go 实际一致), rust 结果, rust 当前处理方式(逐例填), 配置
"""
import csv, os, collections

REPO = "/home/cqh/workspace/typescript-rust"
G = f"{REPO}/scripts/gostd"

def norm(t): return t.replace("\r\n", "\n").strip()
def flat(t):
    i = t.find("\n==== ")
    return t[:i+1] if i >= 0 else t

_blc = {}
def own_baseline(suite, configured):
    k = (suite, configured)
    if k in _blc: return _blc[k]
    p = os.path.join(REPO, "tests/baselines/reference-go", suite, configured + ".errors.txt")
    v = norm(flat(open(p, encoding="utf-8", errors="replace").read())) if os.path.exists(p) else ""
    _blc[k] = v
    return v

go = {}
with open(f"{G}/go_configs.csv", encoding="utf-8") as f:
    for r in csv.DictReader(f):
        go[(r["suite"], r["case"], r["config"])] = r["status"]

fresh = {}
with open(f"{G}/rust_fresh_configs.csv", encoding="utf-8") as f:
    for r in csv.DictReader(f):
        fresh[(r["suite"], r["case"], r["config"])] = (r["status"], r["actual"])

snap = {}
with open(f"{G}/rust_configs.csv", encoding="utf-8") as f:
    for r in csv.DictReader(f):
        snap[(r["suite"], r["case"], r["config"])] = r["status"]

common = [k for k in fresh if k in go and go[k] == "go-pass"]
target = [k for k in common if fresh[k][0] == "rust-diff"]
fixed = [k for k in common if snap.get(k) == "rust-diff" and fresh[k][0] == "rust-pass"]
regressed = [k for k in common if snap.get(k) == "rust-pass" and fresh[k][0] == "rust-diff"]
ran_out = collections.Counter(fresh[k][0] for k in common)
print(f"go-pass 配置(双方可跑): {len(common)}  各状态: {dict(ran_out)}")
print(f"目标分歧(go-pass ∧ 当前 rust-diff): {len(target)}  用例数: {len({(k[0],k[1]) for k in target})}")
print(f"快照为 rust-diff、当前转好(剔除): {len(fixed)}")
print(f"快照为 rust-pass、当前新增分歧(纳入): {len(regressed)}")

rows = []
for k in sorted(target):
    suite, case, cfg = k
    stem = case
    for ext in (".ts", ".tsx"):
        if stem.endswith(ext): stem = stem[:-len(ext)]
    configured = f"{stem}({cfg})" if cfg else stem
    path = f"_submodules/TypeScript/tests/cases/{suite}/{case}"
    rows.append([path, own_baseline(suite, configured), fresh[k][1], "", cfg or "default"])

out = f"{G}/divergence_worklist.csv"
with open(out, "w", newline="", encoding="utf-8-sig") as f:
    w = csv.writer(f)
    w.writerow(["测试用例文件路径", "原结果", "rust 结果", "rust 当前处理方式", "配置"])
    w.writerows(rows)
print(f"已写出 {out}: {len(rows)} 行")

for k in sorted(fixed)[:20]: print("  转好:", k)
for k in sorted(regressed)[:20]: print("  新增:", k)
