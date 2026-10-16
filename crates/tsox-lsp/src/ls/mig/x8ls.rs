#![allow(dead_code, unused_imports, unused_variables)]

use std::sync::Arc;

use tsox_frontend::ast::{self, Node, SyntaxKind};
use tsox_frontend::scanner;
use tsox_checker::checker::Checker;
use tsox_core::core;

pub fn leading_indentation(text: &str) -> &str { ::tsox_core::fntrace::enter("leading_indentation"); 
    let mut end = 0;
    let bytes = text.as_bytes();
    while end < bytes.len() && (bytes[end] == b' ' || bytes[end] == b'\t') {
        end += 1;
    }
    &text[..end]
}

pub fn line_has_only_jsdoc_asterisk(line: &str) -> bool { ::tsox_core::fntrace::enter("line_has_only_jsdoc_asterisk"); 
    let line = line.trim_start_matches([' ', '\t']);
    if let Some(rest) = line.strip_prefix('*') {
        rest.chars().all(|c| c == ' ' || c == '\t')
    } else {
        false
    }
}

pub fn lsp_range_contains(outer: &crate::lsp::lsproto_lsp_basic::Range, inner: &crate::lsp::lsproto_lsp_basic::Range) -> bool { ::tsox_core::fntrace::enter("lsp_range_contains"); 
    use std::cmp::Ordering;
    crate::lsp::lsproto_util::compare_positions(&outer.start, &inner.start) != Ordering::Greater
        && crate::lsp::lsproto_util::compare_positions(&inner.end, &outer.end) != Ordering::Greater
}

pub fn modifier_like_kind(node: Option<&Arc<Node>>) -> SyntaxKind { ::tsox_core::fntrace::enter("modifier_like_kind"); 
    let Some(node) = node else {
        return SyntaxKind::Unknown;
    };
    if ast::mig::m3g::is_modifier(node) {
        return node.kind;
    }
    if ast::is_identifier(node) {
        let keyword_kind = tsox_frontend::scanner::mig::m3i::identifier_to_keyword_kind(node);
        if keyword_kind != SyntaxKind::Unknown && ast::is_modifier_kind(keyword_kind) {
            return keyword_kind;
        }
    }
    SyntaxKind::Unknown
}

pub fn move_range_past_modifiers(node: &Node) -> tsox_core::core::text::TextRange { ::tsox_core::fntrace::enter("move_range_past_modifiers"); 
    if let Some(modifiers) = node.modifiers() {
        if !modifiers.nodes.is_empty() {
            let last_mod = modifiers.nodes.last().unwrap();
            return tsox_core::core::text::TextRange::new(last_mod.end(), node.end());
        }
    }
    tsox_core::core::text::TextRange::new(node.pos(), node.end())
}

pub fn needs_jsx_namespace_fix(jsx_namespace: &str, symbol_token: &Arc<Node>, ch: &Checker) -> bool { ::tsox_core::fntrace::enter("needs_jsx_namespace_fix"); 
    if tsox_frontend::scanner::mig::m3i::is_intrinsic_jsx_name(symbol_token.text()) {
        return true;
    }
    let namespace_symbol = ch.resolve_name(
        jsx_namespace,
        symbol_token,
        ast::SymbolFlags::VALUE,
        true,
    );
    let Some(namespace_symbol) = namespace_symbol else {
        return true;
    };
    if namespace_symbol
        .declarations
        .iter()
        .any(|d| ast::mig::m3g_2::is_type_only_import_or_export_declaration(d))
    {
        (!namespace_symbol.flags).contains(ast::SymbolFlags::VALUE)
    } else {
        false
    }
}
