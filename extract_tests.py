#!/usr/bin/env python3
"""提取 .rs 文件顶层 #[cfg(test)] mod 块（或含 #[test] 的顶层 mod 块）到独立测试文件.

用法: extract_tests.py <file.rs> [--dry]
- mod tests { ... }    -> <mod_dir>/tests.rs      (mod.rs: 同目录;  X.rs: X/tests.rs)
- mod <其他名> { ... }  -> <mod_dir>/<名>_tests.rs (声明同步改名, 保持豁免命名)
- 内容原样搬移不做 dedent, 由 rustfmt 事后规范
"""
import sys
import os
import re


def strip_to_code(text):
    """返回 (code_chars, code_only_text): 字符串/注释内容替换为空格, 保留行列结构."""
    out = []
    i, n = 0, len(text)
    state = 'code'
    hashes = 0
    while i < n:
        c = text[i]
        nxt = text[i + 1] if i + 1 < n else ''
        if state == 'code':
            if c == '/' and nxt == '/':
                state = 'line'
                out.append('  ')
                i += 2
                continue
            if c == '/' and nxt == '*':
                state = 'block'
                out.append('  ')
                i += 2
                continue
            if c == '"':
                state = 'str'
                out.append(' ')
                i += 1
                continue
            if c == 'r' and nxt in '#"':
                j = i + 1
                h = 0
                while j < n and text[j] == '#':
                    h += 1
                    j += 1
                if j < n and text[j] == '"':
                    hashes = h
                    state = 'rawstr'
                    out.append(' ' * (j + 1 - i))
                    i = j + 1
                    continue
            if c == 'b' and nxt == '"':
                state = 'str'
                out.append('  ')
                i += 2
                continue
            if c == "'":
                # 生命周期或字符: 'a' 是字符, 'a 是生命周期
                if i + 2 < n and text[i + 2] == "'":
                    state = 'char'
                    out.append(' ')
                    i += 1
                    continue
                if nxt == '\\':
                    state = 'char'
                    out.append(' ')
                    i += 1
                    continue
                if i + 1 < n and (text[i + 1].isalnum() or text[i + 1] == '_'):
                    # 生命周期 'abc — 保留原字符
                    out.append(c)
                    i += 1
                    continue
                state = 'char'
                out.append(' ')
                i += 1
                continue
            out.append(c)
            i += 1
            continue
        if state == 'line':
            if c == '\n':
                state = 'code'
                out.append('\n')
            else:
                out.append(' ')
            i += 1
            continue
        if state == 'block':
            if c == '*' and nxt == '/':
                state = 'code'
                out.append('  ')
                i += 2
                continue
            out.append('\n' if c == '\n' else ' ')
            i += 1
            continue
        if state == 'str':
            if c == '\\':
                out.append(' \n' if nxt == '\n' else '  ')
                i += 2
                continue
            if c == '"':
                state = 'code'
            out.append('\n' if c == '\n' else ' ')
            i += 1
            continue
        if state == 'rawstr':
            if c == '"' and text[i + 1:i + 1 + hashes] == '#' * hashes:
                state = 'code'
                out.append(' ' * (1 + hashes))
                i += 1 + hashes
                continue
            out.append('\n' if c == '\n' else ' ')
            i += 1
            continue
        if state == 'char':
            if c == '\\':
                out.append(' \n' if nxt == '\n' else '  ')
                i += 2
                continue
            if c == "'":
                state = 'code'
            out.append('\n' if c == '\n' else ' ')
            i += 1
            continue
    return ''.join(out)


MOD_RE = re.compile(r'^(?:#\[[^\]]*\]\s*)*(?:pub(?:\([^)]*\))?\s+)?mod\s+([A-Za-z_][A-Za-z0-9_]*)\s*\{')
ATTR_RE = re.compile(r'^#\[')


def find_blocks(lines):
    """返回 [(start_line, end_line, mod_name, header_line)], 行号 0 基, 含属性行."""
    code = strip_to_code('\n'.join(lines))
    clines = code.split('\n')
    depth = 0
    depth_at_start = []
    for ln in clines:
        depth_at_start.append(depth)
        depth += ln.count('{') - ln.count('}')
    blocks = []
    i = 0
    while i < len(lines):
        if depth_at_start[i] != 0:
            i += 1
            continue
        stripped = clines[i].strip()
        if stripped.startswith('#['):
            # 收集连续属性行
            j = i
            while j < len(lines) and clines[j].strip().startswith('#['):
                j += 1
            if j < len(lines) and depth_at_start[j] == 0 and MOD_RE.match(clines[j].strip() or ''):
                m = MOD_RE.match(clines[j].strip())
                # 块尾: 深度回到 0 的行
                end = j
                d = 0
                for k in range(j, len(lines)):
                    d += clines[k].count('{') - clines[k].count('}')
                    if k > j and d == 0:
                        end = k
                        break
                blocks.append((i, end, m.group(1), j))
                i = end + 1
                continue
        i += 1
    return blocks


def main():
    path = sys.argv[1]
    dry = '--dry' in sys.argv
    text = open(path).read()
    lines = text.split('\n')
    blocks = find_blocks(lines)
    results = []
    for (s, e, name, hdr) in blocks:
        body = '\n'.join(lines[s:e + 1])
        has_cfg_test = bool(re.search(r'#\[\s*cfg\([^)]*test', '\n'.join(lines[s:hdr])))
        has_test_fn = '#[test]' in body
        if not (has_cfg_test or has_test_fn):
            continue
        results.append((s, e, name, hdr, has_cfg_test, has_test_fn))
    if not results:
        print(f'{path}: 无内联测试模块')
        return
    for (s, e, name, hdr, cfg, tfn) in results:
        print(f'  mod {name}: 行 {s + 1}-{e + 1} cfg_test={cfg} test_fn={tfn}')
    if dry:
        return

    is_mod_rs = os.path.basename(path) == 'mod.rs'
    stem = os.path.basename(path)[:-3]
    mod_dir = os.path.dirname(path) if is_mod_rs else path[:-3]
    os.makedirs(mod_dir, exist_ok=True)

    out_lines = list(lines)
    for (s, e, name, hdr, cfg, tfn) in sorted(results, key=lambda b: -b[0]):
        if name == 'tests':
            target, decl_name = 'tests.rs', 'tests'
        elif name.endswith('_tests'):
            target, decl_name = f'{name}.rs', name
        else:
            target, decl_name = f'{name}_tests.rs', f'{name}_tests'
        tpath = os.path.join(mod_dir, target)
        if os.path.exists(tpath):
            print(f'!! 目标已存在, 跳过: {tpath}')
            continue
        inner = lines[hdr + 1:e]
        while inner and not inner[-1].strip():
            inner.pop()
        open(tpath, 'w').write('\n'.join(inner).strip('\n') + '\n')
        out_lines[s:e + 1] = ['#[cfg(test)]', f'mod {decl_name};']
        print(f'  -> {tpath} ({e - hdr} 行)')
    open(path, 'w').write('\n'.join(out_lines))


if __name__ == '__main__':
    main()
