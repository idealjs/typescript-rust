use std::sync::Arc;

use crate::ast::node_flags::ModifierFlags;
use crate::ast::{Node, NodeFlags, SyntaxKind};
use crate::core::compiler_options::CompilerOptions;
use crate::core::compiler_options::ScriptTarget;
use super::*;

pub(crate) fn emit_text_range<S: EmitSink>(
    source: &str,
    start: usize,
    end: usize,
    cuts: &[(usize, usize)],
    replacements: &[(usize, usize, &str, Option<usize>)],
    sink: &mut S,
) {
    if cuts.is_empty() && replacements.is_empty() {
        sink.emit_source(source, start, end);
        return;
    }

    let mut ops: Vec<(usize, usize, Option<(&str, Option<usize>)>)> = Vec::new();
    for &(cs, ce) in cuts {
        if ce > start && cs < end {
            ops.push((cs.max(start), ce.min(end), None));
        }
    }
    for &(rs, re, repl, src_pos) in replacements {
        if re > start && rs < end {
            ops.push((rs.max(start), re.min(end), Some((repl, src_pos))));
        }
    }
    if ops.is_empty() {
        sink.emit_source(source, start, end);
        return;
    }
    ops.sort_by_key(|&(s, _, _)| s);
    let mut pos = start;
    for (s, e, repl) in &ops {
        if *s > pos {
            sink.emit_source(source, pos, *s);
        }
        if let Some((r, src_pos)) = repl {
            if let Some(sp) = src_pos {
                sink.emit_source_mapped(r, *sp);
            } else {
                sink.emit_generated(r);
            }
        }
        pos = *e;
    }
    if pos < end {
        sink.emit_source(source, pos, end);
    }
}

pub(crate) fn collect_all_comment_ranges(text: &str) -> Vec<(usize, usize)> {
    let bytes = text.as_bytes();
    let len = bytes.len();
    let mut ranges: Vec<(usize, usize)> = Vec::new();
    let mut pos = 0usize;

    let mut prev_significant: char = ';';

    while pos < len {
        let b = bytes[pos];
        match b {
            b'/' if pos + 1 < len && bytes[pos + 1] == b'/' => {

                let start = pos;
                pos += 2;
                while pos < len && bytes[pos] != b'\n' && bytes[pos] != b'\r' {
                    pos += 1;
                }
                ranges.push((start, pos));
            }
            b'/' if pos + 1 < len && bytes[pos + 1] == b'*' => {

                let start = pos;
                pos += 2;
                while pos < len {
                    if bytes[pos] == b'*' && pos + 1 < len && bytes[pos + 1] == b'/' {
                        pos += 2;
                        break;
                    }
                    pos += 1;
                }
                ranges.push((start, pos));
            }
            b'/' => {

                if is_regex_context(prev_significant) {
                    let start = pos;
                    pos += 1;
                    let mut in_class = false;
                    while pos < len {
                        let c = bytes[pos];
                        if c == b'\\' && pos + 1 < len {
                            pos += 2;
                            continue;
                        }
                        if c == b'[' {
                            in_class = true;
                        }
                        if c == b']' {
                            in_class = false;
                        }
                        if c == b'/' && !in_class {
                            pos += 1;

                            while pos < len && is_regex_flag_char(bytes[pos]) {
                                pos += 1;
                            }
                            break;
                        }
                        if c == b'\n' {

                            break;
                        }
                        pos += 1;
                    }
                    let _ = start;
                } else {
                    pos += 1;
                }
                prev_significant = '/';
            }
            b'\'' | b'"' => {

                let quote = b;
                pos += 1;
                while pos < len {
                    let c = bytes[pos];
                    if c == b'\\' && pos + 1 < len {
                        pos += 2;
                        continue;
                    }
                    if c == quote {
                        pos += 1;
                        break;
                    }
                    if c == b'\n' {

                        break;
                    }
                    pos += 1;
                }
                prev_significant = char::from(quote);
            }
            b'`' => {

                prev_significant = '`';
                pos += 1;
                skip_template_literal(text, &mut pos);
            }
            b' ' | b'\t' | b'\n' | b'\r' => {

                pos += 1;
            }
            _ => {
                prev_significant = char::from(b);
                pos += 1;
            }
        }
    }

    ranges
}

pub(crate) fn skip_template_literal(text: &str, pos: &mut usize) {
    let bytes = text.as_bytes();
    let len = bytes.len();
    while *pos < len {
        let b = bytes[*pos];
        if b == b'\\' && *pos + 1 < len {
            *pos += 2;
            continue;
        }
        if b == b'`' {
            *pos += 1;
            return;
        }
        if b == b'$' && *pos + 1 < len && bytes[*pos + 1] == b'{' {

            *pos += 2;
            let mut depth = 1;
            while *pos < len && depth > 0 {
                let c = bytes[*pos];
                match c {
                    b'{' => {
                        depth += 1;
                        *pos += 1;
                    }
                    b'}' => {
                        depth -= 1;
                        *pos += 1;
                    }
                    b'\'' | b'"' => {
                        let quote = c;
                        *pos += 1;
                        while *pos < len {
                            if bytes[*pos] == b'\\' && *pos + 1 < len {
                                *pos += 2;
                                continue;
                            }
                            if bytes[*pos] == quote {
                                *pos += 1;
                                break;
                            }
                            *pos += 1;
                        }
                    }
                    b'`' => {
                        *pos += 1;
                        skip_template_literal(text, pos);
                    }
                    _ => {
                        *pos += 1;
                    }
                }
            }
        } else {
            *pos += 1;
        }
    }
}

pub(crate) fn is_regex_context(prev: char) -> bool {
    matches!(
        prev,
        '(' | ','
            | '='
            | ':'
            | '['
            | '!'
            | '&'
            | '|'
            | '?'
            | '{'
            | '}'
            | ';'
            | '<'
            | '>'
            | '+'
            | '-'
            | '*'
            | '/'
            | '%'
            | '~'
            | '^'
            | '\n'
            | '\0'
    )
}

pub(crate) fn is_regex_flag_char(b: u8) -> bool {
    matches!(b, b'g' | b'i' | b'm' | b's' | b'u' | b'y' | b'd' | b'v')
}

pub(crate) fn needs_es5_downlevel(options: &CompilerOptions) -> bool {
    options.target == ScriptTarget::ES5
}

pub(crate) fn collect_es5_replacements(statements: &[Arc<Node>]) -> Vec<(usize, usize, &'static str)> {
    let mut replacements = Vec::new();
    for stmt in statements {
        collect_es5_replacements_recursive(stmt, &mut replacements);
    }
    replacements
}

pub(crate) fn collect_es5_replacements_recursive(
    node: &Node,
    replacements: &mut Vec<(usize, usize, &'static str)>,
) {
    if node.kind == crate::ast::SyntaxKind::VariableDeclarationList {
        let flags = node.flags;
        if flags.contains(NodeFlags::Const) {

            let pos = node.pos();
            replacements.push((pos, pos + 5, "var"));
        } else if flags.contains(NodeFlags::Let) {

            let pos = node.pos();
            replacements.push((pos, pos + 3, "var"));
        }
    }

    crate::ast::node_data_generated::for_each_child(node, |child| {
        collect_es5_replacements_recursive(child, replacements);
        false
    });
}

pub(crate) fn collect_export_modifier_cuts(stmt: &Node, source: &str) -> Vec<(usize, usize)> {
    let modifiers = match stmt.modifiers() {
        Some(m) => m,
        None => return Vec::new(),
    };
    if !modifiers.modifier_flags.contains(ModifierFlags::Export) {
        return Vec::new();
    }

    let mut cuts = Vec::new();
    let bytes = source.as_bytes();
    for mod_node in modifiers.list.iter() {
        if mod_node.kind == SyntaxKind::ExportKeyword || mod_node.kind == SyntaxKind::DefaultKeyword
        {
            let start = mod_node.pos();
            let mut end = mod_node.end();

            while end < bytes.len() && (bytes[end] == b' ' || bytes[end] == b'\t') {
                end += 1;
            }
            cuts.push((start, end));
        }
    }
    cuts
}
