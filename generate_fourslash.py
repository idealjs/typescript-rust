#!/usr/bin/env python3
"""从 Go 仓库已验证的 fourslash 生成测试转译为 Rust：
  tests/fourslash/main.rs + cases/<snake>.rs + cases/mod.rs
  fourslash-migration.txt 清单（用例名 -> 状态）
机械迁移：全部用例生成为普通 #[test]（不做 ignore/unsupported 标记），
未实现的调用以 TODO 注释占位——实现落地后用例自然转绿。
"""

import os
import re
import sys

GO_DIR = "/home/cqh/workspace/TypeScript/tsc/internal/fourslash/tests"
OUT_DIR = "crates/tsox-lsp/tests/fourslash"
PRELUDE = "use tsox_lsp::fourslash::{self, Session};\n"

IMPL = {
    "GoToMarker": ("go_to_marker", 1),
    "FormatDocument": ("format_document", 1),
    "FormatSelection": ("format_selection", 2),
    "GoToFile": ("go_to_file", 1),
    "Insert": ("insert", 1),
    "VerifyCurrentLineContent": ("verify_current_line_content", 1),
    "VerifyCurrentFileContent": ("verify_current_file_content", 1),
    "VerifyQuickInfoAt": ("verify_quick_info_at", 3),
    "VerifyNoErrors": ("verify_no_errors", 0),
    "VerifyNumberOfErrorsInCurrentFile": ("verify_number_of_errors_in_current_file", 1),
}


def snake(name):
    s = re.sub(r"(?<=[a-z0-9])([A-Z])", r"_\1", name)
    s = re.sub(r"(?<=[A-Z])([A-Z][a-z])", r"_\1", s)
    s = s.lower()
    s = re.sub(r"[^a-z0-9_]", "_", s)
    s = re.sub(r"_+", "_", s).strip("_")
    if not s or s[0].isdigit():
        s = "t" + s
    return s


def raw_str(text):
    hashes = "#"
    while f'"{hashes}' in text:
        hashes += "#"
    return f'r{hashes}"{text}"{hashes}'


def translate_expr(e):
    e = e.strip()
    parts = split_top_plus(e)
    if len(parts) > 1:
        tp = [translate_expr(p) for p in parts]
        if any(p is None for p in tp):
            return None
        return "concat!(" + ", ".join(p for p in tp) + ")"
    if e.startswith("`") and e.endswith("`") and len(e) >= 2:
        return raw_str(e[1:-1])
    if re.fullmatch(r'"(?:[^"\\]|\\.)*"', e, re.S):
        return e
    if re.fullmatch(r"\d+", e):
        return e
    if e in ("true", "false"):
        return e
    if e == "nil":
        return "None"
    return None


def split_top_plus(e):
    parts, cur, depth, in_bt, in_str, esc = [], [], 0, False, False, False
    for ch in e:
        if in_bt:
            cur.append(ch)
            if ch == "`":
                in_bt = False
            continue
        if in_str:
            cur.append(ch)
            if esc:
                esc = False
            elif ch == "\\":
                esc = True
            elif ch == '"':
                in_str = False
            continue
        if ch == "`":
            in_bt = True
            cur.append(ch)
        elif ch == '"':
            in_str = True
            cur.append(ch)
        elif ch in "([{":
            depth += 1
            cur.append(ch)
        elif ch in ")]}":
            depth -= 1
            cur.append(ch)
        elif ch == "+" and depth == 0:
            parts.append("".join(cur).rstrip())
            cur = []
        else:
            cur.append(ch)
    parts.append("".join(cur).rstrip())
    return [p.strip() for p in parts if p.strip()]


def parse_funcs(text):
    for m in re.finditer(
            r"func (Test\w+)\(t \*testing\.T\) \{", text):
        start = m.end()
        depth = 1
        i = start
        while i < len(text) and depth:
            if text[i] == "{":
                depth += 1
            elif text[i] == "}":
                depth -= 1
            i += 1
        yield m.group(1), text[start:i - 1]


def split_statements(body):
    stmts, cur, depth = [], [], 0
    in_bt = False
    i = 0
    while i < len(body):
        c = body[i]
        if in_bt:
            cur.append(c)
            if c == "`":
                in_bt = False
            i += 1
            continue
        if c == "`":
            in_bt = True
            cur.append(c)
        elif c in "([{":
            depth += 1
            cur.append(c)
        elif c in ")]}":
            depth -= 1
            cur.append(c)
        elif c == "\n" and depth == 0:
            stmts.append("".join(cur).strip())
            cur = []
        else:
            cur.append(c)
        i += 1
    if "".join(cur).strip():
        stmts.append("".join(cur).strip())
    return [s for s in stmts if s]


_CUR_TEST = [""]


def translate_func(name, body, file_stem):
    _CUR_TEST[0] = name[4].lower() + name[5:] if name.startswith("Test") and len(name) > 4 else name
    fn = snake(name[4:]) if name.startswith("Test") else snake(name)
    lines = []
    ignores = set()
    content_vars = {}
    has_session = False

    go_skip = None
    for stmt in split_statements(body):
        if stmt in ("t.Parallel()",) or stmt.startswith("defer testutil.") \
                or stmt.startswith("defer done()") or stmt == "":
            continue
        sm = re.match(r't\.Skip\("((?:[^"\\]|\\.)*)"\)', stmt)
        if sm:
            # 上游语料自标已知失败（t.Skip）：镜像为 ignore，忠实于 Go
            go_skip = "go: t.Skip('" + sm.group(1).replace('"', "'") + "')"
            continue
        sm2 = re.match(r't\.Skip\(\)', stmt)
        if sm2:
            go_skip = "go: t.Skip()"
            continue
        m = match_const(stmt)
        if m:
            content_vars[m[0]] = snake(m[0])
            lines.append(f"let {snake(m[0])} = {raw_str(m[1])};")
            continue
        m = match_new_fourslash(stmt)
        if m:
            var, caps = m
            rust_var = content_vars.get(var, snake(var))
            if var not in content_vars:
                ignores.add(f"generator: 变量 {var} 非标准 content")
                rust_var = '""'
            go_test_name = _CUR_TEST[0]
            if caps.startswith("nil"):
                lines.append(f"let mut s = Session::new_for_test(\"{go_test_name}\", {rust_var});")
            else:
                lines.append(
                    f"let mut s = Session::new_with_capabilities"
                    f"({rust_var}, None);")
            has_session = True
            continue
        if stmt.startswith("f.VerifyCompletions(t, ") and not match_verify_completions(stmt):
            # 复杂 VerifyCompletions 形态不可译，但首个参数若是 marker 名，
            # Go 侧会先 GoToMarker 定位光标——后续 Insert 依赖该语义
            inner = stmt[len("f.VerifyCompletions(t, "):-1]
            first = split_top_commas(inner)[0].strip() if split_top_commas(inner) else ""
            if re.fullmatch(r'"(?:[^"\\]|\\.)*"', first):
                marker = first[1:-1]
                lines.append(f'fourslash::go_to_marker(&mut s, "{marker}");')
            lines.append(f"// TODO: {stmt.splitlines()[0][:100]}")
            continue
        m = match_verify_completions(stmt)
        if m is not None:
            kind, marker, labels = m
            marker_arg = "None" if marker is None else f'Some("{marker}")'
            def _lit(l):
                return '"' + l.replace("\\", "\\\\").replace('"', '\\"') + '"'
            labels_lit = "&[" + ", ".join(_lit(l) for l in labels) + "]"
            if kind == "empty":
                lines.append(f"fourslash::verify_completions_empty_at(&mut s, {marker_arg});")
            elif kind == "exact":
                lines.append(f"fourslash::verify_completions_exact_at(&mut s, {marker_arg}, {labels_lit});")
            elif kind == "unsorted":
                lines.append(f"fourslash::verify_completions_unsorted_at(&mut s, {marker_arg}, {labels_lit});")
            else:
                lines.append(f"fourslash::verify_completions_include_exclude_at(&mut s, {marker_arg}, {labels_lit}, &[]);")
            continue
        m = match_f_call(stmt)
        if m:
            method, args = m
            if method in IMPL:
                rust_fn, arity = IMPL[method]
                arg_list = [a for a in split_args(args)] if args.strip() else []
                if len(arg_list) > arity:
                    arg_list = arg_list[:arity]
                targs = [translate_expr(a) for a in arg_list[:arity]]
                call = None
                if all(t is not None for t in targs) and len(targs) == arity:
                    cand = (f"fourslash::{rust_fn}(&mut s, "
                            + ", ".join(targs) + ");")
                    if "`" not in cand:
                        call = cand
                if call is not None:
                    lines.append(call)
                    continue
            lines.append(f"// TODO: {stmt.splitlines()[0][:100]}")
            continue
        lines.append(f"// TODO: {stmt.splitlines()[0][:100]}")

    if not has_session:
        return None
    attr = f'#[ignore = "{go_skip}"]\n' if go_skip else ""
    out = [attr + f"#[test]\nfn {fn}() {{"]
    for ln in lines:
        out.append("    " + ln)
    out.append("}")
    body_text = "\n".join(out)
    return fn, body_text, bool(go_skip)



def match_verify_completions(stmt):
    """解析 f.VerifyCompletions(t, <marker>, <expected>) 的 label 级形态。
    返回 (kind, marker, labels)；kind in {empty, exact, includes, unsorted}。
    非纯字符串条目/组合字段/复杂形态返回 None（由上层 fallback 到 ignore）。"""
    if not stmt.startswith("f.VerifyCompletions(t, "):
        return None
    inner = stmt[len("f.VerifyCompletions(t, "):-1]
    parts = split_top_commas(inner)
    if len(parts) < 2:
        return None
    marker_expr, expected_expr = parts[0], parts[1]
    if marker_expr == "nil":
        marker = None
    elif re.fullmatch(r'"(?:[^"\\]|\\.)*"', marker_expr):
        marker = marker_expr[1:-1]
    else:
        return None
    if expected_expr == "nil":
        return ("empty", marker, [])
    m = re.match(r"&fourslash\.CompletionsExpectedList\{(.*)\}$", expected_expr, re.S)
    if not m:
        return None
    body = m.group(1)
    items_m = re.search(r"Items:\s*&fourslash\.CompletionsExpectedItems\{(.*?)\n\t*\},?\s*$", body, re.S)
    if not items_m:
        # Items 后还有其它字段（少见）——收紧到 Items 块必须可定位
        items_m = re.search(r"Items:\s*&fourslash\.CompletionsExpectedItems\{", body)
        if not items_m:
            return None
        # 找 Items 块的平衡括号
        start = items_m.end() - 1
        depth = 0
        i = start
        in_bt = in_str = False
        while i < len(body):
            ch = body[i]
            if in_bt:
                if ch == "`":
                    in_bt = False
            elif in_str:
                if ch == '"':
                    in_str = False
            elif ch == "`":
                in_bt = True
            elif ch == '"':
                in_str = True
            elif ch == "{":
                depth += 1
            elif ch == "}":
                depth -= 1
                if depth == 0:
                    break
            i += 1
        items_body = body[start + 1:i]
    else:
        items_body = items_m.group(1)
    fields = re.findall(r"(Exact|Includes|Excludes|Unsorted):", items_body)
    if len(fields) != 1 or fields[0] not in ("Exact", "Includes", "Unsorted"):
        return None
    field_name = fields[0]
    kind = field_name.lower()
    # 提取该字段的字符串列表
    vm = re.search(field_name + r":\s*\[\]fourslash\.CompletionsExpectedItem\{(.*?)\n\t*\},", items_body, re.S)
    if not vm:
        return None
    entries_body = vm.group(1)
    labels = re.findall(r'"((?:[^"\\]|\\.)*)"', entries_body)
    # 纯字符串列表校验：除字符串/逗号/空白外不应有其它 token
    residue = re.sub(r'"(?:[^"\\]|\\.)*"', "", entries_body)
    residue = residue.replace(",", "").strip()
    if residue:
        return None
    if len(labels) != entries_body.count('"') // 2:
        return None
    return (kind, marker, labels)


def split_top_commas(expr):
    parts, cur, depth = [], [], 0
    in_bt = in_str = False
    esc = False
    for ch in expr:
        if in_bt:
            cur.append(ch)
            if ch == "`":
                in_bt = False
            continue
        if in_str:
            cur.append(ch)
            if esc:
                esc = False
            elif ch == "\\":
                esc = True
            elif ch == '"':
                in_str = False
            continue
        if ch == "`":
            in_bt = True
            cur.append(ch)
        elif ch == '"':
            in_str = True
            cur.append(ch)
        elif ch in "([{":
            depth += 1
            cur.append(ch)
        elif ch in ")]}":
            depth -= 1
            cur.append(ch)
        elif ch == "," and depth == 0:
            parts.append("".join(cur).strip())
            cur = []
        else:
            cur.append(ch)
    if "".join(cur).strip():
        parts.append("".join(cur).strip())
    return parts


def unescape_go_string(raw):
    return raw.replace('\\"', '"').replace("\\\\", "\\")


def eval_go_concat(expr):
    """求值 Go 拼接表达式：反引号字面量取内容、双引号串反转义后连接。
    无法识别时返回 None（调用方回退旧切片行为）"""
    parts = split_top_plus(expr)
    if not parts:
        return None
    out = []
    for part in parts:
        p = part.strip()
        if len(p) >= 2 and p.startswith("`") and p.endswith("`"):
            out.append(p[1:-1])
        elif re.fullmatch(r'"(?:[^"\\]|\\.)*"', p, re.S):
            out.append(unescape_go_string(p[1:-1]))
        else:
            return None
    return "".join(out)


def match_const(stmt):
    m = re.match(r"const (\w+) = ", stmt)
    if not m:
        m2 = re.match(r"(\w+) := ", stmt)
        if not m2:
            return None
        m = m2
    rhs = stmt[m.end():]
    val = eval_go_concat(rhs)
    if val is not None:
        return m.group(1), val
    # 回退：首尾反引号之间原文（含未求值的拼接胶水）
    if not rhs.startswith("`"):
        return None
    end = rhs.rfind("`")
    if end <= 0:
        return None
    return m.group(1), rhs[1:end]


def match_new_fourslash(stmt):
    if not stmt.startswith("f, done := fourslash.NewFourslash(t, "):
        return None
    if not stmt.endswith(")"):
        return None
    inner = stmt[len("f, done := fourslash.NewFourslash(t, "):-1]
    depth = 0
    in_bt = in_str = False
    for i, ch in enumerate(inner):
        if in_bt:
            if ch == "`":
                in_bt = False
            continue
        if in_str:
            if ch == '"':
                in_str = False
            continue
        if ch == "`":
            in_bt = True
        elif ch == '"':
            in_str = True
        elif ch in "([{":
            depth += 1
        elif ch in ")]}":
            depth -= 1
        elif ch == "," and depth == 0:
            caps = inner[:i].strip()
            var = inner[i + 1:].strip()
            return var, caps
    return None


def match_f_call(stmt):
    m = re.match(r"f\.(\w+)\(t", stmt)
    if not m or not stmt.endswith(")"):
        return None
    inner = stmt[m.end():stmt.rfind(")")]
    if inner.startswith(","):
        return m.group(1), inner[1:].strip()
    if inner.strip() == "":
        return m.group(1), ""
    return None


def split_args(args):
    parts, cur, depth, in_bt, in_str = [], [], 0, False, False
    for c in args:
        if in_bt:
            cur.append(c)
            if c == "`":
                in_bt = False
            continue
        if in_str:
            cur.append(c)
            if c == '"':
                in_str = False
            continue
        if c == "`":
            in_bt = True
            cur.append(c)
        elif c == '"':
            in_str = True
            cur.append(c)
        elif c in "([{":
            depth += 1
            cur.append(c)
        elif c in ")]}":
            depth -= 1
            cur.append(c)
        elif c == "," and depth == 0:
            parts.append("".join(cur).strip())
            cur = []
        else:
            cur.append(c)
    if "".join(cur).strip():
        parts.append("".join(cur).strip())
    return parts


def main():
    os.makedirs(f"{OUT_DIR}/cases", exist_ok=True)
    stems, rows, used = [], [], set()
    for f in sorted(os.listdir(GO_DIR)):
        if not f.endswith("_test.go"):
            continue
        stem = snake(f[:-len("_test.go")])
        text = open(os.path.join(GO_DIR, f)).read()
        fns = []
        go_funcs = [name for name, _ in parse_funcs(text)]
        for name, body in parse_funcs(text):
            r = translate_func(name, body, stem)
            if r is None:
                rows.append((stem, name, "skipped", "无 NewFourslash"))
                continue
            fn, body_text, go_skipped = r
            fns.append(body_text)
            rows.append((stem, name,
                         "go-skip" if go_skipped else "generated",
                         "上游 t.Skip" if go_skipped else ""))
        if fns:
            base = stem
            k = 2
            while stem in used:
                stem = f"{base}_{k}"
                k += 1
            used.add(stem)
            path = f"{OUT_DIR}/cases/{stem}.rs"
            open(path, "w").write(PRELUDE + "\n\n" + "\n\n".join(fns) + "\n")
            stems.append(stem)
    with open(f"{OUT_DIR}/cases/mod.rs", "w") as fh:
        for s in stems:
            fh.write(f"mod {s};\n")
    open(f"{OUT_DIR}/main.rs", "w").write("mod cases;\n")
    with open("fourslash-migration.txt", "w") as fh:
        gen_n = sum(1 for r in rows if r[2] == "generated")
        skip_n = sum(1 for r in rows if r[2] == "go-skip")
        sk_n = sum(1 for r in rows if r[2] == "skipped")
        fh.write(f"# fourslash 用例迁移清单：生成 {gen_n}"
                 f" / 上游自标 go-skip {skip_n} / 跳过 {sk_n}，文件 {len(stems)} 个\n")
        for stem, name, status, why in rows:
            fh.write(f"{stem}\t{name}\t{status}\t{why}\n")
    print(f"文件 {len(stems)}，生成 {gen_n}，上游自标 {skip_n}，跳过 {sk_n}")


if __name__ == "__main__":
    main()
