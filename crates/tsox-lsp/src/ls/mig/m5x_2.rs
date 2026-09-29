#![allow(dead_code, unused_imports, unused_variables)]

use std::collections::HashMap;
use std::sync::Arc;

use tsox_frontend::ast::{self, Node, SourceFile, SyntaxKind};

use crate::lsp::lsproto_lsp::DocumentUri;
use crate::ls::types::{DocumentSymbol, SymbolKind};

pub const M5X_MAX_LENGTH: usize = 150;

fn spanmap_feature_document_symbols() -> u32 {
    crate::mig::m6b_2::FEATURE_DOCUMENT_SYMBOLS as u32
}

pub struct M5xDeclarationInfo {
    pub name: String,
    pub declaration: Arc<Node>,
    pub match_score: i64,
}

impl crate::ls::language_service::LanguageService {
    pub fn get_document_symbol_informations(
        &self,
        file: &Arc<SourceFile>,
        document_uri: &DocumentUri,
    ) -> Vec<crate::ls::types::SymbolInformation> {
        let doc_symbols = crate::ls::symbols::get_document_symbols_for_children(
            &file.node,
            file,
        );
        flatten_document_symbols(doc_symbols, document_uri)
    }

    pub fn provide_workspace_symbols(
        &self,
        programs: &[Arc<tsox_compile::compiler::Program>],
        query: &str,
    ) -> Vec<crate::ls::types::SymbolInformation> {
        let preferences = self.user_preferences();
        let exclude_library_symbols = preferences.exclude_library_symbols_in_nav_to.is_true();
        let mut source_files: HashMap<String, Arc<SourceFile>> = HashMap::new();
        for program in programs {
            for source_file in program.source_files() {
                if (program.has_ts_file() || !source_file.is_declaration_file)
                    && !should_exclude_file(source_file, program, exclude_library_symbols)
                {
                    source_files.insert(source_file.file_name.clone(), Arc::clone(source_file));
                }
            }
        }
        let mut infos: Vec<M5xDeclarationInfo> = Vec::new();
        for source_file in source_files.values() {
            let symbols = tsox_checker::binder::bind_source_file(source_file);
            let declaration_map = tsox_frontend::ast::mig::m3b_2::get_declaration_map(source_file, &symbols);
            for (name, declarations) in declaration_map.iter() {
                let score = get_match_score(name, query);
                if score >= 0 {
                    for declaration in declarations {
                        infos.push(M5xDeclarationInfo {
                            name: name.clone(),
                            declaration: Arc::clone(declaration),
                            match_score: score,
                        });
                    }
                }
            }
        }
        infos.sort_by(compare_declaration_infos);
        let count = infos.len().min(256);
        let mut symbols = Vec::with_capacity(count);
        for info in &infos[..count] {
            let node = &info.declaration;
            let source_file = programs
                .iter()
                .find_map(|p| crate::ls::mig::m5t_3::source_file_of_node(p, node))
                .expect("source file for declaration");
            let container = crate::ls::mig::m5w_6::get_container_node(node);
            let container_name =
                container.map(|c| ast::mig::m3b::get_declaration_name(&c)).filter(|n| !n.is_empty());
            let Some(name_node) = ast::get_name_of_declaration(node) else {
                continue;
            };
            let name_start =
                tsox_frontend::astnav::get_start_of_node(&name_node, &source_file, false);
            let name_range = tsox_core::core::text::TextRange::new(name_start, name_node.end());
            let script_view = crate::mig::m5u_conv::SourceFileScriptView {
                file: Arc::clone(&source_file),
            };
            let m5u_converters = crate::mig::m5u_conv::new_converters(
                crate::ls::lsconv_converters::PositionEncodingKind::Utf16,
                Box::new(crate::ls::lsconv_linemap::compute_lsp_line_starts),
            );
            let (location, fidelity) = m5u_converters.to_lsp_location_for_feature(
                &script_view,
                name_range,
                spanmap_feature_document_symbols(),
            );
            if !crate::ls::mig::m5u_2::fidelity_is_single_segment(&fidelity) {
                continue;
            }
            symbols.push(crate::ls::types::SymbolInformation {
                name: info.name.clone(),
                kind: crate::ls::symbols::symbol_kind_from_node(node.kind),
                location,
                container_name,
                tags: None,
                deprecated: None,
            });
        }
        symbols
    }
}

pub fn flatten_document_symbols(
    doc_symbols: Vec<DocumentSymbol>,
    document_uri: &DocumentUri,
) -> Vec<crate::ls::types::SymbolInformation> {
    fn flatten(
        symbols: &[DocumentSymbol],
        container_name: Option<String>,
        document_uri: &DocumentUri,
        result: &mut Vec<crate::ls::types::SymbolInformation>,
    ) {
        for symbol in symbols {
            result.push(crate::ls::types::SymbolInformation {
                name: symbol.name.clone(),
                kind: symbol.kind,
                location: crate::lsp::lsproto_lsp::Location {
                    uri: document_uri.clone(),
                    range: symbol.range.clone(),
                },
                container_name: container_name.clone(),
                tags: symbol.tags.clone(),
                deprecated: symbol.deprecated,
            });
            if let Some(children) = &symbol.children {
                if !children.is_empty() {
                    flatten(children, Some(symbol.name.clone()), document_uri, result);
                }
            }
        }
    }
    let mut result = Vec::new();
    flatten(&doc_symbols, None, document_uri, &mut result);
    result
}

pub fn is_prototype_expando(target: &Arc<Node>) -> bool {
    if ast::is_access_expression(target) {
        let access_name = ast::mig::m3e_4::get_element_or_property_access_name(target);
        return access_name.map(|n| ast::node_text(&n) == "prototype").unwrap_or(false);
    }
    false
}

pub fn merge_expandos(symbols: Vec<DocumentSymbol>) -> Vec<DocumentSymbol> {
    let mut symbols: Vec<Option<DocumentSymbol>> = symbols.into_iter().map(Some).collect();
    let mut name_to_expando_target_index: HashMap<String, Vec<usize>> = HashMap::new();
    let mut name_to_namespace_index: HashMap<String, usize> = HashMap::new();
    for (i, symbol) in symbols.iter().enumerate() {
        let Some(symbol) = symbol else { continue };
        if is_anonymous_name(&symbol.name) {
            continue;
        }
        if symbol.kind == SymbolKind::Class
            || symbol.kind == SymbolKind::Function
            || symbol.kind == SymbolKind::Variable
        {
            name_to_expando_target_index
                .entry(symbol.name.clone())
                .or_default()
                .push(i);
        }
        if symbol.kind == SymbolKind::Namespace {
            name_to_namespace_index.entry(symbol.name.clone()).or_insert(i);
        }
    }
    for i in 0..symbols.len() {
        let children = symbols[i].as_ref().and_then(|s| s.children.clone());
        if let Some(children) = children {
            if let Some(symbol) = symbols[i].as_mut() {
                symbol.children = Some(merge_expandos(children));
            }
        }
        let Some(symbol) = symbols[i].as_ref() else { continue };
        if is_anonymous_name(&symbol.name) {
            continue;
        }
        if symbol.kind == SymbolKind::Property {
            if let Some(indices) = name_to_expando_target_index.get(&symbol.name) {
                for &target_index in indices.iter().rev() {
                    let source = symbols[i].take();
                    if let (Some(target), Some(source)) =
                        (symbols[target_index].as_mut(), source)
                    {
                        merge_children(target, &source);
                    }
                }
            }
        }
        if let Some(symbol) = symbols[i].as_ref() {
            if symbol.kind == SymbolKind::Namespace {
                if let Some(&target_index) = name_to_namespace_index.get(&symbol.name) {
                    if target_index != i {
                        let source = symbols[i].take();
                        if let (Some(target), Some(source)) =
                            (symbols[target_index].as_mut(), source)
                        {
                            merge_children(target, &source);
                        }
                    }
                }
            }
        }
    }
    symbols.into_iter().flatten().collect()
}

pub fn merge_children(target: &mut DocumentSymbol, source: &DocumentSymbol) {
    if let Some(source_children) = &source.children {
        match &mut target.children {
            None => target.children = Some(source_children.clone()),
            Some(target_children) => {
                let mut merged = target_children.clone();
                merged.extend(source_children.iter().cloned());
                let mut merged = merge_expandos(merged);
                merged.sort_by(|a, b| {
                    crate::lsp::lsproto_util::compare_ranges(&a.range, &b.range)
                });
                *target_children = merged;
            }
        }
    }
}

pub fn is_anonymous_name(name: &str) -> bool {
    name == "<function>"
        || name == "<class>"
        || name == "export="
        || name == "default"
        || name == "constructor"
        || name == "()"
        || name == "new()"
        || name == "[]"
        || name.ends_with(") callback")
}

pub fn get_text_of_name(node: &Arc<Node>) -> String {
    match node.kind {
        SyntaxKind::Identifier | SyntaxKind::PrivateIdentifier | SyntaxKind::NumericLiteral => {
            ast::node_text(node).to_string()
        }
        SyntaxKind::StringLiteral => {
            format!("\"{}\"", tsox_frontend::format::mig::m4t_2::escape_string(
            &ast::node_text(node),
            tsox_frontend::format::mig::m4t_2::QuoteChar::DoubleQuote,
        ))
        }
        SyntaxKind::NoSubstitutionTemplateLiteral => {
            format!("`{}`", tsox_frontend::format::mig::m4t_2::escape_string(
                &ast::node_text(node),
                tsox_frontend::format::mig::m4t_2::QuoteChar::Backtick,
            ))
        }
        SyntaxKind::ComputedPropertyName => {
            let expr = crate::ls::mig::m5x_3::computed_property_name_expression(node);
            if expr.as_ref().is_some_and(|e| ast::is_string_or_numeric_literal_like(e)) {
                return get_text_of_name(expr.as_ref().unwrap());
            }
            tsox_frontend::scanner::mig::m3i::get_text_of_node(node)
        }
        _ => tsox_frontend::scanner::mig::m3i::get_text_of_node(node),
    }
}

pub fn get_unnamed_node_label(node: &Arc<Node>) -> String {
    if let Some(parent) = node.parent() {
        let parent = crate::ls::mig::m5x::walk_up_parenthesized_expressions(&parent);
        if ast::is_export_assignment(&parent) {
            if matches!(
                &parent.data,
                tsox_frontend::ast::NodeData::ExportAssignment(d) if d.is_export_equals
            ) {
                return "export=".to_string();
            }
            return "default".to_string();
        }
    }
    match node.kind {
        SyntaxKind::FunctionDeclaration
        | SyntaxKind::FunctionExpression
        | SyntaxKind::ArrowFunction => {
            if node
                .syntactic_modifier_flags()
                .contains(ast::ModifierFlags::Default)
            {
                return "default".to_string();
            }
            if let Some(parent) = node.parent() {
                if ast::is_call_expression(&parent) {
                    if let Some(expr) = crate::ls::mig::m5x_3::call_expression_expression(&parent) {
                        let mut name = get_call_expression_name(&expr);
                        if !name.is_empty() {
                            name = clean_callback_text(&name);
                            if name.len() > M5X_MAX_LENGTH {
                                return format!("{} callback", name);
                            }
                            let args = clean_callback_text(&get_call_expression_literal_args(&parent));
                            return format!("{}({}) callback", name, args);
                        }
                    }
                }
            }
            "<function>".to_string()
        }
        SyntaxKind::ClassDeclaration | SyntaxKind::ClassExpression => {
            if node
                .syntactic_modifier_flags()
                .contains(ast::ModifierFlags::Default)
            {
                return "default".to_string();
            }
            "<class>".to_string()
        }
        SyntaxKind::Constructor => "constructor".to_string(),
        SyntaxKind::CallSignature => "()".to_string(),
        SyntaxKind::ConstructSignature => "new()".to_string(),
        SyntaxKind::IndexSignature => "[]".to_string(),
        _ => String::new(),
    }
}

pub fn get_call_expression_name(node: &Arc<Node>) -> String {
    match node.kind {
        SyntaxKind::Identifier | SyntaxKind::PrivateIdentifier => ast::node_text(node).to_string(),
        SyntaxKind::PropertyAccessExpression => {
            let left = crate::ls::mig::m5x_3::access_expression_expression(node)
                .map(|left| get_call_expression_name(&left))
                .unwrap_or_default();
            let right = crate::ls::mig::m5x_3::access_expression_name(node)
                .map(|right| get_call_expression_name(&right))
                .unwrap_or_default();
            if !left.is_empty() {
                format!("{}.{}", left, right)
            } else {
                right
            }
        }
        _ => String::new(),
    }
}

pub fn get_call_expression_literal_args(call_expr: &Arc<Node>) -> String {
    let mut parts: Vec<String> = Vec::new();
    for arg in crate::ls::mig::m5x_3::call_expression_arguments(call_expr) {
        if ast::is_string_literal_like(&arg) || ast::is_template_expression(&arg) {
            parts.push(tsox_frontend::scanner::mig::m3i::get_text_of_node(&arg));
        }
    }
    parts.join(", ")
}

pub fn clean_callback_text(text: &str) -> String {
    let truncated = tsox_core::stringutil::mig::m3m_2::truncate_by_runes(text, M5X_MAX_LENGTH);
    let mut text = text.to_string();
    if truncated.len() < text.len() {
        text = format!("{}...", truncated);
    }
    text.chars()
        .filter(|r| !tsox_core::stringutil::is_line_break(*r))
        .collect()
}

pub fn get_interior_module(node: &Arc<Node>) -> Arc<Node> {
    let mut current = Arc::clone(node);
    while let Some(body) = crate::ls::mig::m5x_3::module_declaration_body(&current) {
        if !ast::is_module_declaration(&body) {
            break;
        }
        current = body;
    }
    current
}

pub fn should_exclude_file(
    file: &Arc<SourceFile>,
    program: &Arc<tsox_compile::compiler::Program>,
    exclude_library_symbols: bool,
) -> bool {
    exclude_library_symbols
        && (is_inside_node_modules(&file.file_name) || program.is_lib_file(file))
}

pub fn is_inside_node_modules(file_name: &str) -> bool {
    file_name.contains("/node_modules/")
}

pub fn get_match_score(s: &str, pattern: &str) -> i64 {
    let mut score: i64 = 0;
    let mut chars = s.char_indices().peekable();
    let mut rest = s;
    for p in pattern.chars() {
        let exact = p.is_uppercase();
        loop {
            let Some(c) = rest.chars().next() else {
                return -1;
            };
            rest = &rest[c.len_utf8()..];
            if (exact && c == p) || (!exact && c.to_lowercase().eq(p.to_lowercase())) {
                break;
            }
            score += 1;
        }
    }
    score
}

pub fn compare_declaration_infos(d1: &M5xDeclarationInfo, d2: &M5xDeclarationInfo) -> std::cmp::Ordering {
    if d1.match_score != d2.match_score {
        return d1.match_score.cmp(&d2.match_score);
    }
    let c = tsox_core::stringutil::mig::m3m::compare_strings_case_insensitive(&d1.name, &d2.name);
    if c != tsox_core::stringutil::mig::m3m::COMPARISON_EQUAL {
        return match c {
            x if x < 0 => std::cmp::Ordering::Less,
            _ => std::cmp::Ordering::Greater,
        };
    }
    let c = d1.name.cmp(&d2.name);
    if c != std::cmp::Ordering::Equal {
        return c;
    }
    let s1 = ast::get_source_file_of_node(&d1.declaration);
    let s2 = ast::get_source_file_of_node(&d2.declaration);
    match (&s1, &s2) {
        (Some(s1), Some(s2)) if !Arc::ptr_eq(s1, s2) => s1.id().cmp(&s2.id()),
        _ => d1.declaration.pos().cmp(&d2.declaration.pos()),
    }
}
