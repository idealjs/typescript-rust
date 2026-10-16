#!/usr/bin/env python3
"""Rust 全函数插桩：在 crates/tsox*/src 生产代码的每个函数体首行插入
`<core>::fntrace::enter("<函数名>")`，并给 fntrace 模块接线。

规则：
- 匹配函数定义行（可选 pub/pub(crate)/async/unsafe/extern，fn 关键字 + 标识符 + ( 或 <）
- 从函数行向后找第一个 `{` 作为函数体开头，在其后插入 enter 行
- 跳过：const fn、trait 内无体声明（无 `{` 自然不插）、tests/ 目录、宏模板（$ 开头标识符不匹配）
- 幂等：目标行已是 enter 调用则跳过
- enter 路径自动探测：文件所属 crate 到 tsox-core 的可见路径（同 crate 用 crate::fntrace，
  其他 tsox* crate 用 tsox_core::fntrace；tsox-core 内用 crate::fntrace）

用法：instrument_rust.py <repo_root> [--dry-run]
"""
import os
import re
import sys

FN_LINE = re.compile(
    r"^(\s*)((?:pub(?:\([a-z ]+\))?\s+)?(?:const\s+)?(?:async\s+)?(?:unsafe\s+)?"
    r"(?:extern\s+\"[^\"]+\"\s+)?)?fn\s+([a-zA-Z_][a-zA-Z0-9_]*)\s*[(<]"
)
SKIP_DIRS = {"tests", "examples", "benches", "target"}
ENTER_RE = re.compile(r"fntrace::enter\(")


def find_body_brace(lines, start_idx):
    """从函数签名行起找第一个 {；遇到 ; 先出现（trait 声明/无体）返回 None。"""
    for i in range(start_idx, min(start_idx + 12, len(lines))):
        line = lines[i]
        for ch in line:
            if ch == "{":
                return i
            if ch == ";":
                return None
        # 单行内签名与体同行时上面循环已命中；否则继续下一行
    return None


def instrument_file(path, crate_name):
    with open(path, encoding="utf-8") as f:
        lines = f.read().split("\n")
    if any("fntrace::enter" in l for l in lines):
        return 0
    if crate_name == "tsox-core":
        call = "crate::fntrace::enter"
    else:
        call = "::tsox_core::fntrace::enter"
    changed = 0
    out = []
    i = 0
    while i < len(lines):
        line = lines[i]
        m = FN_LINE.match(line)
        insert_at = None
        if m and "const fn" not in line:
            body = find_body_brace(lines, i)
            if body is not None and body < i + 12:
                insert_at = body
                name = m.group(3)
        if insert_at is None:
            out.append(line)
            i += 1
            continue
        # 统一行内插入：{ 后插 enter，兼容空体 {}、{ 后带内容、跨行体三种形态
        while i < insert_at:
            out.append(lines[i])
            i += 1
        body_line = lines[i]
        brace_pos = body_line.index("{")
        out.append(f'{body_line[:brace_pos + 1]} {call}("{name}"); {body_line[brace_pos + 1:]}')
        changed += 1
        i += 1
    if changed:
        with open(path, "w", encoding="utf-8") as f:
            f.write("\n".join(out))
    return changed


def wire(root):
    """落地接线：fntrace 模块入 tsox-core、lib.rs 声明、corpus worker init/flush、
    runner spawn per-case trace 路径。幂等。"""
    import shutil
    core_src = os.path.join(root, "crates", "tsox-core", "src")
    dst = os.path.join(core_src, "fntrace.rs")
    if not os.path.exists(dst):
        shutil.copyfile(os.path.join(os.path.dirname(os.path.abspath(__file__)), "fntrace.rs"), dst)
        print("fntrace.rs -> tsox-core/src/")
    lib = os.path.join(core_src, "lib.rs")
    with open(lib, encoding="utf-8") as f:
        libtxt = f.read()
    if "pub mod fntrace" not in libtxt:
        lines = libtxt.split("\n")
        lines.insert(1, "pub mod fntrace;")
        with open(lib, "w", encoding="utf-8") as f:
            f.write("\n".join(lines))
        print("lib.rs: +pub mod fntrace")
    runner = os.path.join(root, "crates", "tsox", "tests", "corpus", "submodule_compiler.rs")
    with open(runner, encoding="utf-8") as f:
        txt = f.read()
    patched = []
    anchor_out = '    let out_path = std::env::temp_dir().join(format!("tsox_submodule_{idx}_{stem}.out"));'
    if "TSOX_FN_TRACE_DIR" not in txt:
        if anchor_out in txt:
            txt = txt.replace(
                anchor_out,
                anchor_out
                + '\n    let fn_trace_path = std::env::var_os("TSOX_FN_TRACE_DIR")\n'
                + '        .map(|d| std::path::Path::new(&d).join(format!("{stem}.txt")).to_string_lossy().into_owned())\n'
                + '        .unwrap_or_default();',
                1,
            )
            patched.append("trace-path")
        anchor_env = '        .env("TSOX_SUBMODULE_OUT", &out_path)'
        if anchor_env in txt:
            txt = txt.replace(
                anchor_env,
                anchor_env + '\n        .env("TSOX_FN_TRACE", &fn_trace_path)',
                1,
            )
            patched.append("spawn-env")
        anchor_worker = "        let case_path = case_path.clone();"
        if anchor_worker in txt:
            txt = txt.replace(
                anchor_worker,
                "        tsox_core::fntrace::maybe_init();\n" + anchor_worker,
                1,
            )
            patched.append("worker-init")
        anchor_write = "        let _ = std::fs::write(&out_path, payload);"
        if anchor_write in txt:
            txt = txt.replace(
                anchor_write,
                "        if let Ok(p) = std::env::var(\"TSOX_FN_TRACE\") {\n"
                "            if !p.is_empty() {\n"
                "                tsox_core::fntrace::flush_to(&p);\n"
                "            }\n"
                "        }\n" + anchor_write,
                1,
            )
            patched.append("worker-flush")
        with open(runner, "w", encoding="utf-8") as f:
            f.write(txt)
    print(f"runner patch: {patched if patched else 'skip/exists'}")


def main():
    root = sys.argv[1]
    dry = "--dry-run" in sys.argv
    if "--wire" in sys.argv:
        wire(root)
        return
    total, files = 0, 0
    for crate in sorted(os.listdir(os.path.join(root, "crates"))):
        src = os.path.join(root, "crates", crate, "src")
        if not os.path.isdir(src):
            continue
        for dirpath, dirnames, filenames in os.walk(src):
            dirnames[:] = [d for d in dirnames if d not in SKIP_DIRS]
            for fn in filenames:
                if not fn.endswith(".rs") or fn == "fntrace.rs":
                    continue
                path = os.path.join(dirpath, fn)
                if dry:
                    with open(path, encoding="utf-8") as f:
                        n = sum(1 for l in f if FN_LINE.match(l))
                    total += n
                    files += 1 if n else 0
                else:
                    n = instrument_file(path, crate)
                    total += n
                    files += 1 if n else 0
    mode = "dry-run 估计" if dry else "已插桩"
    print(f"{mode}：{files} 个文件，{total} 个函数")


if __name__ == "__main__":
    main()
