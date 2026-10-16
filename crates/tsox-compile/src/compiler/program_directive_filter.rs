#![allow(unused_imports)]

use super::*;
use tsox_frontend::scanner::CommentDirectiveKind;

impl Program {
    /// Go getBindAndCheckDiagnosticsWithChecker →
    /// getDiagnosticsWithPrecedingDirectives：诊断上方连续空行/注释行内
    /// 含 @ts-ignore/@ts-expect-error 则抑制并记该指令已用，未使用的
    /// expect-error 报 TS2578
    pub(crate) fn filter_diagnostics_with_preceding_directives(
        &self,
        diagnostics: Vec<Diagnostic>,
    ) -> Vec<Diagnostic> { ::tsox_core::fntrace::enter("filter_diagnostics_with_preceding_directives"); 
        let mut files_with_directives: Vec<u64> = Vec::new();
        let mut states: std::collections::HashMap<
            u64,
            (std::collections::HashMap<usize, usize>, Vec<bool>),
        > = std::collections::HashMap::new();
        for d in &diagnostics {
            let Some(file) = &d.file else { continue };
            if file.comment_directives.is_empty() {
                continue;
            }
            if states.contains_key(&file.id()) {
                continue;
            }
            files_with_directives.push(file.id());
            let directives: std::collections::HashMap<usize, usize> = file
                .comment_directives
                .iter()
                .enumerate()
                .map(|(i, directive)| (file.line_map.line_at(directive.pos), i))
                .collect();
            let consumed = vec![false; file.comment_directives.len()];
            states.insert(file.id(), (directives, consumed));
        }
        if states.is_empty() {
            return diagnostics;
        }

        let mut filtered: Vec<Diagnostic> = Vec::with_capacity(diagnostics.len());
        let by_id: std::collections::HashMap<u64, Arc<SourceFile>> = diagnostics
            .iter()
            .filter_map(|d| d.file.as_ref().map(|f| (f.id(), Arc::clone(f))))
            .collect();
        for d in diagnostics {
            let Some(file) = &d.file else {
                filtered.push(d);
                continue;
            };
            let Some((directives, consumed)) = states.get_mut(&file.id()) else {
                filtered.push(d);
                continue;
            };
            if let Some(index) = walk_suppressing_directive(file, directives, d.loc.pos()) {
                consumed[index] = true;
            } else {
                filtered.push(d);
            }
        }

        for id in files_with_directives {
            let Some(file) = by_id.get(&id) else { continue };
            let Some((_, consumed)) = states.get(&id) else { continue };
            for (i, directive) in file.comment_directives.iter().enumerate() {
                if directive.kind == CommentDirectiveKind::ExpectError && !consumed[i] {
                    filtered.push(Diagnostic::new(
                        Some(Arc::clone(file)),
                        tsox_core::core::text::TextRange::new(directive.pos, directive.end),
                        tsox_core::diagnostics::messages_generated::UNUSED_TS_EXPECT_ERROR_DIRECTIVE,
                        vec![],
                    ));
                }
            }
        }
        filtered
    }
}

fn walk_suppressing_directive(
    file: &SourceFile,
    directives: &std::collections::HashMap<usize, usize>,
    pos: usize,
) -> Option<usize> { ::tsox_core::fntrace::enter("walk_suppressing_directive"); 
    let mut line = file.line_map.line_at(pos);
    while line > 0 {
        line -= 1;
        if let Some(&index) = directives.get(&line) {
            return Some(index);
        }
        if !is_comment_or_blank_line(
            &file.text,
            file.line_map.line_starts[line] as usize,
        ) {
            break;
        }
    }
    None
}

pub(crate) fn suppressed_by_preceding_directive(file: &SourceFile, pos: usize) -> bool { ::tsox_core::fntrace::enter("suppressed_by_preceding_directive"); 
    if file.comment_directives.is_empty() {
        return false;
    }
    let directives: std::collections::HashMap<usize, usize> = file
        .comment_directives
        .iter()
        .enumerate()
        .map(|(i, directive)| (file.line_map.line_at(directive.pos), i))
        .collect();
    walk_suppressing_directive(file, &directives, pos).is_some()
}

fn is_comment_or_blank_line(text: &str, pos: usize) -> bool { ::tsox_core::fntrace::enter("is_comment_or_blank_line"); 
    let bytes = text.as_bytes();
    let mut pos = pos;
    while pos < bytes.len() && (bytes[pos] == b' ' || bytes[pos] == b'\t') {
        pos += 1;
    }
    pos == bytes.len()
        || bytes[pos] == b'\r'
        || bytes[pos] == b'\n'
        || (pos + 1 < bytes.len() && bytes[pos] == b'/' && bytes[pos + 1] == b'/')
}
