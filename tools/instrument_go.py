#!/usr/bin/env python3
"""Go oracle 全函数插桩：tsc/internal 生产代码每个函数体首行插入
fntrace.Enter("<函数名>")，公共包 tsc/internal/fntrace 提供去重记录与按用例切换，
conformance runner 两处 Switch（runTest 入口 + runSingleConfigTest 的 t.Parallel()
之后回切本用例——注册期全局 cur 会被后续用例切走，不回切则编译期写串文件）。

- 函数匹配：^func (recv)? Name( 或 Name[T](；签名可跨行（窗口 40 行内找行尾 {）
- Enter 去重保序：仅首次调用写入（无去重时重型用例单例可膨胀至 GB 级）
- 跳过 _test.go / _generated.go / //go:build ignore / package main / fntrace 包自身
- 幂等；--restore 按 /tmp/instr/go_backup 备份还原（保留他人未提交改动）
用法：instrument_go.py <typescript-go-root> [--dry-run | --restore]
"""
import os
import re
import shutil
import sys

GO_ROOT_SUB = os.path.join("tsc", "internal")
FN_LINE = re.compile(
    r"^func\s+(?:\([^)]*\)\s+)?([A-Za-z_][A-Za-z0-9_]*)\s*(?:\[[^\]]*\]\s*)?\("
)
RUNNER_SIG = "func (r *CompilerBaselineRunner) runTest(t *testing.T, filename string) {"
FNTRACE_IMPORT = '"github.com/microsoft/TypeScript/tsc/internal/fntrace"'
BACKUP_DIR = "/tmp/instr/go_backup"
TOUCHED = "/tmp/instr/go_touched.txt"

FNTRACE_PKG_SRC = """package fntrace

import (
	"bufio"
	"os"
	"sync"
)

var on = func() bool { return os.Getenv("GOFN_TRACE") != "" }()
var mu sync.Mutex
var cur = ""
var seen map[string]bool
var w *bufio.Writer
var f *os.File

func flushLocked() {
	if w != nil {
		w.Flush()
		f.Close()
		w, f = nil, nil
	}
}

// Switch 切换当前用例的输出文件（runner 每用例调用；flush 并关闭前一个）。
func Switch(path string) {
	mu.Lock()
	defer mu.Unlock()
	flushLocked()
	cur = path
	seen = make(map[string]bool)
}

// Enter 去重保序：每用例仅首次调用写入一行。
func Enter(name string) {
	if !on || cur == "" {
		return
	}
	mu.Lock()
	defer mu.Unlock()
	if seen[name] {
		return
	}
	seen[name] = true
	if w == nil {
		var err error
		f, err = os.OpenFile(cur, os.O_CREATE|os.O_TRUNC|os.O_WRONLY, 0o644)
		if err != nil {
			return
		}
		w = bufio.NewWriterSize(f, 1<<16)
	}
	w.WriteString(name + "\\n")
}

func Flush() {
	mu.Lock()
	defer mu.Unlock()
	flushLocked()
}
"""


def add_import(lines):
    if any("internal/fntrace" in l for l in lines):
        return lines
    for i, line in enumerate(lines):
        if line.startswith("import ("):
            return lines[: i + 1] + ["\t" + FNTRACE_IMPORT] + lines[i + 1 :]
    for i, line in enumerate(lines):
        if line.startswith("import "):
            return lines[: i + 1] + ["import " + FNTRACE_IMPORT] + lines[i + 1 :]
    for i, line in enumerate(lines):
        if line.startswith("package "):
            return lines[: i + 1] + ["", "import " + FNTRACE_IMPORT] + lines[i + 1 :]
    return lines


def instrument_go_file(path):
    with open(path, encoding="utf-8") as f:
        out = f.read().split("\n")
    if any("fntrace.Enter(" in l for l in out):
        return 0
    inserts = []
    i = 0
    while i < len(out):
        m = FN_LINE.match(out[i])
        if m:
            j = i
            while j < len(out) and j < i + 40:
                stripped = out[j].rstrip()
                if stripped.endswith("{"):
                    inserts.append((j, m.group(1)))
                    break
                if stripped.endswith("}") or stripped.endswith(";"):
                    break
                j += 1
        i += 1
    if not inserts:
        return 0
    for j, name in reversed(inserts):
        out.insert(j + 1, f'\tfntrace.Enter("{name}");')
    out = add_import(out)
    with open(path, "w", encoding="utf-8") as f:
        f.write("\n".join(out))
    return len(inserts)


def patch_runner(root):
    """runTest 体首 + runSingleConfigTest 的 t.Parallel() 后各插一处 Switch。"""
    path = os.path.join(root, GO_ROOT_SUB, "testrunner", "compiler_runner.go")
    with open(path, encoding="utf-8") as f:
        lines = f.read().split("\n")
    if any("fntrace.Switch" in l for l in lines):
        return False
    patched = 0
    for i, line in enumerate(lines):
        if line.startswith(RUNNER_SIG):
            lines.insert(
                i + 1,
                '\tfntrace.Switch(filepath.Join(os.Getenv("GOFN_TRACE_DIR"), tspath.GetBaseFileName(filename)+".txt"))',
            )
            patched += 1
            break
    sig_idx = None
    for i, line in enumerate(lines):
        if line.startswith("func (r *CompilerBaselineRunner) runSingleConfigTest("):
            sig_idx = i
            break
    if sig_idx is not None:
        for j in range(sig_idx, min(sig_idx + 16, len(lines))):
            if lines[j].strip() == "t.Parallel()":
                lines.insert(
                    j + 1,
                    '\t\tfntrace.Switch(filepath.Join(os.Getenv("GOFN_TRACE_DIR"), tspath.GetBaseFileName(test.filename)+".txt"))',
                )
                patched += 1
                break
    lines = add_import(lines)
    if '"path/filepath"' not in "\n".join(lines):
        lines.insert(2, '\t"path/filepath"')
    with open(path, "w", encoding="utf-8") as f:
        f.write("\n".join(lines))
    return patched == 2


def backup(root, rel):
    dst = os.path.join(BACKUP_DIR, rel)
    os.makedirs(os.path.dirname(dst), exist_ok=True)
    if not os.path.exists(dst):
        shutil.copyfile(os.path.join(root, rel), dst)
        with open(TOUCHED, "a", encoding="utf-8") as f:
            f.write(rel + "\n")


def restore(root):
    if not os.path.exists(TOUCHED):
        print("无插桩记录")
        return
    n = 0
    with open(TOUCHED, encoding="utf-8") as f:
        for rel in (l.strip() for l in f if l.strip()):
            src = os.path.join(BACKUP_DIR, rel)
            if os.path.isfile(src):
                shutil.copyfile(src, os.path.join(root, rel))
                n += 1
    shutil.rmtree(os.path.join(root, GO_ROOT_SUB, "fntrace"), ignore_errors=True)
    os.remove(TOUCHED)
    print(f"已还原 {n} 个文件，fntrace 包已删")


def main():
    root = sys.argv[1]
    if "--restore" in sys.argv:
        restore(root)
        return
    dry = "--dry-run" in sys.argv
    base = os.path.join(root, GO_ROOT_SUB)
    total, pkgs = 0, set()
    for dirpath, dirnames, filenames in os.walk(base):
        if os.path.relpath(dirpath, root).endswith(os.path.join("internal", "fntrace")):
            continue
        for fn in filenames:
            if not fn.endswith(".go") or fn.endswith("_test.go") or fn.endswith("_generated.go") or fn.startswith("zz_"):
                continue
            path = os.path.join(dirpath, fn)
            with open(path, encoding="utf-8") as f:
                head = f.read(4096)
            if "//go:build ignore" in head or re.search(r"^package main\\b", head, re.M):
                continue
            if dry:
                with open(path, encoding="utf-8") as f:
                    n = sum(1 for l in f if FN_LINE.match(l))
                if n:
                    total += n
                    pkgs.add(os.path.basename(dirpath))
            else:
                with open(path, encoding="utf-8") as f:
                    already = any("fntrace.Enter(" in l for l in f)
                if not already:
                    backup(root, os.path.relpath(path, root))
                n = instrument_go_file(path)
                if n:
                    total += n
                    pkgs.add(os.path.basename(dirpath))
    if not dry:
        pkg_dir = os.path.join(root, GO_ROOT_SUB, "fntrace")
        if not os.path.exists(os.path.join(pkg_dir, "fntrace.go")):
            os.makedirs(pkg_dir, exist_ok=True)
            with open(os.path.join(pkg_dir, "fntrace.go"), "w", encoding="utf-8") as f:
                f.write(FNTRACE_PKG_SRC)
        runner_rel = os.path.relpath(os.path.join(base, "testrunner", "compiler_runner.go"), root)
        with open(os.path.join(root, runner_rel), encoding="utf-8") as f:
            runner_clean = "fntrace.Switch" not in f.read()
        ok = patch_runner(root)
        if ok and runner_clean:
            backup(root, runner_rel)
        print(f"runner patch: {'done' if ok else 'FAILED'}")
    mode = "dry-run 估计" if dry else "已插桩"
    print(f"{mode}：{len(pkgs)} 个包，{total} 个函数")


if __name__ == "__main__":
    main()
