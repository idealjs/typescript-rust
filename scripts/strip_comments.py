#!/usr/bin/env python3
"""Rust-aware comment stripper. Removes //, ///, //!, /* */ (nested),
/** */ while preserving string/char/raw-string/lifetime semantics.
Deletes comment-only lines entirely; strips trailing comments; collapses
resulting triple+ blank lines to one."""
import sys, os, glob

def strip(src):
    out = []
    i = 0
    n = len(src)
    line_comment = block_comment = in_str = in_char = in_raw = False
    raw_hashes = 0
    block_depth = 0
    prev_nonspace = ''
    while i < n:
        c = src[i]
        nxt = src[i+1] if i+1 < n else ''
        if line_comment:
            if c == '\n':
                line_comment = False
                out.append(c)
            i += 1
            continue
        if block_comment:
            if c == '/' and nxt == '*':
                block_depth += 1; i += 2; continue
            if c == '*' and nxt == '/':
                block_depth -= 1; i += 2
                if block_depth == 0:
                    block_comment = False
                continue
            i += 1
            continue
        if in_str:
            if c == '\\':
                out.append(src[i:i+2]); i += 2; continue
            if c == '"':
                in_str = False
            out.append(c); i += 1
            continue
        if in_raw:
            if c == '"' and src[i+1:i+1+raw_hashes] == '#'*raw_hashes:
                in_raw = False
                out.append(src[i:i+1+raw_hashes]); i += 1+raw_hashes; continue
            out.append(c); i += 1
            continue
        if in_char:
            if c == '\\':
                out.append(src[i:i+2]); i += 2; continue
            if c == "'":
                in_char = False
            out.append(c); i += 1
            continue
        # normal state
        if c == '/' and nxt == '/':
            line_comment = True; i += 2; continue
        if c == '/' and nxt == '*':
            block_comment = True; block_depth = 1; i += 2; continue
        if c == '"':
            in_str = True; out.append(c); i += 1; prev_nonspace = '"'; continue
        if c == 'r' and (nxt == '"' or nxt == '#') and (prev_nonspace == '' or not (prev_nonspace.isalnum() or prev_nonspace == '_')):
            # raw string start: count hashes
            j = i + 1
            h = 0
            while j < n and src[j] == '#':
                h += 1; j += 1
            if j < n and src[j] == '"':
                in_raw = True; raw_hashes = h
                out.append(src[i:j+1]); i = j + 1; prev_nonspace = '"'; continue
            out.append(c); i += 1; prev_nonspace = 'r'; continue
        if c == "'":
            # char literal vs lifetime
            if nxt == '\\':
                in_char = True; out.append(c); i += 1; continue
            if i+2 < n and src[i+2] == "'":
                in_char = True; out.append(c); i += 1; continue
            if nxt.isalnum() or nxt == '_':
                # lifetime: consume identifier
                j = i + 1
                while j < n and (src[j].isalnum() or src[j] == '_'):
                    j += 1
                out.append(src[i:j]); i = j; continue
            in_char = True; out.append(c); i += 1; continue
        out.append(c)
        if not c.isspace(): prev_nonspace = c
        i += 1
    return ''.join(out)

def tidy(text):
    lines = []
    for l in text.split('\n'):
        l = l.rstrip()
        lines.append(l)
    # drop lines that became empty AND were comment-only originally is handled by empties;
    # collapse 3+ consecutive blank lines into 1
    out = []
    blanks = 0
    for l in lines:
        if l == '':
            blanks += 1
            if blanks > 1:
                continue
        else:
            blanks = 0
        out.append(l)
    # trim leading/trailing blank lines
    while out and out[0] == '': out.pop(0)
    while out and out[-1] == '': out.pop()
    return '\n'.join(out) + '\n'

changed = 0
files = sorted(set(glob.glob('src/**/*.rs', recursive=True) + glob.glob('tests/**/*.rs', recursive=True)
                   + glob.glob('src/bin/*.rs') + glob.glob('examples/**/*.rs', recursive=True)
                   + glob.glob('benches/**/*.rs', recursive=True)))
for p in files:
    src = open(p, encoding='utf-8').read()
    stripped = tidy(strip(src))
    if stripped != src:
        open(p, 'w', encoding='utf-8').write(stripped)
        changed += 1
print(f"files changed: {changed}/{len(files)}")
