mod comments;
mod es5;

pub(crate) use comments::*;
pub(crate) use es5::*;

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
