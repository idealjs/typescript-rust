#![allow(dead_code, unused_imports, unused_variables)]

use std::collections::{HashMap, HashSet};
use std::sync::Arc;

use super::m5q2b_3::lsproto;
use crate::ls::lsutil_user_preferences::{QuotePreference, UserPreferences};
use tsox_checker::checker::Checker;
use tsox_core::core;
use tsox_frontend::ast::{self, Node, SourceFile, Symbol};
use tsox_frontend::ast::mig::m3b_2::NodeFactory;
use tsox_frontend::ast::mig::{m3f, m3f_4, m3g_2, m3g_3};
use tsox_frontend::astnav;
use tsox_frontend::scanner;

pub use super::m5r::is_interface_or_type_literal_completion_keyword;

use super::m5r::{
    is_checked_file, is_deprecated, is_in_type_parameter_default,
    is_possibly_type_argument_position, is_snippet_scope, symbol_name, origin_includes_symbol_name,
    COMPLETION_SOURCE_CLASS_MEMBER_SNIPPET, COMPLETION_SOURCE_OBJECT_LITERAL_MEMBER_WITH_COMMA,
    COMPLETION_SOURCE_OBJECT_LITERAL_METHOD_SNIPPET, COMPLETION_SOURCE_SWITCH_CASES,
};
use super::m5q_3::{
    client_supports_default_commit_characters, client_supports_default_edit_range,
    client_supports_item_commit_characters, client_supports_item_insert_replace, get_default_commit_characters,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum KeywordCompletionFilters {
    None,
    All,
    ClassElementKeywords,
    InterfaceElementKeywords,
    ConstructorParameterKeywords,
    FunctionLikeBodyKeywords,
    TypeAssertionKeywords,
    TypeKeywords,
    TypeKeyword,
}

pub fn keyword_filters_from_syntax_kind(keyword_completion: ast::SyntaxKind) -> KeywordCompletionFilters {
    match keyword_completion {
        ast::SyntaxKind::TypeKeyword => KeywordCompletionFilters::TypeKeyword,
        _ => panic!(
            "Unknown mapping from ast.Kind `{:?}` to KeywordCompletionFilters",
            keyword_completion
        ),
    }
}

pub struct CompletionDataKeyword {
    pub keyword_completions: Vec<lsproto::CompletionItem>,
    pub is_new_identifier_location: bool,
}

pub fn keyword_completion_data(
    keyword_filters: KeywordCompletionFilters,
    filter_out_ts_only_keywords: bool,
    is_new_identifier_location: bool,
) -> CompletionDataKeyword {
    CompletionDataKeyword {
        keyword_completions: get_keyword_completions(keyword_filters, filter_out_ts_only_keywords),
        is_new_identifier_location,
    }
}

pub fn get_typescript_keyword_completions(keyword_filter: KeywordCompletionFilters) -> Vec<lsproto::CompletionItem> {
    all_keyword_completions()
        .into_iter()
        .filter(|entry| {
            let kind = scanner::string_to_token(&entry.label);
            match keyword_filter {
                KeywordCompletionFilters::None => false,
                KeywordCompletionFilters::All => match kind {
                    Some(kind) => {
                        super::m5r::is_function_like_body_keyword(kind)
                            || kind == ast::SyntaxKind::DeclareKeyword
                            || kind == ast::SyntaxKind::ModuleKeyword
                            || kind == ast::SyntaxKind::TypeKeyword
                            || kind == ast::SyntaxKind::NamespaceKeyword
                            || kind == ast::SyntaxKind::AbstractKeyword
                            || super::m5x_3::is_type_keyword(kind) && kind != ast::SyntaxKind::UndefinedKeyword
                    }
                    None => false,
                },
                KeywordCompletionFilters::FunctionLikeBodyKeywords => {
                    kind.map_or(false, |k| super::m5r::is_function_like_body_keyword(k))
                }
                KeywordCompletionFilters::ClassElementKeywords => {
                    kind.map_or(false, super::m5r::is_class_member_completion_keyword)
                }
                KeywordCompletionFilters::InterfaceElementKeywords => {
                    kind.map_or(false, is_interface_or_type_literal_completion_keyword)
                }
                KeywordCompletionFilters::ConstructorParameterKeywords => {
                    kind.map_or(false, m3g_2::is_parameter_property_modifier)
                }
                KeywordCompletionFilters::TypeAssertionKeywords => {
                    kind.map_or(false, |k| super::m5x_3::is_type_keyword(k) || k == ast::SyntaxKind::ConstKeyword)
                }
                KeywordCompletionFilters::TypeKeywords => kind.map_or(false, super::m5x_3::is_type_keyword),
                KeywordCompletionFilters::TypeKeyword => kind == Some(ast::SyntaxKind::TypeKeyword),
            }
        })
        .collect()
}

/// Go ast.Kind(i)：SyntaxKind 为 #[repr(i16)] 连续枚举，
/// 仅用于关键字区间(BreakKeyword..=DeferKeyword 均为 token kind)
fn syntax_kind_from_i16(kind: i16) -> ast::SyntaxKind {
    debug_assert!(kind >= ast::SyntaxKind::BreakKeyword as i16 && kind <= ast::SyntaxKind::DeferKeyword as i16);
    unsafe { std::mem::transmute::<i16, ast::SyntaxKind>(kind) }
}

fn all_keyword_completions() -> Vec<lsproto::CompletionItem> {
    let mut result = Vec::new();
    let mut kind = ast::SyntaxKind::BreakKeyword as i16;
    while kind <= ast::SyntaxKind::DeferKeyword as i16 {
        let keyword = syntax_kind_from_i16(kind);
        result.push(lsproto::CompletionItem {
            label: scanner::token_to_string(keyword).to_string(),
            kind: Some(lsproto::CompletionItemKind::Keyword),
            sort_text: Some(super::m5q_3::SORT_TEXT_GLOBALS_OR_KEYWORDS.to_string()),
            ..Default::default()
        });
        kind += 1;
    }
    result
}

fn get_keyword_completions(
    keyword_filter: KeywordCompletionFilters,
    filter_out_ts_only_keywords: bool,
) -> Vec<lsproto::CompletionItem> {
    if !filter_out_ts_only_keywords {
        return get_typescript_keyword_completions(keyword_filter);
    }
    get_typescript_keyword_completions(keyword_filter)
        .into_iter()
        .filter(|ci| {
            !super::m5r::is_type_script_only_keyword(
                scanner::string_to_token(&ci.label).unwrap_or(ast::SyntaxKind::Unknown),
            )
        })
        .collect()
}

pub fn get_scope_node(initial_token: Option<&Arc<Node>>, position: usize, file: &Arc<SourceFile>) -> Option<Arc<Node>> {
    let mut scope = initial_token.cloned();
    while let Some(node) = scope {
        if crate::ls::utilities::position_belongs_to_node(&node, position, file) {
            return Some(node);
        }
        scope = node.parent();
    }
    None
}

pub fn is_probably_global_type(
    t: &Arc<tsox_checker::checker::Type>,
    file: &Arc<SourceFile>,
    type_checker: &mut Checker,
) -> bool {
    let file_node = &file.node;
    for name in ["self", "global", "globalThis"] {
        if let Some(symbol) = type_checker.get_global_symbol(name, ast::SymbolFlags::VALUE, None) {
            let ty = type_checker.get_type_of_symbol_at_location(&symbol, file_node);
            if Arc::ptr_eq(&ty, t) {
                return true;
            }
        }
    }
    false
}

pub fn try_get_type_literal_node(node: Option<&Arc<Node>>) -> Option<Arc<Node>> {
    let node = node?;
    let parent = node.parent()?;
    use ast::SyntaxKind as K;
    match node.kind {
        K::OpenBraceToken => {
            if ast::is_type_literal_node(&parent) {
                Some(parent)
            } else {
                None
            }
        }
        K::SemicolonToken | K::CommaToken | K::Identifier => {
            if parent.kind == K::PropertySignature && parent.parent().map_or(false, |pp| ast::is_type_literal_node(&pp))
            {
                parent.parent()
            } else {
                None
            }
        }
        _ => None,
    }
}

pub fn get_switched_type(case_clause: &Arc<Node>, type_checker: &mut Checker) -> Option<Arc<tsox_checker::checker::Type>> {
    let expr = case_clause
        .parent()?
        .parent()?
        .expression()?
        .clone();
    Some(type_checker.get_type_at_location(&expr))
}

pub fn get_recommended_completion(
    previous_token: &Arc<Node>,
    contextual_type: &Arc<tsox_checker::checker::Type>,
    type_checker: &mut Checker,
) -> Option<Arc<Symbol>> {
    let types: Vec<Arc<tsox_checker::checker::Type>> = if contextual_type.is_union() {
        contextual_type.types().map(|t| t.to_vec()).unwrap_or_default()
    } else {
        vec![Arc::clone(contextual_type)]
    };
    for t in &types {
        let Some(symbol) = t.symbol() else { continue };
        if symbol
            .flags
            .intersects(ast::SymbolFlags::EnumMember | ast::SymbolFlags::ENUM | ast::SymbolFlags::Class)
            && !super::m5r::is_abstract_constructor_symbol(symbol)
        {
            return get_first_symbol_in_chain(symbol, previous_token, type_checker);
        }
    }
    None
}

pub fn get_first_symbol_in_chain(
    symbol: &Arc<Symbol>,
    enclosing_declaration: &Arc<Node>,
    type_checker: &mut Checker,
) -> Option<Arc<Symbol>> {
    let chain = type_checker.get_accessible_symbol_chain_public(
        symbol,
        Some(enclosing_declaration),
        ast::SymbolFlags::all(),
        false,
    );
    if !chain.is_empty() {
        return Some(chain[0].clone());
    }
    if let Some(parent) = symbol.parent() {
        if is_module_symbol(&parent) {
            return Some(symbol.clone());
        }
        return get_first_symbol_in_chain(&parent, enclosing_declaration, type_checker);
    }
    None
}

pub fn is_module_symbol(symbol: &Arc<Symbol>) -> bool {
    symbol
        .declarations
        .iter()
        .any(|decl| decl.kind == ast::SyntaxKind::SourceFile)
}

pub fn is_class_like_member_completion(
    symbol: &Arc<Symbol>,
    location: &Arc<Node>,
    file: &Arc<SourceFile>,
) -> bool {
    if ast::is_in_js_file(location) {
        return false;
    }
    let member_flags = ast::SymbolFlags::CLASS_MEMBER & ast::SymbolFlags::EnumMemberExcludes;
    let location_ok = ast::is_class_like(location)
        || location.parent().map_or(false, |parent| {
            parent.parent().map_or(false, |grand| {
                ast::is_class_element(&parent)
                    && parent.name().map_or(false, |name| Arc::ptr_eq(location, name))
                    && crate::ls::lsutil_children::get_last_token(Some(&parent), file)
                        .zip(parent.name())
                        .map_or(false, |(last_token, name)| Arc::ptr_eq(&last_token, name))
                    && ast::is_class_like(&grand)
            })
        })
        || location.parent().map_or(false, |parent| {
            ast::is_syntax_list(location) && ast::is_class_like(&parent)
        });
    symbol.flags.intersects(member_flags) && location_ok
}

pub fn symbol_appears_to_be_type_only(symbol: &Arc<Symbol>, type_checker: &mut Checker) -> bool {
    let flags = type_checker.skip_alias(symbol).combined_local_and_export_symbol_flags();
    !flags.intersects(ast::SymbolFlags::VALUE)
        && (symbol.declarations.is_empty()
            || !ast::is_in_js_file(&symbol.declarations[0])
            || flags.intersects(ast::SymbolFlags::TYPE))
}

pub fn should_include_symbol(
    symbol: &Arc<Symbol>,
    data: &super::m5q2::CompletionDataData,
    location: Option<&Arc<Node>>,
    closest_symbol_declaration: Option<&Arc<Node>>,
    context_token: Option<&Arc<Node>>,
    file: &Arc<SourceFile>,
    type_checker: &mut Checker,
    compiler_options: &core::compiler_options::CompilerOptions,
) -> bool {
    let mut all_flags = symbol.flags;
    let location = location.expect("data.location");
    if location.parent().map_or(false, |p| ast::is_export_assignment(&p)) {
        return true;
    }

    if let Some(closest) = closest_symbol_declaration {
        if ast::is_variable_declaration(closest)
            && symbol
                .value_declaration
                .as_ref()
                .is_some_and(|vd| Arc::ptr_eq(vd, closest))
        {
            return false;
        }
    }

    let symbol_declaration: Option<Arc<Node>> = symbol
        .value_declaration
        .clone()
        .or_else(|| symbol.declarations.first().cloned());

    if let (Some(closest), Some(symbol_declaration)) = (closest_symbol_declaration, symbol_declaration) {
        if ast::is_parameter_declaration(closest) && ast::is_parameter_declaration(&symbol_declaration) {
            let parent = closest.parent();
            let parameters = parent.as_ref().and_then(|p| p.parameters());
            if let Some(parameters) = parameters {
                if symbol_declaration.pos() >= closest.pos() && symbol_declaration.pos() < parameters.end() {
                    return false;
                }
            }
        } else if ast::is_type_parameter_declaration(closest)
            && ast::is_type_parameter_declaration(&symbol_declaration)
        {
            if Arc::ptr_eq(closest, &symbol_declaration)
                && context_token.map_or(false, |t| t.kind == ast::SyntaxKind::ExtendsKeyword)
            {
                return false;
            }
            if context_token.map_or(false, |t| is_in_type_parameter_default(t))
                && !closest
                    .parent()
                    .map_or(false, |p| ast::is_infer_type_node(&p))
            {
                let parent = closest.parent();
                let type_parameters = parent.as_ref().and_then(|p| p.type_parameters());
                if let Some(type_parameters) = type_parameters {
                    if symbol_declaration.pos() >= closest.pos()
                        && symbol_declaration.pos() < type_parameters.end()
                    {
                        return false;
                    }
                }
            }
        }
    }

    let symbol_origin = type_checker.skip_alias(symbol);
    if file.external_module_indicator.is_some()
        && !compiler_options.allow_umd_global_access.is_true()
        && !Arc::ptr_eq(symbol, &symbol_origin)
        && data.symbol_to_sort_text_map
            .get(&m3f::get_symbol_id(symbol))
            .map_or(false, |s| s == super::m5q_3::SORT_TEXT_GLOBALS_OR_KEYWORDS)
        && symbol
            .parent()
            .map_or(false, |p| tsox_checker::checker::is_external_module_symbol(&p))
    {
        return false;
    }

    all_flags = all_flags | symbol_origin.combined_local_and_export_symbol_flags();
    if symbol.flags.intersects(ast::SymbolFlags::Alias) {
        all_flags = all_flags | type_checker.get_symbol_flags(symbol);
    }

    if crate::ls::mig::m5x_4::is_in_right_side_of_internal_import_equals_declaration(location) {
        return all_flags.intersects(ast::SymbolFlags::NAMESPACE);
    }

    if data.is_type_only_location {
        return symbol_can_be_referenced_at_type_location(symbol, type_checker, &mut HashSet::new());
    }

    all_flags.intersects(ast::SymbolFlags::VALUE)
}

/// Go symbolCanBeReferencedAtTypeLocation：symbol 是类型，或包含至少一个类型的模块
fn symbol_can_be_referenced_at_type_location(
    symbol: &Arc<Symbol>,
    type_checker: &mut Checker,
    seen_modules: &mut HashSet<u64>,
) -> bool {
    non_alias_can_be_referenced_at_type_location(symbol, type_checker, seen_modules)
        || {
            let export_symbol = symbol
                .export_symbol
                .clone()
                .unwrap_or_else(|| Arc::clone(symbol));
            let skipped = type_checker.skip_alias(&export_symbol);
            non_alias_can_be_referenced_at_type_location(&skipped, type_checker, seen_modules)
        }
}

fn non_alias_can_be_referenced_at_type_location(
    symbol: &Arc<Symbol>,
    type_checker: &mut Checker,
    seen_modules: &mut HashSet<u64>,
) -> bool {
    symbol.flags.intersects(ast::SymbolFlags::TYPE)
        || type_checker.is_unknown_symbol(symbol)
        || (symbol.flags.intersects(ast::SymbolFlags::MODULE)
            && seen_modules.insert(m3f::get_symbol_id(symbol))
            && type_checker
                .get_exports_of_module(symbol)
                .iter()
                .any(|e| symbol_can_be_referenced_at_type_location(e, type_checker, seen_modules)))
}

pub fn is_valid_trigger(
    file: &Arc<SourceFile>,
    trigger_character: &str,
    context_token: Option<&Arc<Node>>,
    position: usize,
) -> bool {
    use ast::SyntaxKind as K;
    match trigger_character {
        "." | "@" => true,
        "\"" | "'" | "`" => match context_token {
            Some(context_token) => {
                super::m5r::is_string_literal_or_template(&context_token)
                    && position == astnav::get_start_of_node(context_token, file, false) + 1
            }
            None => false,
        },
        "#" => match context_token {
            Some(context_token) => {
                ast::is_private_identifier(&context_token) && ast::get_containing_class(&context_token).is_some()
            }
            None => false,
        },
        "<" => match context_token {
            Some(context_token) => {
                context_token.kind == K::LessThanToken
                    && (!context_token.parent().map_or(false, |p| ast::is_binary_expression(&p))
                        || super::m5q_3::binary_expression_may_be_open_tag(&context_token.parent().unwrap()))
            }
            None => false,
        },
        "/" => match context_token {
            None => false,
            Some(context_token) => {
                if ast::is_string_literal_like(&context_token) {
                    m3g_3::try_get_import_from_module_specifier(&context_token).is_some()
                } else {
                    context_token.kind == K::LessThanSlashToken
                        && context_token
                            .parent()
                            .map_or(false, |p| ast::is_jsx_closing_element(&p))
                }
            }
        },
        " " => context_token
            .map_or(false, |t| {
                t.kind == K::ImportKeyword
                    && t.parent().map_or(false, |p| p.kind == K::SourceFile)
            }),
        "*" => crate::ls::jsdoc_snippet::is_potentially_valid_jsdoc_snippet_completion_position(file, position),
        _ => panic!("Unknown trigger character: {}", trigger_character),
    }
}

pub fn try_get_function_like_body_completion_container(context_token: Option<&Arc<Node>>) -> Option<Arc<Node>> {
    let context_token = context_token?;
    let mut prev: Option<Arc<Node>> = None;
    let mut scope = Some(context_token.clone());
    while let Some(node) = scope {
        if ast::is_class_like(&node) {
            return None;
        }
        if ast::is_function_like_declaration(&node)
            && prev
                .as_ref()
                .is_some_and(|p| node.body().is_some_and(|b| Arc::ptr_eq(p, b)))
        {
            return Some(node);
        }
        prev = Some(node.clone());
        scope = node.parent();
    }
    None
}

pub fn try_get_constructor_like_completion_container(context_token: Option<&Arc<Node>>) -> Option<Arc<Node>> {
    let context_token = context_token?;
    let parent = context_token.parent()?;
    use ast::SyntaxKind as K;
    match context_token.kind {
        K::OpenParenToken | K::CommaToken => {
            if ast::is_constructor_declaration(&parent) {
                Some(parent)
            } else {
                None
            }
        }
        _ => {
            if is_constructor_parameter_completion(&context_token) {
                parent.parent()
            } else {
                None
            }
        }
    }
}

pub fn is_constructor_parameter_completion(node: &Arc<Node>) -> bool {
    node.parent().map_or(false, |parent| {
        parent.parent().map_or(false, |grand| {
            ast::is_parameter_declaration(&parent)
                && ast::is_constructor_declaration(&grand)
                && (m3g_2::is_parameter_property_modifier(node.kind) || m3f_4::is_declaration_name(node))
        })
    })
}

pub fn is_from_object_type_declaration(node: &Arc<Node>) -> bool {
    node.parent().map_or(false, |parent| {
        parent.parent().map_or(false, |grand| {
            m3f_4::is_class_or_type_element(&parent) && m3g_2::is_object_type_declaration(&grand)
        })
    })
}

pub fn try_get_containing_jsx_element(context_token: Option<&Arc<Node>>, file: &Arc<SourceFile>) -> Option<Arc<Node>> {
    let context_token = context_token?;
    let parent = context_token.parent()?;
    use ast::SyntaxKind as K;
    match context_token.kind {
        K::GreaterThanToken | K::LessThanSlashToken | K::SlashToken | K::Identifier
        | K::PropertyAccessExpression | K::JsxNamespacedName | K::JsxAttributes | K::JsxAttribute
        | K::JsxSpreadAttribute => {
            if parent.kind == K::JsxSelfClosingElement || parent.kind == K::JsxOpeningElement {
                if context_token.kind == K::GreaterThanToken {
                    let preceding_token = astnav::find_preceding_token(&file.node, context_token.pos());
                    if parent.type_arguments().map_or(true, |t| t.is_empty())
                        || preceding_token.map_or(false, |t| t.kind == K::SlashToken)
                    {
                        return None;
                    }
                }
                Some(parent)
            } else if ast::is_jsx_namespaced_name(&parent) {
                parent.parent().filter(|pp| {
                    pp.kind == K::JsxSelfClosingElement || pp.kind == K::JsxOpeningElement
                })
            } else if parent.kind == K::JsxAttribute {
                parent.parent().and_then(|pp| pp.parent())
            } else {
                None
            }
        }
        K::StringLiteral => {
            if parent.kind == K::JsxAttribute || parent.kind == K::JsxSpreadAttribute {
                parent.parent().and_then(|pp| pp.parent())
            } else {
                None
            }
        }
        K::CloseBraceToken => {
            if parent.kind == K::JsxExpression {
                return parent
                    .parent()
                    .filter(|pp| pp.kind == K::JsxAttribute)
                    .and_then(|pp| pp.parent())
                    .and_then(|ppp| ppp.parent());
            }
            if parent.kind == K::JsxSpreadAttribute {
                return parent.parent().and_then(|pp| pp.parent());
            }
            None
        }
        _ => None,
    }
}
