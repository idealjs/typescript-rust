#!/usr/bin/env python3
"""按 corpus_results.csv 的 FAIL 清单切割失败信息分片到 /tmp/flywheel_shards/。

每个分片三段：参考(期望) / 本地(实际) / 机械 diff。
数据源：crates/tsox/tests/corpus/{testdata/baselines/reference, baselines/local}/compiler/。
须在全量语料跑完后执行（local baselines 需为最新）。
"""
import csv
import difflib
import os
import re

ROOT = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
REF = f"{ROOT}/crates/tsox/tests/corpus/testdata/baselines/reference/compiler"
LOC = f"{ROOT}/crates/tsox/tests/corpus/baselines/local/compiler"
OUT = "/tmp/flywheel_shards"
LOG = f"{ROOT}/fullrun.log"
STACK_DIFF = f"{ROOT}/corpus_stack_diff.csv"
EXTS = (".ts", ".tsx", ".js", ".jsx", ".mts", ".cts")
LOG_RE = re.compile(r"^\[w\d+\] #\d+/\d+ (FAIL|SKIP) (\S+) \(([\d.]+)s\)(.*)$")


def candidates(name):
    cands = [f"{name}.errors.txt"]
    stem = name
    for ext in EXTS:
        if name.endswith(ext):
            stem = name[: -len(ext)]
            break
    cands += [f"{stem}.errors.txt", f"{stem}.ts.errors.txt", f"{stem}.tsx.errors.txt"]
    seen, uniq = set(), []
    for c in cands:
        if c not in seen:
            seen.add(c)
            uniq.append(c)
    return uniq


def resolve(base_dir, name):
    for c in candidates(name):
        path = os.path.join(base_dir, c)
        if os.path.isfile(path):
            with open(path, encoding="utf-8", errors="replace") as f:
                return c, f.read()
    return None, None


def load_log_lines():
    lines = {}
    if os.path.isfile(LOG):
        with open(LOG, encoding="utf-8", errors="replace") as f:
            for line in f:
                m = LOG_RE.match(line.rstrip("\n"))
                if m and m.group(2) not in lines:
                    lines[m.group(2)] = line.rstrip("\n")
    return lines


def load_stack_diff():
    """读仓库根 corpus_stack_diff.csv（case,go_only,rust_only），无则空表。"""
    diff = {}
    if os.path.isfile(STACK_DIFF):
        with open(STACK_DIFF, encoding="utf-8") as f:
            for line in f.read().split("\n")[1:]:
                if not line.strip():
                    continue
                parts = line.split(",", 2)
                if len(parts) == 3:
                    diff[parts[0].removesuffix(".ts").removesuffix(".tsx")] = (
                        parts[1],
                        parts[2],
                    )
    return diff


def main():
    os.makedirs(OUT, exist_ok=True)
    log_lines = load_log_lines()
    stack_diff = load_stack_diff()
    with open(f"{ROOT}/corpus_results.csv", encoding="utf-8") as f:
        rows = list(csv.reader(f))[1:]
    n, missing = 0, []
    for row in rows:
        if not row or not row[0].startswith("compiler/"):
            continue
        name = row[0][len("compiler/"):]
        ref_file, ref = resolve(REF, name)
        loc_file, loc = resolve(LOC, name)
        log_line = log_lines.get(name) or log_lines.get(f"{name}.ts") or ""
        stem_key = name
        for ext in EXTS:
            if stem_key.endswith(ext):
                stem_key = stem_key[: -len(ext)]
                break
        sd = stack_diff.get(stem_key)
        parts = [
            f"# 用例 {name}",
            "",
            f"全量日志行：{log_line if log_line else '（未在 fullrun.log 找到该名行）'}",
            "",
            "## 执行栈对照（两侧全函数 trace 差集，见 corpus_stack_diff.csv）",
            f"go_only（Go 执行而 Rust 未执行）：{sd[0] if sd and sd[0] else '（无）'}",
            f"rust_only（Rust 独有执行，前 60 个）：{';'.join(sd[1].split(';')[:60]) if sd and sd[1] else '（无）'}",
            "",
            f"参考基线文件：{ref_file if ref_file else '无（Go 期望零错误，或超时/挂起类）'}",
            f"本地基线文件：{loc_file if loc_file else '无（本地无输出：挂起/超时/信号嫌疑）'}",
            "",
            "## 参考(期望)",
            ref.rstrip() if ref is not None else "（无参考基线文件）",
            "",
            "## 本地(实际)",
            loc.rstrip() if loc is not None else "（本地未产出 baseline 文件）",
            "",
            "## 机械 diff（- 参考 + 本地）",
        ]
        if ref is not None and loc is not None:
            diff = list(
                difflib.unified_diff(
                    ref.splitlines(), loc.splitlines(), fromfile="参考", tofile="本地", lineterm=""
                )
            )
            parts += diff if diff else ["（无差异——基线文本相同，差异可能在测试断言层）"]
        else:
            parts += ["（无法生成 diff：参考或本地基线缺失）"]
        safe = name.replace("/", "_")
        with open(os.path.join(OUT, f"{safe}.md"), "w", encoding="utf-8") as f:
            f.write("\n".join(parts) + "\n")
        n += 1
        if ref is None and loc is None:
            missing.append(name)
    print(f"分片完成：{n} 例 -> {OUT}")
    if missing:
        print(f"双侧基线均缺失 {len(missing)} 例（挂起/超时嫌疑，需主 agent 关注）：")
        print("  " + ", ".join(missing))


if __name__ == "__main__":
    main()
