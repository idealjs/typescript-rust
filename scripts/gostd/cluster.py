import glob, re, collections, json
clusters = collections.Counter()
detail = collections.defaultdict(list)
code_re = {'-': re.compile(r'^-\S*?error (TS\d+)', re.M), '+': re.compile(r'^\+\S*?error (TS\d+)', re.M)}
strip_pos = lambda l: re.sub(r'\(\d+,\d+\)', '(X,X)', l)
for f in sorted(glob.glob('/tmp/gostd/gaps/*.diff')):
    name = f.split('/')[-1].replace('.diff', '')
    txt = open(f).read()
    minus = [l[1:] for l in txt.split('\n') if l.startswith('-') and not l.startswith('---')]
    plus = [l[1:] for l in txt.split('\n') if l.startswith('+') and not l.startswith('+++')]
    mc = set(code_re['-'].findall(txt))
    pc = set(code_re['+'].findall(txt))
    if not minus:
        kind = "we-extra:" + (','.join(sorted(pc))[:40] if pc else 'format')
    elif not plus:
        kind = "we-miss:" + ','.join(sorted(mc))[:44]
    elif mc == pc:
        only_pos = (sorted(map(strip_pos, minus)) == sorted(map(strip_pos, plus))) and minus != plus
        kind = "same-codes-" + ("pos-only" if only_pos else "text-diff")
    else:
        kind = "mixed:" + ','.join(sorted(mc))[:22] + ">" + ','.join(sorted(pc))[:22]
    clusters[kind] += 1
    detail[kind].append(name)
for k, n in clusters.most_common(30):
    print(f"{n:4d}  {k}")
print("total:", sum(clusters.values()))
json.dump(dict(detail), open('/tmp/gostd/gap_clusters.json', 'w'))
