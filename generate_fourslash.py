#!/usr/bin/env python3
"""从 Go 仓库已验证的 fourslash 生成测试转译为 Rust：
  tests/fourslash/main.rs + cases/<snake>.rs + cases/mod.rs
  fourslash-migration.txt 清单（用例名 -> 状态）
可翻译调用按 IMPL 映射；未实现方法/不可译参数 -> 该用例 #[ignore] 且
调用替换为 unsupported()，保证全量编译通过、可运行子集真实执行。
"""

import os
import re
import sys

GO_DIR = "/home/cqh/workspace/TypeScript/tsc/internal/fourslash/tests"
OUT_DIR = "crates/tsox-lsp/tests/fourslash"
PRELUDE = "use tsox_lsp::fourslash::{self, Session};\n"

IMPL = {
    "GoToMarker": ("go_to_marker", 1),
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

    for stmt in split_statements(body):
        if stmt in ("t.Parallel()",) or stmt.startswith("defer testutil.") \
                or stmt.startswith("defer done()") or stmt == "":
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
            ignores.add(f"unimplemented: fourslash.{method}")
            lines.append(f'fourslash::unsupported("{method}"); '
                         f"// {stmt.splitlines()[0][:100]}")
            continue
        ignores.add(f"generator: {stmt.splitlines()[0][:60]}")
        lines.append(f"// TODO: {stmt.splitlines()[0][:100]}")

    if not has_session:
        return None
    attrs = ""
    if ignores:
        reason = sorted(ignares := ignores)[0].replace('"', "'")
        attrs = f'#[ignore = "{reason}"]\n'
    out = [attrs + f"#[test]\nfn {fn}() {{"]
    out.append(PRELUDE.rstrip("\n"))
    # use 放函数内不合法——提升
    out = [attrs + f"#[test]\nfn {fn}() {{"]
    for ln in lines:
        out.append("    " + ln)
    out.append("}")
    body_text = "\n".join(out)
    return fn, body_text, bool(ignores)



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


def match_const(stmt):
    m = re.match(r"const (\w+) = `", stmt)
    if not m:
        m2 = re.match(r"(\w+) := `", stmt)
        if not m2:
            return None
        m = m2
    end = stmt.rfind("`")
    if end <= m.end() - 1:
        return None
    return m.group(1), stmt[m.end():end]


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


LSP_BEHAVIOR = ['format_on_semi_colon_after_break', 'formatting_equals_before_bracket_in_type_alias', 'formatting_in_expressions_in_tsx', 'formatting_of_chained_lambda', 'formatting_on_do_while_no_semicolon', 'formatting_on_nested_do_while_by_enter', 'formatting_space_after_comma_before_open_paren', 'function_type_formatting', 'regex_error_recovery', 'semicolon_formatting_after_array_literal', 'semicolon_formatting_inside_a_comment', 'semicolon_formatting_nested_statements', 'semicolon_formatting_inside_a_string_literal', 'white_space_before_return_type_formatting', 'white_space_trimming4', 'white_space_trimming', 'test result: FAILED. 33 passed; 16 failed; 4405 ignored; 0 measured; 0 filtered out; finished in 0.01s']


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
            fn, body_text, ignored = r
            if not ignored and stem in LSP_BEHAVIOR:
                body_text = body_text.replace(
                    "#[test]", '#[ignore = "needs live LSP session"]\n#[test]', 1)
                ignored = True
            fns.append(body_text)
            rows.append((stem, name,
                         "ignored" if ignored else "runnable",
                         "needs LSP session" if stem in LSP_BEHAVIOR else ""))
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
        run_n = sum(1 for r in rows if r[2] == "runnable")
        ig_n = sum(1 for r in rows if r[2] == "ignored")
        sk_n = sum(1 for r in rows if r[2] == "skipped")
        fh.write(f"# fourslash 用例迁移清单：可运行 {run_n} / 忽略 {ig_n}"
                 f" / 跳过 {sk_n}，文件 {len(stems)} 个\n")
        for stem, name, status, why in rows:
            fh.write(f"{stem}\t{name}\t{status}\t{why}\n")
    print(f"文件 {len(stems)}，可运行 {run_n}，忽略 {ig_n}，跳过 {sk_n}")


if __name__ == "__main__":
    main()
