#![allow(dead_code, unused_imports, unused_variables)]

use std::sync::Arc;

use crate::lsp::lsproto;
use crate::ls::lsutil_user_preferences::{QuotePreference, UserPreferences};
use tsox_checker::checker::Checker;
use tsox_core::core;
use tsox_frontend::ast::{self, Node, SourceFile, Symbol, SyntaxKind};
use tsox_frontend::ast::mig::m3b_2::NodeFactory;
use tsox_frontend::astnav;
use tsox_frontend::scanner;

use super::m5q2::{ImportStatementCompletionInfo, get_jsdoc_param_name_with_initializer, generate_jsdoc_param_tags_for_destructuring, SORT_TEXT_GLOBALS_OR_KEYWORDS};
use super::m5q_3::{create_snippet_printer, escape_snippet_text, get_completions_symbol_kind, get_dot_accessor, client_supports_item_snippet, SnippetPrinter, SORT_TEXT_LOCATION_PRIORITY};
use super::m5r::{get_dot_accessor as get_dot_accessor_m5r, get_word_length_and_start, is_module_specifier_missing_or_empty, get_potentially_invalid_import_specifier, is_non_contextual_keyword, str_ptr_to, COMPLETION_SOURCE_SWITCH_CASES};
use super::m5q2::get_filter_text;
use super::m5q2b::PresentMemberModifiers;

pub fn get_insert_text_and_replacement_span_for_import_completion(
    fix: &crate::ls::autoimport_fix::Fix,
    import_kind: crate::ls::autoimport::ImportKind,
    import_statement_completion: &ImportStatementCompletionInfo,
    use_semicolons: bool,
    file: &Arc<SourceFile>,
    preferences: &UserPreferences,
    is_snippet: bool,
) -> (String, Option<lsproto::Range>) { ::tsox_core::fntrace::enter("get_insert_text_and_replacement_span_for_import_completion"); 
    let quoted_module_specifier = escape_snippet_text(&super::m5x_3::quote(
        file,
        preferences,
        &fix.auto_import_fix.module_specifier,
    ));
    let tab_stop = if is_snippet { "$1" } else { "" };
    let suffix = if use_semicolons { ";" } else { "" };
    let top_level_type_only_text = if import_statement_completion.is_top_level_type_only {
        format!("{} ", scanner::token_to_string(SyntaxKind::TypeKeyword))
    } else {
        " ".to_string()
    };
    let name = escape_snippet_text(&fix.auto_import_fix.name);
    let replacement_span = import_statement_completion.replacement_span.clone();

    match import_kind {
        crate::ls::autoimport::ImportKind::CommonJS => (
            format!(
                "import{}{}{} = require({}){}",
                top_level_type_only_text, name, tab_stop, quoted_module_specifier, suffix
            ),
            replacement_span,
        ),
        crate::ls::autoimport::ImportKind::Default => (
            format!(
                "import{}{}{} from {}{}",
                top_level_type_only_text, name, tab_stop, quoted_module_specifier, suffix
            ),
            replacement_span,
        ),
        crate::ls::autoimport::ImportKind::Namespace => (
            format!(
                "import{}* as {} from {}{}",
                top_level_type_only_text, name, quoted_module_specifier, suffix
            ),
            replacement_span,
        ),
        crate::ls::autoimport::ImportKind::Named => {
            let type_only = if import_statement_completion.could_be_type_only_import_specifier {
                format!("{} ", scanner::token_to_string(SyntaxKind::TypeKeyword))
            } else {
                String::new()
            };
            (
                format!(
                    "import{}{{ {}{}{} }} from {}{}",
                    top_level_type_only_text, type_only, name, tab_stop, quoted_module_specifier, suffix
                ),
                replacement_span,
            )
        }
    }
}

pub fn get_line_of_position_m5q2b(file: &Arc<SourceFile>, pos: usize) -> usize { ::tsox_core::fntrace::enter("get_line_of_position_m5q2b"); 
    file.line_map.line_at(pos)
}

pub fn lsutil_script_element_kind_modifier_flags(kind_modifiers: &[String]) -> u32 { ::tsox_core::fntrace::enter("lsutil_script_element_kind_modifier_flags"); 
    let mut flags = 0;
    for modifier in kind_modifiers {
        match modifier.as_str() {
            "optional" => flags |= LSUTIL_SCRIPT_ELEMENT_KIND_MODIFIER_OPTIONAL,
            "deprecated" | "deprecation" => flags |= LSUTIL_SCRIPT_ELEMENT_KIND_MODIFIER_DEPRECATED,
            _ => {}
        }
    }
    flags
}

pub const LSUTIL_SCRIPT_ELEMENT_KIND_MODIFIER_OPTIONAL: u32 = 1 << 0;
pub const LSUTIL_SCRIPT_ELEMENT_KIND_MODIFIER_DEPRECATED: u32 = 1 << 1;

pub fn supplemental_file_index_m5q2b(file: &Arc<SourceFile>) -> Option<i32> { ::tsox_core::fntrace::enter("supplemental_file_index_m5q2b"); 
    super::m5r::supplemental_file_index(file)
}

pub fn could_be_type_only_import_specifier_m5q2b(import_specifier: &Arc<Node>, context_token: &Arc<Node>) -> bool { ::tsox_core::fntrace::enter("could_be_type_only_import_specifier_m5q2b"); 
    ast::is_import_specifier(import_specifier)
        && (import_specifier_is_type_only_m5q2b(import_specifier)
            || (import_specifier
                .name()
                .is_some_and(|n| Arc::ptr_eq(n, context_token))
                && is_type_keyword_token_or_identifier_m5q2b(context_token)))
}

fn import_specifier_is_type_only_m5q2b(node: &Arc<Node>) -> bool { ::tsox_core::fntrace::enter("import_specifier_is_type_only_m5q2b"); 
    match &node.data {
        ast::node_data_generated::NodeData::ImportSpecifier(d) => d.is_type_only,
        _ => false,
    }
}

fn is_type_keyword_token_or_identifier_m5q2b(node: &Arc<Node>) -> bool { ::tsox_core::fntrace::enter("is_type_keyword_token_or_identifier_m5q2b"); 
    ast::mig::m3g_2::is_type_keyword_token(node)
        || ast::is_identifier(node)
            && scanner::mig::m3i::identifier_to_keyword_kind(node) == SyntaxKind::TypeKeyword
}

pub fn can_complete_from_named_bindings(named_bindings: &Arc<Node>) -> bool { ::tsox_core::fntrace::enter("can_complete_from_named_bindings"); 
    let import_declaration = named_bindings.parent().and_then(|p| p.parent());
    let module_specifier_missing = import_declaration
        .as_ref()
        .and_then(|d| ast::mig::m3b::module_specifier(d))
        .is_some_and(|m| is_module_specifier_missing_or_empty(Some(m)));
    if !module_specifier_missing
        || import_declaration.as_ref().and_then(|d| d.name()).is_some()
    {
        return false;
    }
    if ast::is_named_imports(named_bindings) {
        let invalid_named_import =
            get_potentially_invalid_import_specifier_m5q2b(Some(named_bindings));
        let elements: Vec<Arc<Node>> = match &named_bindings.data {
            ast::node_data_generated::NodeData::NamedImports(d) => d.elements.nodes.clone(),
            _ => Vec::new(),
        };
        let mut valid_imports = elements.len();
        if let Some(invalid) = &invalid_named_import {
            if let Some(index) = elements.iter().position(|e| Arc::ptr_eq(e, invalid)) {
                valid_imports = index;
            }
        }
        valid_imports < 2 && valid_imports > 0 || valid_imports == 0 && elements.is_empty()
    } else {
        true
    }
}

pub fn get_potentially_invalid_import_specifier_m5q2b(
    named_bindings: Option<&Arc<Node>>,
) -> Option<Arc<Node>> { ::tsox_core::fntrace::enter("get_potentially_invalid_import_specifier_m5q2b"); 
    let named_bindings = named_bindings?;
    if named_bindings.kind != SyntaxKind::NamedImports {
        return None;
    }
    let elements: Vec<Arc<Node>> = match &named_bindings.data {
        ast::node_data_generated::NodeData::NamedImports(d) => d.elements.nodes.clone(),
        _ => Vec::new(),
    };
    elements
        .iter()
        .find(|e| match &e.data {
            ast::node_data_generated::NodeData::ImportSpecifier(d) => {
                d.property_name.is_none()
                    && is_non_contextual_keyword(scanner::string_to_token(&d.name.text()))
            }
            _ => false,
        })
        .cloned()
}

