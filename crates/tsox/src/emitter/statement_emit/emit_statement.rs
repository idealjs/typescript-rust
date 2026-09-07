use crate::ast::Node;
use crate::emitter::*;

use super::*;

pub(crate) fn emit_statement<S: EmitSink>(
    node: &Node,
    source: &str,
    comment_cuts: &[(usize, usize)],
    replacements: &[(usize, usize, &str, Option<usize>)],
    sink: &mut S,
) {
    let mut cuts: Vec<(usize, usize)> = Vec::new();
    collect_type_cuts(node, source, &mut cuts);

    if !comment_cuts.is_empty() {
        for &(cs, ce) in comment_cuts {
            if ce > node.pos() && cs < node.end() {
                cuts.push((cs, ce));
            }
        }
    }

    let mut stmt_replacements: Vec<(usize, usize, &str, Option<usize>)> = Vec::new();
    for &(rs, re, repl, src_pos) in replacements {
        if re > node.pos() && rs < node.end() {
            stmt_replacements.push((rs, re, repl, src_pos));
        }
    }

    if cuts.is_empty() && stmt_replacements.is_empty() {
        sink.emit_source(source, node.pos(), node.end());
        return;
    }

    let mut ops: Vec<(usize, usize, Option<(&str, Option<usize>)>)> = Vec::new();
    for (cs, ce) in &cuts {
        if *ce > node.pos() && *cs < node.end() {
            ops.push(((*cs).max(node.pos()), (*ce).min(node.end()), None));
        }
    }
    for (rs, re, repl, src_pos) in &stmt_replacements {
        ops.push((*rs, *re, Some((*repl, *src_pos))));
    }
    ops.sort_by_key(|&(s, _, _)| s);

    let mut pos = node.pos();
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
    if pos < node.end() {
        sink.emit_source(source, pos, node.end());
    }
}
