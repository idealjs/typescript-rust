#!/usr/bin/env python3
"""Pair go_configs.csv (per-config go status) with rust_configs.csv (per-config
rust status) by normalized config name; emit per-case alignment + worklists."""
import csv, collections, json, os

def norm_name(name):
    # both sides: "k=v,k=v" (possibly empty) -> frozenset of pairs
    if not name: return frozenset()
    return frozenset(p.strip().lower() for p in name.split(",") if p.strip())

go = collections.defaultdict(dict)   # (suite,case) -> {normname: (status,note)}
for r in csv.DictReader(open('/tmp/gostd/go_configs.csv')):
    if r['status'] in ('',): continue
    go[(r['suite'], r['case'])][norm_name(r['config'])] = (r['status'], r['note'])
ru = collections.defaultdict(dict)
for r in csv.DictReader(open('/tmp/gostd/rust_configs.csv')):
    if r['status'].startswith(('skip-case','timeout-case','error-case')):
        ru.setdefault((r['suite'], r['case']), {})['<case>'] = (r['status'], r['note'])
        continue
    ru[(r['suite'], r['case'])][norm_name(r['config'])] = (r['status'], r['note'])

all_keys = sorted(set(go) | set(ru))
stat = collections.Counter()
mismatch_rows = []
for k in all_keys:
    g = go.get(k, {}); r = ru.get(k, {})
    # case-level failures first
    if '<case>' in r:
        stat['rust-case-error'] += 1
        mismatch_rows.append((k, '<case>', 'go=' + str(g.get('<case>', ('', ''))[0]), r['<case>'][0] + ': ' + r['<case>'][1]))
        continue
    configs = set(g) | set(r)
    case_aligned = True
    for cn in configs:
        gs = g.get(cn, ('go-missing-config', ''))[0]
        rs = r.get(cn, ('rust-missing-config', ''))[0]
        gskip = gs.startswith('go-') and gs != 'go-pass' and gs != 'go-diff'
        rskip = rs.startswith('rust-skip')
        if gskip or rskip:
            stat['skipped-config'] += 1
            continue
        gok = gs == 'go-pass'
        rok = rs == 'rust-pass'
        if gok and rok:
            stat['aligned-pass'] += 1
        elif not gok and not rok:
            stat['aligned-diff'] += 1
        else:
            case_aligned = False
            key = 'rust-diff/go-pass' if gok else 'rust-pass/go-diff'
            stat[key] += 1
            mismatch_rows.append((k[0], k[1], str(sorted(cn)), gs, rs))
    stat['cases-aligned'] += case_aligned
    stat['cases-total'] += 1

print("config-level:", {k: v for k, v in stat.items() if not k.startswith('cases')})
print(f"cases: aligned {stat['cases-aligned']} / {stat['cases-total']}")
with open('/tmp/gostd/mismatch_worklist.csv', 'w', newline='') as f:
    w = csv.writer(f); w.writerow(['suite','case','config','go_status','rust_status'])
    for row in mismatch_rows: w.writerow(row)
print(f"mismatch configs: {len(mismatch_rows)} -> /tmp/gostd/mismatch_worklist.csv")
