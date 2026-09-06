#!/usr/bin/env python3
"""Run TypeScript test cases by path or worklist index on BOTH implementations
(tsgo + tsox), classify each side's output against the tsgo own-baseline tree.

usage: test.sh <case-path | worklist-row-number>...
  - path: bare filename, suite-relative (compiler/x.ts), corpus-relative, absolute
  - number: 1-based row number in scripts/gostd/divergence_worklist.csv
Exit code: 0 = both sides consistent, 1 = any divergence, 2 = usage error.
"""
import os, re, sys, json, shutil, subprocess, difflib, argparse, csv

SCRIPT_DIR = os.path.dirname(os.path.abspath(__file__))
REPO = os.path.dirname(os.path.dirname(SCRIPT_DIR))
ROOT = os.path.join(REPO, "_submodules/TypeScript/tests/cases")
LIBS = os.path.join(ROOT, "../lib")
OWN_REF = os.path.join(REPO, "tests/baselines/reference-go")
TSGO = os.environ.get("TSGO_BIN",
                      os.path.expanduser("~/workspace/typescript-go/built/local/tsgo"))
WORK = "/tmp/tsox_case_run"
SUITES = ("compiler", "conformance")

sys.path.insert(0, SCRIPT_DIR)
import godecls

DECLS = None
VARY = None
SKIPPED = None

OPTION_RE = re.compile(r"^\s*//\s*@(\w+)\s*:\s*(.*)$")
FILE_DIR_RE = re.compile(r"^\s*//\s*@filename:\s*(\S+)\s*$", re.I)
SYMLINK_RE = re.compile(r"^\s*//\s*@link\s*:\s*([^\\r\\n]*)\s*->\s*([^\\r\\n]*)")
REFERENCES_RE = re.compile(r"reference\s+path")
UNTRANSLATABLE = {"baseurl", "paths", "rootdirs", "typesversions", "currentdirectory",
                  "noimplicitreferences", "runexternalcode", "symlink", "filename"}


def init_metadata():
    global DECLS, VARY, SKIPPED
    decls = godecls.parse_decls()
    maps = godecls.parse_enum_maps()
    for name, d in decls.items():
        if d["map"] and d["map"] in maps:
            d["values"] = maps[d["map"]]
        elif d["kind"] == "Boolean":
            d["values"] = ["true", "false"]
    vary = {n: d for n, d in decls.items()
            if d["kind"] in ("Boolean", "Enum") and not d["cmdline_only"] and d["affects"]}
    vary["noemit"] = decls.get("noemit", {"kind": "Boolean", "values": ["true", "false"]})
    vary["isolatedmodules"] = decls.get("isolatedmodules",
                                        {"kind": "Boolean", "values": ["true", "false"]})
    DECLS, VARY = decls, vary
    src = open(os.path.join(REPO, "tests/submodule_compiler.rs"), encoding="utf-8").read()
    m = re.search(r"const TSGO_SKIPPED_TESTS: &\[&str\] = &\[(.*?)\];", src, re.S)
    SKIPPED = set(re.findall(r'"([^"]+)"', m.group(1)))


def norm(t): return t.replace("\r\n", "\n").strip()
def flat(t):
    i = t.find("\n==== ")
    return t[:i + 1] if i >= 0 else t


def own_baseline(suite, configured):
    p = os.path.join(OWN_REF, suite, configured + ".errors.txt")
    if not os.path.exists(p):
        return None
    return norm(flat(open(p, encoding="utf-8", errors="replace").read()))


def resolve_case(arg):
    cand = arg
    for prefix in (REPO + "/", "_submodules/TypeScript/tests/cases/"):
        if cand.startswith(prefix):
            cand = cand[len(prefix):]
    for suite in SUITES:
        p = os.path.join(ROOT, suite, cand)
        if os.path.isfile(p):
            return suite, cand
    base = os.path.basename(cand)
    names = [base] if base.endswith((".ts", ".tsx")) else \
            [base + e for e in (".ts", ".tsx")]
    hits = []
    for suite in SUITES:
        for dp, _, ns in os.walk(os.path.join(ROOT, suite)):
            for n in ns:
                if n in names:
                    hits.append((suite, os.path.relpath(os.path.join(dp, n),
                                                        os.path.join(ROOT, suite))))
    if len(hits) == 1:
        return hits[0]
    if not hits:
        raise SystemExit(f"2: 未找到用例: {arg}")
    raise SystemExit("2: 用例名歧义，请带路径:\n  " +
                     "\n  ".join(f"{s}/{r}" for s, r in hits))


def split_values(option, value):
    if not value:
        return None
    star = False
    includes, excludes = [], []
    for s in value.split(","):
        s = s.strip()
        if not s:
            continue
        if s == "*":
            star = True
        elif s.startswith(("-", "!")):
            excludes.append(s[1:])
        else:
            includes.append(s)
    if not includes and not star and not excludes:
        return None
    vals = []
    if star:
        d = VARY.get(option) or DECLS.get(option) or {}
        vals.extend(d.get("values") or [])
    vals.extend(includes)
    out, seen = [], set()
    for v in vals:
        if v in excludes or v.lstrip("!-") in excludes:
            continue
        k = v.lower()
        if k not in seen:
            seen.add(k)
            out.append(v)
    return out


def parse_case(text, fallback_name):
    units, settings = [], {}
    cur_name, cur = None, []
    cur_dir = ""
    for line in text.split("\n"):
        if SYMLINK_RE.match(line):
            continue
        m = OPTION_RE.match(line)
        if m:
            name, val = m.group(1).lower(), m.group(2).strip()
            if name == "currentdirectory":
                cur_dir = val
            elif name == "filename":
                if cur_name is not None:
                    units.append((cur_name, "\n".join(cur)))
                cur_name, cur = val.strip(), []
            else:
                settings[name] = val.rstrip(";").strip()
            continue
        if cur:
            cur.append(line)
        elif line.strip() != "":
            cur.append(line)
    if cur_name is None:
        cur_name = fallback_name
    units.append((cur_name, "\n".join(cur)))
    return units, settings, cur_dir


def build_configs(settings):
    option_entries, variation, nonvarying = [], 1, {}
    for opt, val in settings.items():
        if opt in VARY:
            entries = split_values(opt, val)
            if entries is None:
                continue
            if len(entries) > 1:
                variation *= len(entries)
                if variation > 25:
                    return None, "variations>25"
                option_entries.append([opt] + entries)
            elif len(entries) == 1:
                nonvarying[opt] = entries[0]
        else:
            nonvarying[opt] = val
    configs = []
    if option_entries:
        import itertools
        for combo in itertools.product(*[e[1:] for e in option_entries]):
            sel = {e[0]: v for e, v in zip(option_entries, combo)}
            name = ",".join(f"{k}={sel[k].lower()}" for k in sorted(sel))
            merged = dict(sel)
            merged.update(nonvarying)
            configs.append((name, merged))
    else:
        configs.append(("", dict(nonvarying)))
    return configs, None


def to_cli(options):
    flags, bad = [], []
    for opt, val in sorted(options.items()):
        if opt in ("filename", "currentdirectory", "symlink",
                   "noimplicitreferences", "runexternalcode"):
            continue
        if opt in UNTRANSLATABLE:
            bad.append(opt)
            continue
        d = DECLS.get(opt)
        if d is None:
            bad.append(opt)
            continue
        if d["kind"] == "List":
            flags += [f"--{d['orig']}", ",".join(x.strip() for x in val.split(",")
                                                 if x.strip())]
        elif d["kind"] == "Boolean":
            if val.lower() in ("true", "false"):
                flags += [f"--{d['orig']}", val.lower()]
            else:
                bad.append(opt)
        else:
            flags += [f"--{d['orig']}", val]
    return flags, bad


def run_go(suite, rel, case_dir):
    path = os.path.join(ROOT, suite, rel)
    base = os.path.basename(rel)
    stem, _ = os.path.splitext(base)
    if base in SKIPPED:
        return [{"config": "", "status": "skip-case", "note": "tsgo skippedTests"}]
    text = open(path, encoding="utf-8", errors="replace").read()
    units, settings, cur_dir = parse_case(text, base)
    if any(SYMLINK_RE.match(l) for l in text.split("\n")):
        return [{"config": "", "status": "skip-case", "note": "symlink case"}]
    if not cur_dir:
        cur_dir = "/.src"
    if cur_dir not in ("/.src", "/"):
        return [{"config": "", "status": "skip-case", "note": "custom currentdirectory"}]
    ts_cfg = None
    for i, (n, c) in enumerate(units):
        if n == "tsconfig.json" or n.endswith("tsconfig.json") or n == "jsconfig.json":
            ts_cfg = (i, n, c)
            break
    configs, err = build_configs(settings)
    if err:
        return [{"config": "", "status": "error-case", "note": err}]
    shutil.rmtree(case_dir, ignore_errors=True)
    os.makedirs(case_dir, exist_ok=True)
    for i, (n, c) in enumerate(units):
        dst = os.path.join(case_dir, n.lstrip("/"))
        os.makedirs(os.path.dirname(dst) or case_dir, exist_ok=True)
        open(dst, "w", encoding="utf-8").write(c)
    for _, c in units:
        for lib in set(re.findall(r'reference path="/\.lib/([^"]+)"', c)):
            src = os.path.join(LIBS, lib)
            if os.path.exists(src):
                dst = os.path.join(case_dir, ".lib", lib)
                os.makedirs(os.path.dirname(dst), exist_ok=True)
                shutil.copy(src, dst)
    last_content = units[-1][1] if units else ""
    if ts_cfg is not None:
        entries = None
    elif "require" in last_content or REFERENCES_RE.search(last_content):
        entries = [units[-1][0].lstrip("/")]
    else:
        entries = [n.lstrip("/") for (n, _) in units]
    rows = []
    for cfg_name, options in configs:
        configured = f"{stem}({cfg_name})" if cfg_name else stem
        raw_expect = own_baseline(suite, configured)
        expect = None if raw_expect is None else \
            norm("\n".join(l for l in raw_expect.split("\n")
                           if not l.startswith("!!!")))
        flags, bad = to_cli(options)
        if bad:
            rows.append({"config": cfg_name, "status": "skip-config",
                         "note": f"untranslatable: {','.join(bad)}", "expect": expect})
            continue
        if ts_cfg is not None:
            args = [TSGO, "--noEmit"] + flags + ["-p", "."]
        else:
            args = [TSGO, "--noEmit"] + flags + entries
        try:
            p = subprocess.run(args, cwd=case_dir, capture_output=True, timeout=60,
                               text=True, errors="replace")
            out = p.stdout + p.stderr
        except subprocess.TimeoutExpired:
            rows.append({"config": cfg_name, "status": "timeout", "note": "",
                         "expect": expect})
            continue
        actual = []
        for l in out.splitlines():
            l = l.replace(case_dir + "/", "").replace(case_dir, "").lstrip("./")
            if re.search(r"\(\d+,\d+\): \w+ TS\d+:", l) \
                    or re.match(r"(?:error|warning|message) TS\d+:", l) \
                    or l.startswith(("  ", "!!!")):
                actual.append(l.rstrip())
        actual = norm("\n".join(actual))
        rows.append({"config": cfg_name, "status": "pass" if actual == (expect or "") else "diff",
                     "note": "", "expect": expect, "actual": actual})
    return rows


def rust_exe():
    env = os.environ.get("RUST_EXE")
    if env:
        return env
    import glob
    cands = [e for e in glob.glob(os.path.join(REPO, "target/release/deps/submodule_compiler-*"))
             if not e.endswith(".d")]
    if not cands:
        raise SystemExit("2: 未找到测试二进制，先构建: "
                         "cargo test --release --test submodule_compiler --no-run")
    return max(cands, key=os.path.getmtime)


def run_rust(suite, rel):
    path = os.path.join(ROOT, suite, rel)
    base = os.path.basename(rel)
    stem, _ = os.path.splitext(base)
    if base in SKIPPED:
        return [{"config": "", "status": "skip-case", "note": "tsgo skippedTests"}]
    out_path = f"{WORK}/rust_out_{os.getpid()}.json"
    if os.path.exists(out_path):
        os.remove(out_path)
    env = dict(os.environ, TSOX_SUBMODULE_WORKER=path, TSOX_SUBMODULE_OUT=out_path)
    try:
        subprocess.run([rust_exe(), "--exact", "submodule_compiler_cases", "--nocapture"],
                       env=env, capture_output=True, timeout=120)
    except subprocess.TimeoutExpired:
        return [{"config": "", "status": "timeout", "note": "worker timeout"}]
    try:
        entries = json.load(open(out_path))
    except Exception:
        return [{"config": "", "status": "error-case", "note": "no worker payload"}]
    rows = []
    for e in entries:
        suffix = e.get("suffix", "")
        configured = f"{stem}({suffix})" if suffix else stem
        if e.get("skip"):
            rows.append({"config": suffix, "status": "skip-config", "note": e["skip"]})
            continue
        text = e.get("text", "") or ""
        if text == "<no content>":
            text = ""
        expect = own_baseline(suite, configured)
        rows.append({"config": suffix,
                     "status": "pass" if norm(text) == (expect or "") else "diff",
                     "note": "", "expect": expect, "actual": norm(text)})
    return rows


def show(tag, r):
    st = r["status"]
    if st in ("skip-case", "skip-config", "timeout", "error-case"):
        extra = f" ({r['note']})" if r.get("note") else ""
        print(f"  {tag:<5}: {st}{extra}")
        return st in ("skip-case", "skip-config")
    if r["status"] == "pass":
        print(f"  {tag:<5}: 一致")
        return True
    expect = r.get("expect")
    print(f"  {tag:<5}: 不一致" +
          ("" if expect is not None else "（基线缺失，按空判定）"))
    d = list(difflib.unified_diff((expect or "").split("\n"),
                                  r["actual"].split("\n") if r["actual"] else [],
                                  "基线", tag, lineterm=""))
    for l in d[2:]:
        print("    " + l)
    return False


def resolve_index(num):
    p = os.path.join(SCRIPT_DIR, "divergence_worklist.csv")
    rows = list(csv.DictReader(open(p, encoding="utf-8-sig")))
    n = int(num)
    if not (1 <= n <= len(rows)):
        raise SystemExit(f"2: 序号超出范围 1–{len(rows)}: {num}")
    r = rows[n - 1]
    path = r["测试用例文件路径"]
    rel = path.split("/cases/", 1)[1]
    return rel, ("" if r["配置"] == "default" else r["配置"])


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("--side", choices=("both", "go", "rust"), default="both")
    ap.add_argument("--config", default="",
                    help="只显示指定配置（default/空=全部）")
    ap.add_argument("--keep", action="store_true",
                    help="保留 Go 侧物化目录 /tmp/tsox_case_run 供检查")
    ap.add_argument("targets", nargs="+",
                    help="用例路径或 divergence_worklist.csv 行号")
    a = ap.parse_args()
    init_metadata()
    os.makedirs(WORK, exist_ok=True)
    any_diff = False
    for arg in a.targets:
        cfg_filter = a.config
        if re.fullmatch(r"\d+", arg):
            rel, idx_cfg = resolve_index(arg)
            suite = rel.split("/", 1)[0]
            if idx_cfg and not cfg_filter:
                cfg_filter = idx_cfg
            rel = rel.split("/", 1)[1]
            print(f"=== #{arg} -> {suite}/{rel}" +
                  (f" [配置 {cfg_filter}]" if cfg_filter else ""))
        else:
            suite, rel = resolve_case(arg)
            print(f"=== {suite}/{rel}")
        base = os.path.basename(rel)
        if base in SKIPPED:
            print("  tsgo skippedTests 成员，双侧 runner 均跳过")
            continue
        case_dir = os.path.join(WORK, f"{suite}_{rel.replace('/', '_')}")
        go_rows = run_go(suite, rel, case_dir) if a.side in ("both", "go") else None
        rust_rows = run_rust(suite, rel) if a.side in ("both", "rust") else None
        if not a.keep:
            shutil.rmtree(case_dir, ignore_errors=True)
        names = []
        for rows in (go_rows, rust_rows):
            if rows:
                for r in rows:
                    if r["config"] not in names:
                        names.append(r["config"])
        gmap = {r["config"]: r for r in go_rows or []}
        rmap = {r["config"]: r for r in rust_rows or []}
        for cfg in names:
            if cfg_filter and cfg != ("" if cfg_filter == "default" else cfg_filter):
                continue
            label = cfg or "default"
            print(f"  ── [{label}]")
            if a.side in ("both", "go") and cfg in gmap:
                ok = show("go", gmap[cfg])
                any_diff |= not ok
            if a.side in ("both", "rust") and cfg in rmap:
                ok = show("rust", rmap[cfg])
                any_diff |= not ok
            if a.side == "both":
                if cfg not in gmap:
                    print("  go    : (未展开该配置)")
                if cfg not in rmap:
                    print("  rust  : (未展开该配置)")
    sys.exit(1 if any_diff else 0)


if __name__ == "__main__":
    main()
