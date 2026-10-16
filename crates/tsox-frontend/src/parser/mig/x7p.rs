#![allow(dead_code, unused_imports, unused_variables)]

use std::sync::Arc;

use crate::ast::diagnostic::Diagnostic;
use crate::parser::parsing_context::Parser;
use tsox_core::core::text::TextRange;
use tsox_core::diagnostics::Message;
use crate::ast::*;

thread_local! {
    /// Go parser.jsDiagnostics:Parser 无此字段,解析期以线程局部累积,
    /// 建 SourceFile 时经 take_js_diagnostics 转移
    static JS_DIAGNOSTICS: std::cell::RefCell<Vec<Diagnostic>> =
        const { std::cell::RefCell::new(Vec::new()) };
}

pub(crate) fn take_js_diagnostics() -> Vec<Diagnostic> { ::tsox_core::fntrace::enter("take_js_diagnostics"); 
    JS_DIAGNOSTICS.with(|c| std::mem::take(&mut *c.borrow_mut()))
}

impl Parser {
    pub fn js_error_at_range(
        &mut self,
        loc: TextRange,
        message: Message,
        message_args: &[&str],
    ) { ::tsox_core::fntrace::enter("js_error_at_range"); 
        let source_text = self.scanner.text.clone();
        let start = skip_trivia(&source_text, loc.pos());
        JS_DIAGNOSTICS.with(|c| {
            c.borrow_mut().push(Diagnostic::new(
                None,
                TextRange::new(start, loc.end()),
                message,
                message_args.iter().map(|s| s.to_string()).collect(),
            ))
        });
    }
}

fn skip_trivia(text: &str, pos: usize) -> usize { ::tsox_core::fntrace::enter("skip_trivia"); 
    let mut pos = pos.min(text.len());
    while let Some(ch) = text[pos..].chars().next() {
        if !ch.is_whitespace() {
            break;
        }
        pos += ch.len_utf8();
    }
    pos
}
