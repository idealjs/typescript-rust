#!/usr/bin/env python3
"""通用 .rs 文件拆分器: 按顶层条目分组到 <=MAX 行的子模块, 超大固有 impl 按方法再拆."""
import re
import sys
import os

MAX = 290

KEYWORDS = r'(?:pub(?:\([^)]*\))?\s+)?(?:async\s+)?(?:unsafe\s+)?(?:extern\s+(?:"[^"]*")?\s+)?(fn|struct|enum|union|trait|impl|type|const|static|mod|macro_rules!)'
ITEM_RE = re.compile(r'^' + KEYWORDS + r'\s*[!\s]?\s*([A-Za-z_][A-Za-z0-9_]*)?')
ATTR_RE = re.compile(r'^(#\[|///|//!|//)')
IMPL_METHOD_RE = re.compile(r'^    (?:#\[[^\]]*\]\s*)?(?:pub(?:\([^)]*\))?\s+)?(?:const\s+)?fn\b')
KEYWORD_NAMES = ('impl', 'fn', 'type', 'enum', 'struct', 'const', 'static', 'mod', 'trait', 'union')


def item_kind_name(line):
    m = ITEM_RE.match(line)
    if not m:
        return None, None
    kind = m.group(1)
    name = m.group(2)
    if kind == 'impl':
        rest = line[m.end():]
        tm = re.search(r'([A-Za-z_][A-Za-z0-9_]*)\s*[<{(]', rest)
        name = tm.group(1) if tm else 'impl'
    return kind, name


def snake(name):
    if name.isupper():
        return re.sub(r'[^a-z0-9_]', '_', name.lower()).strip('_') or 'part'
    s = re.sub(r'(?<!^)(?=[A-Z])', '_', name).lower()
    return re.sub(r'[^a-z0-9_]', '_', s).strip('_') or 'part'


def find_block_end(lines, start):
    depth = 0
    started = False
    i = start
    while i < len(lines):
        depth += lines[i].count('{') - lines[i].count('}')
        if '{' in lines[i]:
            started = True
        if started and depth <= 0:
            return i
        i += 1
    return len(lines) - 1


def split_impl_block(block):
    n = len(block)
    impl_line = next(i for i, l in enumerate(block) if ITEM_RE.match(l))
    starts = []
    for i in range(impl_line + 1, n):
        l = block[i]
        if IMPL_METHOD_RE.match(l):
            j = i
            while j - 1 > impl_line and re.match(r'^\s*(#\[|///|//)', block[j - 1]):
                j -= 1
            starts.append(j)
        elif l.startswith('}'):
            break
    if not starts:
        return [('IMPL', '\n'.join(block).strip())]
    ends = []
    for idx, st in enumerate(starts):
        if idx + 1 < len(starts):
            ends.append(starts[idx + 1])
        else:
            ends.append(next(i for i, l in enumerate(block) if i > st and l.startswith('}')))
    items = list(zip(starts, ends))
    for (a, b) in items:
        for k in range(a, b):
            block[k] = re.sub(r'^    fn ', '    pub(crate) fn ', block[k])
    groups, cur, cur_lines = [], [], 0
    for (a, b) in items:
        ln = b - a
        if cur and cur_lines + ln > MAX:
            groups.append(cur)
            cur, cur_lines = [], 0
        cur.append((a, b))
        cur_lines += ln
    if cur:
        groups.append(cur)
    header_line = block[impl_line]
    return [('IMPL', header_line + '\n' + '\n'.join('\n'.join(block[a:b]) for (a, b) in g) + '\n}') for g in groups]


def revert_trait_impl_methods(body):
    out_lines = body.split('\n')
    i = 0
    while i < len(out_lines):
        if re.match(r'^(pub )?trait [A-Za-z_]', out_lines[i]) or re.match(r'^impl\b.*\bfor\b', out_lines[i]):
            j = find_block_end(out_lines, i)
            for k in range(i, j + 1):
                out_lines[k] = out_lines[k].replace('    pub(crate) fn ', '    fn ')
            i = j
        i += 1
    return '\n'.join(out_lines)


def bump_struct_fields(body):
    out_lines = body.split('\n')
    i = 0
    while i < len(out_lines):
        if re.match(r'^(pub(\([^)]*\))? )?struct [A-Za-z_]', out_lines[i]):
            j = find_block_end(out_lines, i)
            for k in range(i + 1, j):
                if re.match(r'^    ([a-z_][a-z0-9_]*):', out_lines[k]):
                    out_lines[k] = '    pub(crate) ' + out_lines[k][4:]
            i = j
        i += 1
    return '\n'.join(out_lines)


def split(path):
    lines = open(path).read().split('\n')
    n = len(lines)
    starts = []
    for i, l in enumerate(lines):
        if l and not l[0].isspace() and ITEM_RE.match(l):
            starts.append(i)
    if not starts:
        print(f'{path}: 无顶层条目, 跳过')
        return False
    real_starts = []
    for st in starts:
        j = st
        while j - 1 >= 0 and lines[j - 1] and not lines[j - 1][0].isspace() and ATTR_RE.match(lines[j - 1]):
            j -= 1
        real_starts.append(j)
    real_starts = sorted(set(real_starts))
    header = lines[:real_starts[0]]

    items = []
    for idx, st in enumerate(real_starts):
        e = real_starts[idx + 1] if idx + 1 < len(real_starts) else n
        items.append((st, e))

    rel = os.path.dirname(path).replace('crates/tsox/src/', '').replace('/', '::')
    if os.path.basename(path) == 'mod.rs':
        parent_path = rel.rsplit('::', 1)[0] if '::' in rel else ''
    else:
        parent_path = rel
    if parent_path.startswith('crates'):
        parent_path = ''

    root_items, move_items, tests_item = [], [], None
    for (s, e) in items:
        first = next(l for l in lines[s:e] if ITEM_RE.match(l))
        kind, _ = item_kind_name(first)
        text = '\n'.join(lines[s:e]).strip()
        if kind == 'mod':
            root_items.append((s, e))
        elif text.startswith('#[cfg(test)]') and 'mod tests' in text:
            tests_item = (s, e)
        else:
            move_items.append((s, e))

    expanded = []
    for (s, e) in move_items:
        block = lines[s:e]
        head = next((l for l in block if ITEM_RE.match(l)), '')
        if e - s > MAX and re.match(r'^impl\b', head) and ' for ' not in head:
            expanded.extend(split_impl_block(block))
        else:
            expanded.append((s, e))
    move_items = expanded

    groups, cur, cur_lines = [], [], 0
    for it in move_items:
        if isinstance(it, tuple) and len(it) == 2 and isinstance(it[0], str) and it[0] == 'IMPL':
            ln = len(it[1].split('\n'))
        else:
            ln = it[1] - it[0]
        if cur and cur_lines + ln > MAX:
            groups.append(cur)
            cur, cur_lines = [], 0
        cur.append(it)
        cur_lines += ln
    if cur:
        groups.append(cur)

    mod_dir = os.path.dirname(path) if path.endswith('mod.rs') else path[:-3]
    os.makedirs(mod_dir, exist_ok=True)

    names = []
    for g in groups:
        if isinstance(g[0], tuple) and len(g[0]) == 2 and g[0][0] == 'IMPL':
            m = re.search(r'impl[^{]*?([A-Za-z_][A-Za-z0-9_]*)\s*[<{]', g[0][1])
            base = snake(m.group(1)) if m else 'impl'
        else:
            first = next(l for l in lines[g[0][0]:g[-1][1]] if ITEM_RE.match(l))
            kind, name = item_kind_name(first)
            base = snake(name or kind)
        if base in KEYWORD_NAMES:
            base = base + '_chunk'
        mod_name = base
        n = 2
        while mod_name in names or os.path.exists(os.path.join(mod_dir, mod_name + '.rs')):
            mod_name = f'{base}_{n}'
            n += 1
        names.append(mod_name)

        parts = []
        for it in g:
            if isinstance(it, tuple) and len(it) == 2 and isinstance(it[0], str) and it[0] == 'IMPL':
                t = it[1]
                if parent_path:
                    t = re.sub(r'\bsuper::', f'crate::{parent_path}::', t)
                    t = t.replace(f'crate::{parent_path}::{parent_path}::', f'crate::{parent_path}::')
                parts.append(t)
                continue
            s, e = it
            seg = '\n'.join(lines[s:e])
            if parent_path:
                seg = re.sub(r'\bsuper::', f'crate::{parent_path}::', seg)
                seg = seg.replace(f'crate::{parent_path}::{parent_path}::', f'crate::{parent_path}::')
            seg = re.sub(r'^fn ', 'pub(crate) fn ', seg, flags=re.M)
            seg = re.sub(r'^struct ', 'pub(crate) struct ', seg, flags=re.M)
            seg = re.sub(r'^enum ', 'pub(crate) enum ', seg, flags=re.M)
            seg = re.sub(r'^type ', 'pub(crate) type ', seg, flags=re.M)
            seg = re.sub(r'^const ', 'pub(crate) const ', seg, flags=re.M)
            seg = re.sub(r'^static ', 'pub(crate) static ', seg, flags=re.M)
            seg = re.sub(r'^    fn ', '    pub(crate) fn ', seg, flags=re.M)
            seg = revert_trait_impl_methods(seg)
            seg = bump_struct_fields(seg)
            parts.append(seg)
        body = '\n\n'.join(p.strip() for p in parts)
        out = '#![allow(unused_imports)]\n\nuse super::*;\n\n' + body.strip() + '\n'
        open(f'{mod_dir}/{mod_name}.rs', 'w').write(out)

    root_parts = [l for l in header if l.strip()]
    root_parts += [f'mod {m};' for m in names]
    root_parts += [f'#[allow(unused_imports)]\npub use {m}::*;' for m in names]
    for (s, e) in root_items:
        root_parts.append('\n'.join(lines[s:e]).strip())
    if tests_item:
        root_parts.append('#[cfg(test)]\nmod tests;')
        ttext = '\n'.join(lines[tests_item[0]:tests_item[1]]).strip()
        tlines = ttext.split('\n')
        inner = tlines[1:]
        while inner and not inner[-1].strip():
            inner.pop()
        if inner and inner[-1].strip() == '}':
            inner = inner[:-1]
        dedented = [l[4:] if l.startswith('    ') else l for l in inner]
        open(f'{mod_dir}/tests.rs', 'w').write('\n'.join(dedented).lstrip('\n').rstrip() + '\n')
    open(path, 'w').write('\n'.join(root_parts) + '\n')
    print(f'{path}: 拆为 {len(names)} 个子模块: {", ".join(names)}')
    return True


if __name__ == '__main__':
    split(sys.argv[1])
