#!/usr/bin/env python3
"""Go/Rust 函数名抓取与命名归一匹配,产出单表 func_alignment.csv(靠齐追踪表)。

一张表左右对照:每行 = 一对已匹配函数(两侧字段同row)或单侧函数(go_only /
rust_only)。行按归一名 norm_name 排序,同名/近名两侧自然相邻,便于人工对照。
Go camelCase 与 Rust snake_case 的命名差异由本脚本归一处理(norm_name 列即
转换结果),纯静态抓取与名称匹配,不做语义判断。

用法:
  python3 tools/gen_func_alignment.py                 # 生成/刷新 CSV(保留已标记行)
  python3 tools/gen_func_alignment.py --stats         # 仅打印统计
  python3 tools/gen_func_alignment.py --mark --go inferFromObjectTypes,reorderCandidates \
      --status yes --round w29 --note "w29 逐行移植"
  python3 tools/gen_func_alignment.py --mark --rust infer_from_object_types \
      --status partial --round w28

CSV 列:
  norm_name     归一名(Go camelCase→snake_case,连续单字母段合并为缩写),
                排序键,对照锚点
  go_func       Go 函数名(camelCase);rust_only 行为空
  go_recv       Go receiver 类型(如 Checker),无则空
  go_file       Go 文件(不带行号,避免代码行移动造成 diff 噪声)
  rust_func     Rust 函数名(snake_case);go_only 行为空
  rust_file     Rust 文件,单个(一个函数行只对应一个文件,见文件对齐规则)
  rust_impl     Rust 所属 impl 类型(如 Checker),自由函数为空
  match_type    exact | suffix_variant | fuzzy | go_only | rust_only
  score         匹配置信度(exact/suffix=1.0,fuzzy=difflib 比值,单侧=0)
  aligned       靠齐标记:yes(已靠齐) / partial(部分) / no(确认有分歧);空=未标记
  marked_round  标记轮次(如 w29)
  note          备注

匹配规则(命名差异处理):
  1. Go 名 camelToSnake 归一(含连续大写缩写段合并:getUMDType→get_umd_type),
     与 Rust 函数名精确相等 → exact
  2. Rust 名 = 归一名 + 常见后缀(_ex/_worker/_inner/_impl 等) → suffix_variant
  3. 剩余未匹配间做 difflib 模糊匹配(阈值 0.84,同 token 预过滤) → fuzzy
  4. 文件对齐:名字命中后,在同名 Rust 站点中选出唯一文件(见下);
     无任何站点过线 → go_only
  5. 未被任何 Go 行认领的 Rust 站点 → rust_only(同名多文件的,剩余文件
     以 ";" 连接在同一行)

文件对齐(一个函数行对应一个文件):
  文件名均归一为去下划线的连贯 stem(owner_cache.rs ↔ ownercache.go 同为
  "ownercache"),按等级打分选站点:
    tier 4  stem 精确相等;或 rust stem = go 路径目录 token + go stem 的
            拼接(autoimport_util.rs ↔ autoimport/util.go,
            ata_ata.rs ↔ ata/ata.go)
    tier 3  stem 互为子串(任一方向,较短侧 ≥4 字符;如 checker.go ⊂
            checker_types.rs,discovertypings.go ⊂ ata_discover_typings.rs)
    tier 2  stem difflib 比值 ≥0.85
    tier 1  比值 ≥0.70
    tier 0  无文件相似性
  加分项:路径目录 token 交集(每命中 +2,封顶 2);go_recv 与 rust_impl
  归一相等(+6,impl 内多站点时用于选中具体 impl)。
  过线条件:tier ≥3;或 recv 与 impl 相等(tier 0 也接受);或 exact 名字
  命中且 rust 文件以该函数名单独命名(≥8 字符,如
  parse_command_line_worker.rs ↔ parseCommandLineWorker)。
"""
import argparse
import csv
import re
import sys
from collections import defaultdict
from difflib import SequenceMatcher
from pathlib import Path

GO_ROOT = Path("/home/cqh/workspace/typescript-go/tsc")
RUST_ROOT = Path("/home/cqh/workspace/ts2rust-port/crates")
OUT_CSV = Path("/home/cqh/workspace/ts2rust-port/func_alignment.csv")

GO_FUNC_RE = re.compile(
    r"^func\s+(?:\([^)]*\)\s*)?([A-Za-z_]\w*)(?:\[[^\]]*\])?\s*\(")
GO_RECV_RE = re.compile(r"^func\s+\(\w+\s+\*?(\w+)")
RUST_FN_RE = re.compile(
    r"^\s*(?:pub(?:\([^)]*\))?\s+)?(?:const\s+)?(?:unsafe\s+)?(?:async\s+)?"
    r"(?:extern\s+\"[^\"]*\"\s+)?fn\s+([a-z0-9_]+)")
RUST_IMPL_RE = re.compile(r"^\s*impl(?:<[^>]*>)?\s+(?:\S+\s+for\s+)?([A-Za-z0-9_:]+)")

SUFFIXES = ("_ex", "_worker", "_inner", "_impl", "_native", "_with_mapper",
            "_with_args", "_on_node", "_of_node", "_for_node", "_2", "_3")
MATCH_ORDER = {"fuzzy": 0, "go_only": 1, "rust_only": 2, "suffix_variant": 3, "exact": 4}


def camel_to_snake(name: str) -> str:
    s = re.sub(r"(.)([A-Z][a-z]+)", r"\1_\2", name)
    s = re.sub(r"([a-z0-9])([A-Z])", r"\1_\2", s)
    snake = s.lower().replace("__", "_")
    return collapse_initialisms(snake)


def collapse_initialisms(snake: str) -> str:
    parts = snake.split("_")
    out, buf = [], []
    for p in parts:
        if len(p) == 1 and p.isalpha():
            buf.append(p)
            continue
        if buf:
            out.append("".join(buf))
            buf = []
        out.append(p)
    if buf:
        out.append("".join(buf))
    return "_".join(out)


def concat_norm(name: str) -> str:
    return camel_to_snake(name).replace("_", "")


def path_parts(rel: str):
    parts = rel.split("/")
    stem = concat_norm(parts[-1].rsplit(".", 1)[0])
    dirs = {concat_norm(p) for p in parts[:-1]}
    return stem, dirs


_AFFINITY_CACHE = {}


def file_affinity(go_file: str, rust_file: str):
    key = (go_file, rust_file)
    if key not in _AFFINITY_CACHE:
        gs, gd = path_parts(go_file)
        rs, rd = path_parts(rust_file)
        if gs == rs or any(rs == d + gs for d in gd if len(d) >= 3):
            tier = 4
        elif min(len(gs), len(rs)) >= 4 and (gs in rs or rs in gs):
            tier = 3
        else:
            ratio = SequenceMatcher(None, gs, rs).ratio()
            tier = 2 if ratio >= 0.85 else (1 if ratio >= 0.7 else 0)
        _AFFINITY_CACHE[key] = (tier, len(gd & rd))
    return _AFFINITY_CACHE[key]


def choose_site(row, sites):
    recv_n = concat_norm(row["go_recv"]) if row["go_recv"] else ""
    best_key, best = None, None
    for idx, (f, _, im) in enumerate(sites):
        tier, overlap = file_affinity(row["go_file"], f)
        recv_hit = 1 if recv_n and recv_n == concat_norm(im) else 0
        key = (tier * 10 + min(overlap, 2) * 2 + recv_hit * 6, -idx)
        if best_key is None or key > best_key:
            best_key, best = key, (tier, recv_hit, f, im)
    return best


def scan_go():
    rows = []
    for p in sorted(GO_ROOT.rglob("*.go")):
        if p.name.endswith("_test.go") or "_generated" in p.name:
            continue
        rel = str(p.relative_to(GO_ROOT.parent))
        try:
            lines = p.read_text(encoding="utf-8", errors="replace").splitlines()
        except OSError:
            continue
        for i, line in enumerate(lines, 1):
            m = GO_FUNC_RE.match(line)
            if not m:
                continue
            recv = ""
            rm = GO_RECV_RE.match(line)
            if rm:
                recv = rm.group(1)
            rows.append({"go_func": m.group(1), "go_recv": recv,
                         "go_file": rel})
    return rows


def scan_rust():
    fns = defaultdict(list)  # snake name -> [(file, line, impl)]
    for p in sorted(RUST_ROOT.rglob("*.rs")):
        sp = str(p)
        if "/tests/" in sp or "/target/" in sp or sp.endswith("_generated.rs"):
            continue
        if "/benches/" in sp or "/examples/" in sp:
            continue
        rel = str(p.relative_to(RUST_ROOT))
        try:
            lines = p.read_text(encoding="utf-8", errors="replace").splitlines()
        except OSError:
            continue
        impl = ""
        for i, line in enumerate(lines, 1):
            im = RUST_IMPL_RE.match(line)
            if im:
                impl = im.group(1)
                continue
            fm = RUST_FN_RE.match(line)
            if fm:
                fns[fm.group(1)].append((rel, i, impl))
    return fns


def tokens(name: str) -> set:
    return frozenset(t for t in name.split("_") if len(t) > 2)


def load_rename_map():
    """人工核实的改名孪生映射(Go 名 -> Rust 名),来源 twin-verified.tsv。"""
    m = {}
    tsv = OUT_CSV.parent / 'twin-verified.tsv'
    if not tsv.exists():
        return m
    for line in open(tsv, encoding='utf-8'):
        parts = line.strip().split('\t')
        if len(parts) >= 2 and parts[1]:
            m[parts[0]] = parts[1]
    return m


def build_matches(go_rows, rust_fns):
    claimed = set()
    out = []
    rename_map = load_rename_map()
    # pass 0: 人工核实的改名孪生映射
    for row in go_rows:
        norm = camel_to_snake(row["go_func"])
        row["_norm"] = norm
        rname = rename_map.get(row["go_func"])
        if rname and rname in rust_fns:
            row["match_type"], row["score"] = "exact", 1.0
            row["_rust"] = rname
            claimed.add(rname)
    # pass 1/2: exact 与 suffix_variant(按 Go 归一名查)
    by_norm = {}
    for rname in rust_fns:
        by_norm.setdefault(rname, []).append(rname)
    for row in go_rows:
        norm = camel_to_snake(row["go_func"])
        row["_norm"] = norm
        if norm in rust_fns:
            row["match_type"], row["score"] = "exact", 1.0
            row["_rust"] = norm
            claimed.add(norm)
    for row in go_rows:
        if "match_type" in row:
            continue
        norm = row["_norm"]
        hit = None
        for suf in SUFFIXES:
            if norm + suf in rust_fns:
                hit = norm + suf
                break
        if hit is None:
            for suf in ("ex", "worker", "inner", "impl"):
                if norm + "_" + suf in rust_fns:
                    hit = norm + "_" + suf
                    break
        if hit:
            row["match_type"], row["score"] = "suffix_variant", 1.0
            row["_rust"] = hit
            claimed.add(hit)
    # pass 3: fuzzy(剩余对剩余,token 预过滤 + difflib)
    rest_rust = [r for r in rust_fns if r not in claimed]
    rest_rust_tokens = {r: tokens(r) for r in rest_rust}
    candidates = [row for row in go_rows if "match_type" not in row]
    scored = []
    for row in candidates:
        norm = row["_norm"]
        nt = tokens(norm)
        best, best_r = 0.0, None
        for r in rest_rust:
            if abs(len(r) - len(norm)) > 8:
                continue
            if nt and not (nt & rest_rust_tokens[r]):
                continue
            sm = SequenceMatcher(None, norm, r, autojunk=False)
            if sm.real_quick_ratio() < 0.84 or sm.quick_ratio() < 0.84:
                continue
            ratio = sm.ratio()
            if ratio > best:
                best, best_r = ratio, r
        if best_r and best >= 0.84:
            scored.append((best, row, best_r))
    scored.sort(key=lambda x: -x[0])
    used = set()
    for score, row, r in scored:
        if r in used or "match_type" in row:
            continue
        row["match_type"], row["score"], row["_rust"] = "fuzzy", round(score, 3), r
        used.add(r)
        claimed.add(r)
    for row in go_rows:
        row.setdefault("match_type", "unmatched")
        row.setdefault("score", 0.0)
        row.setdefault("_rust", "")
    for row in go_rows:
        name = row["_rust"]
        if not name or name not in rust_fns:
            continue
        tier, recv_hit, f, im = choose_site(row, rust_fns[name])
        fn_named = (row["match_type"] == "exact"
                    and len(concat_norm(name)) >= 8
                    and concat_norm(name) == path_parts(f)[0])
        if tier >= 3 or recv_hit or fn_named:
            row["_site"] = (f, im)
            row["_tier"] = tier
        else:
            row["_rust"] = ""
    return claimed


def load_marks():
    marks = {}
    if OUT_CSV.exists():
        with open(OUT_CSV, newline="", encoding="utf-8") as f:
            for r in csv.DictReader(f):
                key = (r["go_recv"], r["go_func"])
                marks[key] = (r.get("aligned", ""), r.get("marked_round", ""),
                              r.get("note", ""))
    return marks


def generate():
    go_rows = scan_go()
    rust_fns = scan_rust()
    build_matches(go_rows, rust_fns)
    marks = load_marks()

    fields = ["norm_name", "go_func", "go_recv", "go_file", "rust_func",
              "rust_file", "rust_impl", "match_type", "score", "aligned",
              "marked_round", "note"]
    rows_out = []
    covered = set()
    tier_stats = defaultdict(int)
    for row in go_rows:
        rust_name = row.get("_rust", "")
        site = row.get("_site")
        if site:
            f, im = site
            covered.add((rust_name, f))
            tier_stats["file_exact" if row["_tier"] == 4 else
                       ("file_subset" if row["_tier"] == 3 else "recv_only")] += 1
        aligned, rnd, note = marks.get((row["go_recv"], row["go_func"]),
                                       ("", "", ""))
        rows_out.append({
            "norm_name": row["_norm"], "go_func": row["go_func"],
            "go_recv": row["go_recv"], "go_file": row["go_file"],
            "rust_func": rust_name if site else "",
            "rust_file": f if site else "",
            "rust_impl": im if site else "",
            "match_type": row["match_type"] if site else "go_only",
            "score": row["score"] if site else 0.0,
            "aligned": aligned, "marked_round": rnd, "note": note,
        })
    matched_names = {r["_rust"] for r in go_rows if r.get("_site")}
    for rname in sorted(rust_fns):
        pairs = []
        for loc in rust_fns[rname]:
            p = (loc[0], loc[2])
            if p not in pairs:
                pairs.append(p)
        if rname in matched_names:
            pairs = [p for p in pairs if (rname, p[0]) not in covered]
        if not pairs:
            continue
        rows_out.append({
            "norm_name": rname, "go_func": "", "go_recv": "", "go_file": "",
            "rust_func": rname, "rust_file": ";".join(f for f, _ in pairs),
            "rust_impl": ";".join(im for _, im in pairs),
            "match_type": "rust_only", "score": 0.0,
            "aligned": "", "marked_round": "", "note": "",
        })
    rows_out.sort(key=lambda r: (r["norm_name"], MATCH_ORDER[r["match_type"]],
                                 r["go_recv"], r["rust_impl"]))
    with open(OUT_CSV, "w", newline="", encoding="utf-8") as f:
        w = csv.DictWriter(f, fieldnames=fields)
        w.writeheader()
        w.writerows(rows_out)
    return rows_out, tier_stats


def do_mark(args):
    if not OUT_CSV.exists():
        sys.exit("func_alignment.csv 不存在,先运行生成模式")
    rows = list(csv.DictReader(open(OUT_CSV, newline="", encoding="utf-8")))
    go_names = {n.strip() for n in args.go.split(",") if n.strip()} if args.go else set()
    rust_names = {n.strip() for n in args.rust.split(",") if n.strip()} if args.rust else set()
    hit = 0
    for r in rows:
        if (r["go_func"] in go_names) or (r["rust_func"] in rust_names):
            r["aligned"] = args.status
            if args.round:
                r["marked_round"] = args.round
            if args.note:
                r["note"] = args.note
            hit += 1
    if hit:
        with open(OUT_CSV, "w", newline="", encoding="utf-8") as f:
            w = csv.DictWriter(f, fieldnames=list(rows[0].keys()))
            w.writeheader()
            w.writerows(rows)
    print(f"marked {hit} rows (status={args.status})")


def _noop():  # 占位保持结构
    pass


def print_stats(rows, tier_stats=None):
    by_mt = defaultdict(int)
    for r in rows:
        by_mt[r["match_type"]] += 1
    print(f"表行总数(Go 函数 + Rust 独有函数): {len(rows)}")
    for mt in ("exact", "suffix_variant", "fuzzy", "go_only", "rust_only"):
        print(f"  {mt:14s}: {by_mt[mt]}")
    if tier_stats:
        print("文件对齐分布(已配对行):")
        for k in ("file_exact", "file_subset", "recv_only"):
            print(f"  {k:12s}: {tier_stats[k]}")
    by_al = defaultdict(int)
    for r in rows:
        by_al[r["aligned"] or "(未标记)"] += 1
    print("靠齐标记分布:")
    for k, v in sorted(by_al.items()):
        print(f"  {k}: {v}")
    by_mod = defaultdict(int)
    for r in rows:
        if r["match_type"] == "go_only":
            parts = r["go_file"].split("/")
            mod = parts[2] if len(parts) >= 3 and parts[1] == "internal" else (parts[1] if len(parts) >= 2 else parts[0])
            by_mod[mod] += 1
    print("go_only 按 Go 模块分布(top15):")
    for k, v in sorted(by_mod.items(), key=lambda x: -x[1])[:15]:
        print(f"  {k:20s}: {v}")


def main():
    ap = argparse.ArgumentParser(description=__doc__,
                                 formatter_class=argparse.RawDescriptionHelpFormatter)
    ap.add_argument("--mark", action="store_true", help="标记模式(修改 CSV)")
    ap.add_argument("--go", help="逗号分隔的 Go 函数名")
    ap.add_argument("--rust", help="逗号分隔的 Rust 函数名")
    ap.add_argument("--status", default="yes", choices=["yes", "partial", "no"])
    ap.add_argument("--round", default="", help="标记轮次(如 w29)")
    ap.add_argument("--note", default="", help="备注")
    ap.add_argument("--stats", action="store_true", help="生成后仅打印统计")
    args = ap.parse_args()

    if args.mark:
        do_mark(args)
        return
    rows, tier_stats = generate()
    print_stats(rows, tier_stats)
    print(f"输出: {OUT_CSV}")


if __name__ == "__main__":
    main()
