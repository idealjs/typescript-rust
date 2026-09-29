#![allow(dead_code, unused_imports, unused_variables)]

use std::sync::Arc;

use tsox_core::core::text::TextRange;
use tsox_frontend::ast::{self, Node, SourceFile, SyntaxKind};

use crate::lsp::lsproto_lsp::Range;

pub struct M5xSourceFileScript {
    pub file: Arc<SourceFile>,
}

impl crate::ls::lsconv_converters::Script for M5xSourceFileScript {
    fn file_name(&self) -> &str {
        &self.file.file_name
    }
    fn text(&self) -> &str {
        &self.file.text
    }
}

pub struct M5xCompletionsFromTypes {
    pub types: Vec<StringLiteralTypeRef>,
    pub is_new_identifier: bool,
}

pub struct M5xCompletionsFromProperties {
    pub symbols: Vec<SymbolRef>,
    pub has_index_signature: bool,
}

pub struct M5xPathCompletion {
    pub name: String,
    pub kind: crate::ls::lsutil_symbol_display::ScriptElementKind,
    pub extension: String,
}

pub struct M5xPathCompletions {
    pub entries: Vec<M5xPathCompletion>,
    pub replacement_span: Option<Range>,
}

pub struct M5xStringLiteralCompletions {
    pub from_types: Option<M5xCompletionsFromTypes>,
    pub from_properties: Option<M5xCompletionsFromProperties>,
    pub from_paths: Option<M5xPathCompletions>,
}

pub type StringLiteralTypeRef = Arc<tsox_checker::checker::types::Type>;
pub type SymbolRef = Arc<tsox_frontend::ast::Symbol>;

impl crate::ls::language_service::LanguageService {
    pub fn path_completion_replacement_span(
        &self,
        file: &Arc<SourceFile>,
        text_range: Option<TextRange>,
    ) -> Option<Range> {
        let text_range = text_range?;
        let (lsp_range, fidelity) = self.m5x_create_lsp_range_from_bounds(
            text_range.pos as usize,
            text_range.end as usize,
            file,
        );
        if !crate::ls::mig::m5u_3::fidelity_is_exact(&fidelity) {
            return None;
        }
        Some(lsp_range)
    }

}

pub fn remove_leading_directory_separator(path: &str) -> String {
    path.strip_prefix('/').unwrap_or(path).to_string()
}

fn has_index_signature(t: &Arc<tsox_checker::checker::Type>, type_checker: &mut tsox_checker::checker::Checker) -> bool {
    type_checker.get_string_index_type(t).is_some() || type_checker.get_number_index_type(t).is_some()
}

pub fn string_literal_completions_for_object_literal(
    type_checker: &mut tsox_checker::checker::Checker,
    object_literal_expression: &Arc<Node>,
) -> Option<M5xCompletionsFromProperties> {
    let contextual_type = type_checker
        .get_contextual_type(object_literal_expression, tsox_checker::checker::ContextFlags::None)?;
    let completions_type = type_checker.get_contextual_type(
        object_literal_expression,
        tsox_checker::checker::ContextFlags::IgnoreNodeInferences,
    );
    let symbols = crate::ls::completions_object_like::properties_for_object_expression(
        type_checker,
        &contextual_type,
        completions_type.as_ref(),
        object_literal_expression,
    );
    Some(M5xCompletionsFromProperties {
        symbols,
        has_index_signature: has_index_signature(&contextual_type, type_checker),
    })
}

pub fn string_literal_completions_from_properties(
    t: &Arc<tsox_checker::checker::Type>,
    type_checker: &mut tsox_checker::checker::Checker,
) -> M5xCompletionsFromProperties {
    let symbols: Vec<SymbolRef> = type_checker
        .get_apparent_properties(t)
        .into_iter()
        .filter(|s| {
            s.value_declaration.as_ref().map_or(true, |d| {
                !ast::mig::m3g_2::is_private_identifier_class_element_declaration(d)
            })
        })
        .collect();
    M5xCompletionsFromProperties {
        symbols,
        has_index_signature: has_index_signature(t, type_checker),
    }
}

pub fn to_completions_from_types(types: Vec<StringLiteralTypeRef>) -> Option<M5xCompletionsFromTypes> {
    if types.is_empty() {
        return None;
    }
    Some(M5xCompletionsFromTypes {
        types,
        is_new_identifier: false,
    })
}

pub fn to_string_literal_completions_from_types(
    types: Vec<StringLiteralTypeRef>,
) -> Option<M5xStringLiteralCompletions> {
    let result = to_completions_from_types(types)?;
    Some(M5xStringLiteralCompletions {
        from_types: Some(result),
        from_properties: None,
        from_paths: None,
    })
}

pub fn to_path_completions(
    names: Vec<ModuleCompletionNameAndKind>,
) -> Vec<M5xPathCompletion> {
    names
        .into_iter()
        .map(|name_and_kind| M5xPathCompletion {
            name: name_and_kind.name,
            kind: modulet_to_script_element_kind(name_and_kind.kind),
            extension: name_and_kind.extension,
        })
        .collect()
}

const MODULE_COMPLETION_KIND_DIRECTORY: u32 = 0;
const MODULE_COMPLETION_KIND_FILE: u32 = 1;
const MODULE_COMPLETION_KIND_EXTERNAL_MODULE_NAME: u32 = 2;

fn modulet_to_script_element_kind(
    kind: u32,
) -> crate::ls::lsutil_symbol_display::ScriptElementKind {
    match kind {
        MODULE_COMPLETION_KIND_DIRECTORY => crate::ls::lsutil_symbol_display::ScriptElementKind::Directory,
        MODULE_COMPLETION_KIND_FILE => crate::ls::lsutil_symbol_display::ScriptElementKind::ScriptElement,
        _ => crate::ls::lsutil_symbol_display::ScriptElementKind::ExternalModuleName,
    }
}

pub struct ModuleCompletionNameAndKind {
    pub name: String,
    pub kind: u32,
    pub extension: String,
}

pub fn try_remove_directory_prefix(
    path: &str,
    prefix: &str,
    use_case_sensitive_file_names: bool,
) -> Option<String> {
    let (trimmed, had_prefix) = tsox_core::tspath::mig::m3j::trim_file_path_prefix(
        path,
        prefix,
        use_case_sensitive_file_names,
    );
    if !had_prefix {
        return None;
    }
    let without_prefix = if trimmed.starts_with('/') || trimmed.starts_with('\\') {
        trimmed[1..].to_string()
    } else {
        trimmed
    };
    Some(without_prefix)
}

pub fn walk_up_parentheses(node: &Arc<Node>) -> Arc<Node> {
    match node.kind {
        SyntaxKind::ParenthesizedType => walk_up_parenthesized_types(node),
        SyntaxKind::ParenthesizedExpression => walk_up_parenthesized_expressions(node),
        _ => Arc::clone(node),
    }
}

pub fn walk_up_parenthesized_expressions(node: &Arc<Node>) -> Arc<Node> {
    let mut current = Arc::clone(node);
    while ast::is_parenthesized_expression(&current) {
        let Some(parent) = current.parent() else {
            break;
        };
        current = parent;
    }
    current
}

pub fn walk_up_parenthesized_types(node: &Arc<Node>) -> Arc<Node> {
    let mut current = Arc::clone(node);
    while current.kind == SyntaxKind::ParenthesizedType {
        let Some(parent) = current.parent() else {
            break;
        };
        current = parent;
    }
    current
}

pub fn without_start_and_end(s: &str, start: &str, end: &str) -> Option<String> {
    if s.starts_with(start) && s.ends_with(end) && s.len() >= start.len() + end.len() {
        return Some(s[start.len()..s.len() - end.len()].to_string());
    }
    None
}
