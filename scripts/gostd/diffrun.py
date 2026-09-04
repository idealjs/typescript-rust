#!/usr/bin/env python3
"""Case-by-case differential runner replicating tsgo's compiler_runner.go
semantics. For every (case, config) that Go's runner would execute:
  - run tsgo and classify vs tsgo's OWN committed baseline  -> go status
  - run tsox (worker) and classify vs the same baseline     -> rust status
Outputs a full per-case table + mismatch worklist."""
import os, re, json, shutil, subprocess, csv, sys, itertools
from concurrent.futures import ProcessPoolExecutor

TS = os.path.expanduser("~/workspace/typescript-go/built/local/tsgo")
ROOT = "/home/cqh/workspace/typescript-rust/_submodules/TypeScript/tests/cases"
LIBS = "/home/cqh/workspace/typescript-rust/_submodules/TypeScript/tests/lib"
OWN_REF = os.path.expanduser("~/workspace/typescript-go/tsc/testdata/baselines/reference")
RUST_EXE = os.environ.get("RUST_EXE", "")
WORK = "/tmp/gostd/work"
OUT = "/tmp/gostd"

META = json.load(open("/tmp/gostd/decls.json"))
VARY = META["vary"]; DECLS = META["decls"]

SKIPPED = {
 "APILibCheck.ts","APISample_Watch.ts","APISample_WatchWithDefaults.ts",
 "APISample_WatchWithOwnWatchHost.ts","APISample_compile.ts","APISample_jsdoc.ts",
 "APISample_linter.ts","APISample_parseConfig.ts","APISample_transform.ts",
 "APISample_watcher.ts","preserveUnusedImports.ts",
 "noCrashWithVerbatimModuleSyntaxAndImportsNotUsedAsValues.ts",
 "verbatimModuleSyntaxCompat.ts","verbatimModuleSyntaxCompat2.ts",
 "verbatimModuleSyntaxCompat3.ts","verbatimModuleSyntaxCompat4.ts",
 "preserveValueImports.ts","preserveValueImports_importsNotUsedAsValues.ts",
 "preserveValueImports_errors.ts","preserveValueImports_mixedImports.ts",
 "preserveValueImports_module.ts","importsNotUsedAsValues_error.ts",
 "alwaysStrictNoImplicitUseStrict.ts","nonPrimitiveIndexingWithForInSupressError.ts",
 "parameterInitializerBeforeDestructuringEmit.ts",
 "mappedTypeUnionConstraintInferences.ts","lateBoundConstraintTypeChecksCorrectly.ts",
 "keyofDoesntContainSymbols.ts","isolatedModulesOut.ts","noStrictGenericChecks.ts",
 "noImplicitUseStrict_umd.ts","noImplicitUseStrict_system.ts",
 "noImplicitUseStrict_es6.ts","noImplicitUseStrict_commonjs.ts",
 "noImplicitUseStrict_amd.ts","noImplicitAnyIndexingSuppressed.ts",
 "excessPropertyErrorsSuppressed.ts","moduleNoneDynamicImport.ts",
 "moduleNoneErrors.ts","moduleNoneOutFile.ts",
 "noErrorUsingImportExportModuleAugmentationInDeclarationFile1.ts",
 "noErrorUsingImportExportModuleAugmentationInDeclarationFile2.ts",
 "noErrorUsingImportExportModuleAugmentationInDeclarationFile3.ts",
 "requireOfJsonFileWithModuleEmitNone.ts",
 "requireOfJsonFileWithModuleNodeResolutionEmitNone.ts",
}

OPTION_RE = re.compile(r"^\s*//\s*@(\w+)\s*:\s*(.*)$")
FILE_DIR_RE = re.compile(r"^\s*//\s*@filename:\s*(\S+)\s*$", re.I)
SYMLINK_RE = re.compile(r"^\s*//\s*@link\s*:\s*([^\\r\\n]*)\s*->\s*([^\\r\\n]*)")
REFERENCES_RE = re.compile(r"reference\s+path")

def strip_semi(v): return v.rstrip(";").strip() if v.strip().endswith(";") else v.strip()

def parse_case(text, fallback_name):
    """Port of ParseTestFilesAndSymlinks (compiler flavor) + extractCompilerSettings.
    Returns (units:[(name, content)], settings, symlinks, current_dir, error)."""
    units = []
    settings = {}
    cur_name = None
    cur = []           # content lines
    cur_dir = ""
    for line in text.split("\n"):
        if SYMLINK_RE.match(line):
            continue
        m = OPTION_RE.match(line)
        if m:
            name = m.group(1).lower()
            val = m.group(2).strip()
            if name == "currentdirectory":
                cur_dir = val
            if name == "filename":
                if cur_name is not None:
                    units.append((cur_name, "\n".join(cur)))
                cur_name = val.strip()
                cur = []
            else:
                settings[name] = val.rstrip(";").strip() if val.strip().endswith(";") else val.strip()
            continue
        if cur_name is not None or True:
            if cur:
                cur.append(line)
            else:
                if line.strip() == "":
                    continue      # leading blanks dropped until first content
                cur.append(line)
    # EOF: push final unit
    if cur_name is None:
        cur_name = fallback_name
    units.append((cur_name, "\n".join(cur)))
    return units, settings, cur_dir

def split_values(option, value):
    """Port of splitOptionValues (incl. `*` and exclusions)."""
    if not value: return None
    star = False; includes = []; excludes = []
    for s in value.split(","):
        s = s.strip()
        if not s: continue
        if s == "*": star = True
        elif s.startswith("-") or s.startswith("!"): excludes.append(s[1:])
        else: includes.append(s)
    if not includes and not star and not excludes: return None
    vals = []
    if star:
        d = VARY.get(option) or DECLS.get(option) or {}
        vals.extend(d.get("values") or [])
    vals.extend(includes)
    out = []
    seen = set()
    for v in vals:
        if v in excludes or v.lstrip("!-") in excludes: continue
        k = v.lower()
        if k in seen: continue
        seen.add(k); out.append(v)
    return out

def build_configs(settings):
    """Port of GetFileBasedTestConfigurations. Returns list[(name, {opt: val})]."""
    option_entries = []
    variation = 1
    nonvarying = {}
    for opt, val in settings.items():
        if opt in VARY:
            entries = split_values(opt, val)
            if entries is None: continue
            if len(entries) > 1:
                variation *= len(entries)
                if variation > 25:
                    return None, "variations>25"
                option_entries.append([opt] + entries)
            elif len(entries) == 1:
                nonvarying[opt] = entries[0]
        else:
            nonvarying[opt] = val
    if not option_entries and not nonvarying:
        nonvarying = {}   # no settings: still one default config (Go runTest else-branch)
    configs = []
    if option_entries:
        for combo in itertools.product(*[e[1:] for e in option_entries]):
            sel = {e[0]: v for e, v in zip(option_entries, combo)}
            name = ",".join(f"{k}={sel[k].lower()}" for k in sorted(sel))
            merged = dict(sel); merged.update(nonvarying)
            configs.append((name, merged))
    else:
        configs.append(("", dict(nonvarying)))
    return configs, None

CLI_PASS_THROUGH = set()  # names mappable to --name flags (filled from decls)
UNTRANSLATABLE = {"baseurl","paths","rootdirs","typesversions","currentdirectory",
                  "noimplicitreferences","runexternalcode","symlink","filename"}

def to_cli(options):
    """Translate harness options to tsgo CLI flags. Returns (flags, untranslatable)."""
    flags = []; bad = []
    for opt, val in sorted(options.items()):
        if opt in ("filename","currentdirectory","symlink","noimplicitreferences","runexternalcode"):
            continue
        if opt in UNTRANSLATABLE:
            bad.append(opt); continue
        d = DECLS.get(opt)
        if d is None:
            bad.append(opt); continue
        if d["kind"] == "List":
            flags += [f"--{d['orig']}", ",".join(x.strip() for x in val.split(",") if x.strip())]
        elif d["kind"] in ("Boolean",):
            if val.lower() in ("true","false"):
                flags += [f"--{d['orig']}", val.lower()]
            else:
                bad.append(opt)
        else:
            flags += [f"--{d['orig']}", val]
    return flags, bad

def norm(t): return t.replace("\r\n", "\n").strip()

def own_baseline(suite, configured_stem):
    p = os.path.join(OWN_REF, suite, configured_stem + ".errors.txt")
    if not os.path.exists(p): return ""
    return norm(flat(open(p, encoding="utf-8", errors="replace").read()))

def flat(t):
    i = t.find("\n==== ")
    return t[:i+1] if i >= 0 else t

def run_case(job):
    suite, rel = job
    base = os.path.basename(rel)
    stem_ext = os.path.splitext(base)
    stem, extname = stem_ext[0], stem_ext[1]
    if base in SKIPPED:
        return [("skip-case", suite, rel, "", "tsgo skippedTests", "")]
    try:
        text = open(os.path.join(ROOT, suite, rel), encoding="utf-8", errors="replace").read()
    except Exception as e:
        return [("error-case", suite, rel, "", str(e), "")]
    units, settings, cur_dir = parse_case(text, base)
    if "@symlink" in text or any(SYMLINK_RE.match(l) for l in text.split("\n")):
        return [("skip-case", suite, rel, "", "symlink case", "")]
    if not cur_dir:
        cur_dir = "/.src"
    if cur_dir not in ("/.src", "/"):
        return [("skip-case", suite, rel, "", "custom currentdirectory", "")]
    # tsconfig unit?
    ts_cfg = None
    for i, (n, c) in enumerate(units):
        if n == "tsconfig.json" or n.endswith("tsconfig.json") or n == "jsconfig.json":
            ts_cfg = (i, n, c); break
    configs, err = build_configs(settings)
    if err:
        return [("error-case", suite, rel, "", err, "")]
    if not configs:
        return []
    case_dir = os.path.join(WORK, suite, rel.replace("/", "_"))
    shutil.rmtree(case_dir, ignore_errors=True)
    os.makedirs(case_dir, exist_ok=True)
    # materialize units (directive lines already excluded by parser)
    for i, (n, c) in enumerate(units):
        if ts_cfg is not None and i == ts_cfg[0]:
            continue
        name = n.lstrip("/")
        dst = os.path.join(case_dir, name)
        os.makedirs(os.path.dirname(dst) or case_dir, exist_ok=True)
        open(dst, "w", encoding="utf-8").write(c)
    # /.lib fixtures
    for _, c in units:
        for lib in set(re.findall(r'reference path="/\.lib/([^"]+)"', c)):
            src = os.path.join(LIBS, lib)
            if os.path.exists(src):
                dst = os.path.join(case_dir, ".lib", lib)
                os.makedirs(os.path.dirname(dst), exist_ok=True)
                shutil.copy(src, dst)
    # entry file selection (root rule)
    last_content = units[-1][1] if units else ""
    if ts_cfg is not None:
        entries = None        # -p mode
    else:
        if 'require' in last_content or REFERENCES_RE.search(last_content):
            last_name = units[-1][0].lstrip("/")
            entries = [last_name]
        else:
            entries = [n.lstrip("/") for i,(n,_) in enumerate(units) if not (ts_cfg is not None and i == ts_cfg[0])]
    out_rows = []
    for cfg_name, options in configs:
        configured = f"{stem}({cfg_name}){extname}" if cfg_name else f"{stem}{extname}"
        expect = own_baseline(suite, configured)
        flags, bad = to_cli(options)
        if bad:
            out_rows.append((suite, rel, cfg_name, "skip-config", f"untranslatable: {','.join(bad)}", ""))
            continue
        if ts_cfg is not None:
            args = [TS, "--noEmit", "-p", "."] if not flags else [TS, "--noEmit"] + flags + ["-p", "."]
            note = "p-mode" + ("+overlay-approx" if flags else "")
        else:
            args = [TS, "--noEmit"] + flags + entries
            note = ""
        try:
            p = subprocess.run(args, cwd=case_dir, capture_output=True, timeout=60,
                               text=True, errors="replace")
            out = p.stdout + p.stderr
        except subprocess.TimeoutExpired:
            out_rows.append((suite, rel, cfg_name, "go-timeout", note, ""))
            continue
        actual_lines = []
        for l in out.splitlines():
            l = l.replace(case_dir + "/", "").replace(case_dir, "").lstrip("./")
            if re.search(r'\(\d+,\d+\): \w+ TS\d+:', l) or l.startswith("  ") or l.startswith("!!!"):
                actual_lines.append(l.rstrip())
        status = "go-pass" if norm("\n".join(actual_lines)) == expect else "go-diff"
        out_rows.append((suite, rel, cfg_name, status, note, ""))
    return out_rows

def main():
    side = sys.argv[1] if len(sys.argv) > 1 else "go"
    os.makedirs(OUT, exist_ok=True)
    shutil.rmtree(WORK, ignore_errors=True); os.makedirs(WORK, exist_ok=True)
    jobs = []
    for suite in ("compiler", "conformance"):
        for dp, _, ns in os.walk(os.path.join(ROOT, suite)):
            for n in sorted(ns):
                if n.endswith((".ts", ".tsx")):
                    jobs.append((suite, os.path.relpath(os.path.join(dp, n), os.path.join(ROOT, suite))))
    jobs = [j for j in jobs if os.path.basename(j[1]) not in SKIPPED]
    print(f"{len(jobs)} cases (go standard, skippedTests excluded)", flush=True)
    done = 0
    with open(os.path.join(OUT, f"{side}_configs.csv"), "w", newline="") as f:
        w = csv.writer(f); w.writerow(["suite","case","config","status","note","rust_status"])
        with ProcessPoolExecutor(max_workers=6) as ex:
            for rows in ex.map(run_case, jobs, chunksize=4):
                for r in rows:
                    w.writerow(r)
                done += 1
                if done % 500 == 0: f.flush(); print(done, flush=True)
    print("DONE", side, flush=True)

if __name__ == "__main__":
    main()
