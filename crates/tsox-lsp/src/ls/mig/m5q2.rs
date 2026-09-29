#![allow(dead_code, unused_imports, unused_variables)]

use std::collections::{HashMap, HashSet};
use std::sync::Arc;

use crate::ls::lsutil_user_preferences::UserPreferences;
use tsox_checker::checker::Checker;
use tsox_frontend::ast::{self, Node, SourceFile, Symbol, SyntaxKind};
use tsox_frontend::ast::mig::m3b::get_name_table;
use tsox_frontend::ast::mig::m3b_2::NodeFactory;
use tsox_frontend::ast::mig::{m3b, m3e_3, m3f, m3g_2, m3h};
use tsox_frontend::astnav;

use super::m5q_3::{
    escape_snippet_text, get_dot_accessor, LiteralValue, SORT_TEXT_LOCATION_PRIORITY,
};
use super::m5q2b_6::get_jsdoc_param_annotation;
pub use super::m5q_3::SORT_TEXT_GLOBALS_OR_KEYWORDS;
use super::m5r::{
    is_deprecated, is_named_imports_or_exports, is_recommended_completion_match, is_snippet_scope,
    is_type_script_only_keyword, starts_with_quote, str_ptr_is_empty, str_ptr_to, trim_element_access,
    SymbolOriginInfo, COMPLETION_SOURCE_CLASS_MEMBER_SNIPPET, COMPLETION_SOURCE_OBJECT_LITERAL_METHOD_SNIPPET,
    COMPLETION_SOURCE_OBJECT_LITERAL_MEMBER_WITH_COMMA, COMPLETION_SOURCE_SWITCH_CASES,
    SYMBOL_ORIGIN_INFO_KIND_OBJECT_LITERAL_METHOD,
};
use super::m5r_2::{
    is_interface_or_type_literal_completion_keyword, KeywordCompletionFilters,
};


mod lsproto {
    pub use crate::lsp::lsproto::*;
    pub use crate::mig::m5m::ResolvedClientCapabilitiesContext as Context;

    pub use crate::ls::mig::m5q2b_3::lsproto::CompletionItem;
    pub use crate::ls::mig::m5q2b_3::lsproto::CompletionItemKind;

    #[derive(Debug, Clone, Default, PartialEq, Eq, Hash)]
    pub struct CompletionItemLabelDetails {
        pub detail: Option<String>,
        pub description: Option<String>,
    }

    #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
    pub enum CodeLensKind {
        QuickFix,
        Refactor,
        RefactorExtract,
        Source,
    }

    #[derive(Debug, Clone, Default)]
    pub struct CodeLens {
        pub range: crate::lsp::lsproto::Range,
        pub data: Option<CodeLensKind>,
    }

    #[derive(Debug, Clone)]
    pub enum Documentation {
        String(String),
        MarkupContent(crate::lsp::lsproto::MarkupContent),
    }
}

mod core {
    pub use tsox_core::core::compiler_options::CompilerOptions;
    pub use tsox_core::core::core::{every, some};
    pub use tsox_core::core::mig::m3j_3::get_new_line_kind;
    pub use tsox_frontend::ast::node_source_file::LanguageVariant;

    pub fn stringify_json<T: serde::Serialize>(value: &T) -> String {
        tsox_core::core::mig::m3j::stringify_json(value, "", "").unwrap_or_default()
    }
}

mod scanner {
    pub use tsox_frontend::scanner::mig::m3i::{
        get_text_of_node, identifier_to_keyword_kind, is_identifier_text,
    };
    pub use tsox_frontend::scanner::{string_to_token, token_to_string};

    pub fn get_ecma_line_of_position(file: &tsox_frontend::ast::SourceFile, position: usize) -> usize {
        tsox_emit::mig::m4m_2::get_ecma_line_of_position(file, position)
    }
}

pub const SORT_TEXT_AUTO_IMPORT_SUGGESTIONS: &str = "16";
pub const SORT_TEXT_JAVASCRIPT_IDENTIFIERS: &str = "18";

impl Clone for LiteralValue {
    fn clone(&self) -> Self {
        match self {
            LiteralValue::String(s) => LiteralValue::String(s.clone()),
            LiteralValue::Number(n) => LiteralValue::Number(*n),
            LiteralValue::PseudoBigInt(b) => LiteralValue::PseudoBigInt(b.clone()),
        }
    }
}

impl Default for KeywordCompletionFilters {
    fn default() -> Self {
        KeywordCompletionFilters::None
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum CompletionKind {
    #[default]
    None,
    ObjectPropertyDeclaration,
    Global,
    PropertyAccess,
    MemberLike,
    String,
}

#[derive(Default, Clone)]
pub struct JsxInitializer {
    pub is_initializer: bool,
    pub initializer: Option<Arc<Node>>,
}

#[derive(Default, Clone)]
pub struct ImportStatementCompletionInfo {
    pub is_keyword_only_completion: bool,
    pub keyword_completion: Option<SyntaxKind>,
    pub is_new_identifier_location: bool,
    pub is_top_level_type_only: bool,
    pub could_be_type_only_import_specifier: bool,
    pub replacement_span: Option<lsproto::Range>,
}

#[derive(Clone, Default)]
pub struct CompletionDataData {
    pub symbols: Vec<Arc<Symbol>>,
    pub auto_imports: Vec<crate::ls::autoimport_view::FixAndExport>,
    pub completion_kind: CompletionKind,
    pub is_in_snippet_scope: bool,
    pub property_access_to_convert: Option<Arc<Node>>,
    pub is_new_identifier_location: bool,
    pub location: Option<Arc<Node>>,
    pub keyword_filters: KeywordCompletionFilters,
    pub literals: Vec<super::m5q_3::LiteralValue>,
    pub symbol_to_origin_info_map: HashMap<usize, SymbolOriginInfo>,
    pub symbol_to_sort_text_map: HashMap<u64, String>,
    pub recommended_completion: Option<Arc<Symbol>>,
    pub previous_token: Option<Arc<Node>>,
    pub context_token: Option<Arc<Node>>,
    pub jsx_initializer: JsxInitializer,
    pub inside_jsdoc_tag_type_expression: bool,
    pub is_type_only_location: bool,
    pub is_jsx_identifier_expected: bool,
    pub is_right_of_open_tag: bool,
    pub is_right_of_dot_or_question_dot: bool,
    pub import_statement_completion: Option<ImportStatementCompletionInfo>,
    pub has_unresolved_auto_imports: bool,
    pub default_commit_characters: Vec<String>,
}

#[derive(Debug, Clone, Default)]
pub struct MemberCompletionEntry {
    pub insert_text: String,
    pub filter_text: String,
    pub is_snippet: bool,
    pub additional_text_edits: Vec<lsproto::TextEdit>,
}

pub struct ObjectLiteralMethodSymbol {
    pub symbol: Arc<Symbol>,
    pub origin: SymbolOriginInfo,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct CodeLensKey {
    pub kind: lsproto::CodeLensKind,
    pub start_line: u32,
    pub start_character: u32,
    pub end_line: u32,
    pub end_character: u32,
}

pub fn key_for_code_lens(code_lens: &lsproto::CodeLens) -> CodeLensKey {
    CodeLensKey {
        kind: code_lens.data.unwrap_or(lsproto::CodeLensKind::QuickFix),
        start_line: code_lens.range.start.line,
        start_character: code_lens.range.start.character,
        end_line: code_lens.range.end.line,
        end_character: code_lens.range.end.character,
    }
}

pub fn get_line_of_position(file: &Arc<SourceFile>, pos: usize) -> usize {
    scanner::get_ecma_line_of_position(file, pos)
}

static KEYWORD_COMPLETIONS_CACHE: std::sync::OnceLock<
    std::sync::Mutex<HashMap<u32, Vec<lsproto::CompletionItem>>>,
> = std::sync::OnceLock::new();

fn cache() -> &'static std::sync::Mutex<HashMap<u32, Vec<lsproto::CompletionItem>>> {
    KEYWORD_COMPLETIONS_CACHE.get_or_init(|| std::sync::Mutex::new(HashMap::new()))
}

pub fn get_keyword_completions(
    keyword_filter: KeywordCompletionFilters,
    filter_out_ts_only_keywords: bool,
) -> Vec<lsproto::CompletionItem> {
    if !filter_out_ts_only_keywords {
        return get_typescript_keyword_completions_cached(keyword_filter);
    }
    let index = keyword_filter as u32 + KeywordCompletionFilters::TypeKeyword as u32 + 1;
    let cached = {
        let mut guard = cache().lock().unwrap();
        if !guard.contains_key(&index) {
            let result: Vec<lsproto::CompletionItem> = get_typescript_keyword_completions_cached(keyword_filter)
                .into_iter()
                .filter(|ci| {
                    !is_type_script_only_keyword(scanner::string_to_token(&ci.label).unwrap_or(SyntaxKind::Unknown))
                })
                .collect();
            guard.insert(index, result);
        }
        guard.get(&index).unwrap().clone()
    };
    cached
}

fn get_typescript_keyword_completions_cached(
    keyword_filter: KeywordCompletionFilters,
) -> Vec<lsproto::CompletionItem> {
    {
        let guard = cache().lock().unwrap();
        if let Some(cached) = guard.get(&(keyword_filter as u32)) {
            return cached.clone();
        }
    }
    let result: Vec<lsproto::CompletionItem> = super::m5r_2::get_typescript_keyword_completions(keyword_filter);
    cache().lock().unwrap().insert(keyword_filter as u32, result.clone());
    result
}

pub fn get_contextual_keywords(
    file: &Arc<SourceFile>,
    context_token: Option<&Arc<Node>>,
    position: usize,
) -> Vec<lsproto::CompletionItem> {
    let mut entries = Vec::new();
    let Some(context_token) = context_token else {
        return entries;
    };
    let parent = context_token.parent();
    let token_line = scanner::get_ecma_line_of_position(file, context_token.end());
    let current_line = scanner::get_ecma_line_of_position(file, position);
    let module_specifier = parent.as_ref().and_then(|parent| {
        if ast::is_import_declaration(parent) || ast::is_export_declaration(parent) {
            m3b::module_specifier(parent)
        } else {
            None
        }
    });
    if let Some(module_specifier) = module_specifier {
        if Arc::ptr_eq(module_specifier, context_token) && token_line == current_line {
            let mut item = lsproto::CompletionItem::default();
            item.label = scanner::token_to_string(SyntaxKind::AssertKeyword).to_string();
            item.kind = Some(lsproto::CompletionItemKind::Keyword);
            item.sort_text = Some(SORT_TEXT_GLOBALS_OR_KEYWORDS.to_string());
            entries.push(item);
        }
    }
    entries
}

pub fn get_js_completion_entries(
    file: &Arc<SourceFile>,
    position: usize,
    unique_names: &mut HashSet<String>,
    mut sorted_entries: Vec<lsproto::CompletionItem>,
) -> Vec<lsproto::CompletionItem> {
    let name_table = get_name_table(file);
    for (name, pos) in name_table.iter() {
        if *pos == position as i64 {
            continue;
        }
        if !unique_names.contains(name) && scanner::is_identifier_text(name, core::LanguageVariant::Standard) {
            unique_names.insert(name.clone());
            let mut item = lsproto::CompletionItem::default();
            item.label = name.clone();
            item.kind = Some(lsproto::CompletionItemKind::Text);
            item.sort_text = Some(SORT_TEXT_JAVASCRIPT_IDENTIFIERS.to_string());
            item.commit_characters = Some(Vec::new());
            sorted_entries.push(item);
        }
    }
    sorted_entries
}

pub fn get_filter_text(
    file: &Arc<SourceFile>,
    position: usize,
    insert_text: &str,
    label: &str,
    word_start: char,
    dot_accessor: &str,
) -> String {
    // Private field completion, e.g. label `#bar`.
    if let Some(after) = label.strip_prefix('#') {
        if !insert_text.is_empty() {
            if let Some(after) = insert_text.strip_prefix("this.#") {
                if word_start == '#' {
                    // `method() { this.#| }`
                    // `method() { #| }`
                    return String::new();
                } else {
                    // `method() { this.| }`
                    // `method() { | }`
                    return after.to_string();
                }
            }
        } else {
            if word_start == '#' {
                // `method() { this.#| }`
                return String::new();
            } else {
                // `method() { this.| }`
                // `method() { | }`
                return after.to_string();
            }
        }
    }

    // For `this.` completions, generally don't set the filter text since we don't want them to be overly deprioritized. microsoft/vscode#74164
    if insert_text.starts_with("this.") {
        return String::new();
    }

    // Handle the case: bracket accessor insert text should filter like `.abc`.
    if insert_text.starts_with('[') {
        return format!("{}{}", dot_accessor, trim_element_access(insert_text));
    }

    if let Some(rest) = insert_text.strip_prefix("?.") {
        // filterText should be `.ab c` instead of `?.['ab c']`.
        if rest.starts_with('[') {
            return format!("{}{}", dot_accessor, trim_element_access(rest));
        } else {
            // filterText should be `.abc` instead of `?.abc`.
            return format!("{}{}", dot_accessor, rest);
        }
    }

    // In all other cases, fall back to using the insertText.
    insert_text.to_string()
}

pub fn get_completion_entry_display_name_for_symbol(
    symbol: &Arc<Symbol>,
    origin: Option<&SymbolOriginInfo>,
    completion_kind: CompletionKind,
    is_jsx_identifier_expected: bool,
) -> (String, bool) {
    if super::m5r::origin_is_ignore(origin) {
        return (String::new(), false);
    }

    let name = if super::m5r::origin_includes_symbol_name(origin) {
        super::m5r::symbol_name(origin.unwrap())
    } else {
        m3e_3::symbol_name(symbol)
    };
    if name.is_empty()
        || symbol.flags & ast::SymbolFlags::MODULE != ast::SymbolFlags::None
            && starts_with_quote(&name)
        || tsox_checker::checker::is_known_symbol(symbol)
    {
        return (String::new(), false);
    }

    let variant = if is_jsx_identifier_expected {
        core::LanguageVariant::Jsx
    } else {
        core::LanguageVariant::Standard
    };
    if scanner::is_identifier_text(&name, variant)
        || symbol
            .value_declaration
            .as_ref()
            .map_or(false, |d| m3g_2::is_private_identifier_class_element_declaration(d))
    {
        return (name, false);
    }
    if symbol.flags & ast::SymbolFlags::Alias != ast::SymbolFlags::None {
        // Allow non-identifier import/export aliases since we can insert them as string literals
        return (name, true);
    }

    match completion_kind {
        CompletionKind::MemberLike => {
            if super::m5r::origin_is_computed_property_name(origin) {
                return (super::m5r::symbol_name(origin.unwrap()), false);
            }
            (String::new(), false)
        }
        CompletionKind::ObjectPropertyDeclaration => {
            (core::stringify_json(&name), false)
        }
        CompletionKind::PropertyAccess | CompletionKind::Global => {
            // Don't add a completion for a name starting with a space. See https://github.com/Microsoft/TypeScript/pull/20547
            if name.chars().next() == Some(' ') {
                return (String::new(), false);
            }
            (name, true)
        }
        CompletionKind::None | CompletionKind::String => (name, false),
    }
}

pub fn get_contextual_type(
    previous_token: &Arc<Node>,
    position: usize,
    file: &Arc<SourceFile>,
    type_checker: &mut Checker,
) -> Option<Arc<tsox_checker::checker::types::Type>> {
    use tsox_checker::checker::types::ContextFlags;
    let Some(parent) = previous_token.parent() else {
        return None;
    };
    match previous_token.kind {
        SyntaxKind::Identifier => {
            get_contextual_type_from_parent(previous_token, type_checker, ContextFlags::None)
        }
        SyntaxKind::EqualsToken => match parent.kind {
            SyntaxKind::VariableDeclaration => {
                let initializer = match &parent.data {
                    ast::NodeData::VariableDeclaration(d) => d.initializer.as_ref(),
                    _ => None,
                }?;
                type_checker.get_contextual_type(initializer, ContextFlags::None)
            }
            SyntaxKind::BinaryExpression => match &parent.data {
                ast::NodeData::BinaryExpression(d) => {
                    Some(type_checker.get_type_at_location(&d.left))
                }
                _ => None,
            },
            SyntaxKind::JsxAttribute => {
                type_checker.get_contextual_type_for_jsx_attribute(&parent, ContextFlags::None)
            }
            _ => None,
        },
        SyntaxKind::NewKeyword => type_checker.get_contextual_type(&parent, ContextFlags::None),
        SyntaxKind::CaseKeyword => {
            if ast::is_case_clause(&parent) {
                return super::m5r_2::get_switched_type(&parent, type_checker);
            }
            None
        }
        SyntaxKind::OpenBraceToken => {
            if ast::is_jsx_expression(&parent) {
                let Some(grandparent) = parent.parent() else {
                    return None;
                };
                if !ast::is_jsx_element(&grandparent) && !ast::is_jsx_fragment(&grandparent) {
                    return type_checker
                        .get_contextual_type_for_jsx_attribute(&grandparent, ContextFlags::None);
                }
            }
            None
        }
        SyntaxKind::OpenBracketToken => {
            // When completing after `[` in an array literal (e.g., `[/*here*/]`),
            // we should provide contextual type for the first element
            if ast::is_array_literal_expression(&parent) {
                let contextual_array_type =
                    type_checker.get_contextual_type(&parent, ContextFlags::None)?;
                return type_checker.get_contextual_type_for_array_literal_at_position(
                    Some(&contextual_array_type),
                    &parent,
                    position,
                );
            }
            None
        }
        SyntaxKind::CloseBracketToken => None,
        SyntaxKind::QuestionToken => {
            if ast::is_conditional_expression(&parent) {
                return super::m5q_3::get_contextual_type_for_conditional_expression(
                    &parent,
                    position as i32,
                    file,
                    type_checker,
                );
            }
            None
        }
        SyntaxKind::ColonToken => {
            if ast::is_conditional_expression(&parent) {
                return super::m5q_3::get_contextual_type_for_conditional_expression(
                    &parent,
                    position as i32,
                    file,
                    type_checker,
                );
            }
            None
        }
        SyntaxKind::CommaToken => {
            if ast::is_array_literal_expression(&parent) {
                let contextual_array_type =
                    type_checker.get_contextual_type(&parent, ContextFlags::None)?;
                return type_checker.get_contextual_type_for_array_literal_at_position(
                    Some(&contextual_array_type),
                    &parent,
                    position,
                );
            }
            None
        }
        _ => {
            let arg_info = super::m5q_3::get_argument_info_for_completions(
                previous_token,
                position as i32,
                file,
                type_checker,
            );
            if let Some(arg_info) = arg_info {
                return type_checker.get_contextual_type_for_argument_at_index(
                    &arg_info.invocation,
                    arg_info.argument_index,
                );
            }
            if super::m5r::is_equality_operator_kind(previous_token.kind) {
                if let ast::NodeData::BinaryExpression(d) = &parent.data {
                    if super::m5r::is_equality_operator_kind(d.operator_token.kind) {
                        // completion at `x ===/**/`
                        return Some(type_checker.get_type_at_location(&d.left));
                    }
                }
            }
            match type_checker.get_contextual_type(previous_token, ContextFlags::IgnoreNodeInferences) {
                Some(contextual_type) => Some(contextual_type),
                None => type_checker.get_contextual_type(previous_token, ContextFlags::None),
            }
        }
    }
}

fn get_contextual_type_from_parent(
    node: &Arc<Node>,
    type_checker: &mut Checker,
    context_flags: tsox_checker::checker::types::ContextFlags,
) -> Option<Arc<tsox_checker::checker::types::Type>> {
    super::m5x_6::get_contextual_type_from_parent(node, type_checker, context_flags)
}

pub fn compute_commit_characters_and_is_new_identifier(
    context_token: Option<&Arc<Node>>,
    file: &Arc<SourceFile>,
    position: usize,
) -> (bool, Vec<String>) {
    let Some(context_token) = context_token else {
        return (false, super::m5q_3::all_commit_characters().to_vec());
    };
    let Some(parent) = context_token.parent() else {
        return (false, super::m5q_3::all_commit_characters().to_vec());
    };
    let containing_node_kind = parent.kind;
    let token_kind = keyword_for_node(context_token);
    use SyntaxKind as K;
    match token_kind {
        K::CommaToken => match containing_node_kind {
            // func( a, |
            // new C(a, |
            K::CallExpression | K::NewExpression => {
                // func\n(a, |
                if parent
                    .expression()
                    .map_or(false, |e| get_line_of_position(file, e.end()) != get_line_of_position(file, position))
                {
                    return (true, no_comma_commit_characters());
                }
                (true, super::m5q_3::all_commit_characters().to_vec())
            }
            // const x = (a, |
            K::BinaryExpression => (true, no_comma_commit_characters()),
            // constructor( a, | /* public, protected, private keywords are allowed here, so show completion */
            // var x: (s: string, list|
            // const obj = { x, |
            K::Constructor | K::FunctionType | K::ObjectLiteralExpression => {
                (true, empty_commit_characters())
            }
            // [a, |
            K::ArrayLiteralExpression => (true, super::m5q_3::all_commit_characters().to_vec()),
            _ => (false, super::m5q_3::all_commit_characters().to_vec()),
        },
        K::OpenParenToken => match containing_node_kind {
            // func( |
            // new C(a|
            K::CallExpression | K::NewExpression => {
                // func\n( |
                if parent
                    .expression()
                    .map_or(false, |e| get_line_of_position(file, e.end()) != get_line_of_position(file, position))
                {
                    return (true, no_comma_commit_characters());
                }
                (true, super::m5q_3::all_commit_characters().to_vec())
            }
            // const x = (a|
            K::ParenthesizedExpression => (true, no_comma_commit_characters()),
            // constructor( |
            // function F(pred: (a| /* this can become an arrow function, where 'a' is the argument */
            K::Constructor | K::ParenthesizedType => (true, empty_commit_characters()),
            _ => (false, super::m5q_3::all_commit_characters().to_vec()),
        },
        K::OpenBracketToken => match containing_node_kind {
            // [ |
            // [ | : string ]
            // [ |    /* this can become an index signature */
            K::ArrayLiteralExpression | K::IndexSignature | K::TupleType | K::ComputedPropertyName => {
                (true, super::m5q_3::all_commit_characters().to_vec())
            }
            _ => (false, super::m5q_3::all_commit_characters().to_vec()),
        },
        // module |
        // namespace |
        // import |
        K::ModuleKeyword | K::NamespaceKeyword | K::ImportKeyword => (true, empty_commit_characters()),
        K::DotToken => match containing_node_kind {
            // module A.|
            K::ModuleDeclaration => (true, empty_commit_characters()),
            _ => (false, super::m5q_3::all_commit_characters().to_vec()),
        },
        K::OpenBraceToken => match containing_node_kind {
            // class A { |
            // const obj = { |
            K::ClassDeclaration | K::ObjectLiteralExpression => (true, empty_commit_characters()),
            _ => (false, super::m5q_3::all_commit_characters().to_vec()),
        },
        K::EqualsToken => match containing_node_kind {
            // const x = a|
            // x = a|
            K::VariableDeclaration | K::BinaryExpression => {
                (true, super::m5q_3::all_commit_characters().to_vec())
            }
            _ => (false, super::m5q_3::all_commit_characters().to_vec()),
        },
        K::TemplateHead => {
            // `aa ${|`
            (containing_node_kind == K::TemplateExpression, super::m5q_3::all_commit_characters().to_vec())
        }
        K::TemplateMiddle => {
            // `aa ${10} dd ${|`
            (containing_node_kind == K::TemplateSpan, super::m5q_3::all_commit_characters().to_vec())
        }
        K::AsyncKeyword => {
            // const obj = { async c|()
            // const obj = { async c|
            if containing_node_kind == K::MethodDeclaration || containing_node_kind == K::ShorthandPropertyAssignment {
                return (true, empty_commit_characters());
            }
            (false, super::m5q_3::all_commit_characters().to_vec())
        }
        K::AsteriskToken => {
            // const obj = { * c|
            if containing_node_kind == K::MethodDeclaration {
                return (true, empty_commit_characters());
            }
            (false, super::m5q_3::all_commit_characters().to_vec())
        }
        _ => {
            if super::m5r::is_class_member_completion_keyword(token_kind) {
                return (true, empty_commit_characters());
            }
            (false, super::m5q_3::all_commit_characters().to_vec())
        }
    }
}

pub fn no_comma_commit_characters() -> Vec<String> {
    vec![",".to_string()]
}

pub fn empty_commit_characters() -> Vec<String> {
    Vec::new()
}

pub fn keyword_for_node(node: &Arc<Node>) -> SyntaxKind {
    if ast::is_identifier(node) {
        return scanner::identifier_to_keyword_kind(node);
    }
    node.kind
}

pub fn filter_class_members_list(
    base_symbols: Vec<Arc<Symbol>>,
    existing_members: &[Arc<Node>],
    class_element_modifier_flags: ast::ModifierFlags,
    file: &Arc<SourceFile>,
    position: usize,
) -> Vec<Arc<Symbol>> {
    let mut existing_member_names: HashSet<String> = HashSet::new();
    for member in existing_members {
        // Ignore omitted expressions for missing members.
        if member.kind != SyntaxKind::PropertyDeclaration
            && member.kind != SyntaxKind::MethodDeclaration
            && member.kind != SyntaxKind::GetAccessor
            && member.kind != SyntaxKind::SetAccessor
        {
            continue;
        }

        // If this is the current item we are editing right now, do not filter it out
        if astnav::get_start_of_node(member, file, false) <= position && position <= member.end() {
            continue;
        }

        // Don't filter member even if the name matches if it is declared private in the list.
        if ast::get_combined_modifier_flags(member) & ast::ModifierFlags::Private != ast::ModifierFlags::empty() {
            continue;
        }

        // Do not filter it out if the static presence doesn't match.
        if ast::is_static(member) != (class_element_modifier_flags & ast::ModifierFlags::Static != ast::ModifierFlags::empty()) {
            continue;
        }

        let existing_name = member.name().map(|n| m3f::get_property_name_for_property_name_node(n)).unwrap_or_default();
        if !existing_name.is_empty() {
            existing_member_names.insert(existing_name);
        }
    }

    base_symbols
        .into_iter()
        .filter(|property_symbol| {
            !existing_member_names.contains(&m3e_3::symbol_name(property_symbol))
                && !property_symbol.declarations.is_empty()
                && tsox_checker::checker::get_declaration_modifier_flags_from_symbol(property_symbol)
                    & ast::ModifierFlags::Private
                    == ast::ModifierFlags::empty()
                && !property_symbol
                    .value_declaration
                    .as_ref()
                    .map_or(false, |d| m3g_2::is_private_identifier_class_element_declaration(d))
        })
        .collect()
}

pub fn get_jsdoc_tag_at_position(node: &Arc<Node>, position: usize) -> Option<Arc<Node>> {
    let mut current = Some(Arc::clone(node));
    while let Some(n) = current {
        if ast::is_jsdoc_tag(&n) && n.loc.contains_inclusive(position) {
            return Some(n);
        }
        if ast::is_jsdoc(&n) {
            return None;
        }
        current = n.parent();
    }
    None
}

pub fn get_jsdoc_param_name_with_initializer(param_name: &str, initializer: &Arc<Node>) -> String {
    let initializer_text = scanner::get_text_of_node(initializer).trim().to_string();
    if initializer_text.contains('\n') || initializer_text.len() > 80 {
        return format!("[{}]", param_name);
    }
    format!("[{}={}]", param_name, initializer_text)
}

pub fn generate_jsdoc_param_tags_for_destructuring(
    path: &str,
    pattern: &Arc<Node>,
    initializer: Option<&Arc<Node>>,
    dot_dot_dot_token: Option<&Arc<Node>>,
    is_js: bool,
    is_snippet: bool,
    type_checker: &mut Checker,
    options: &core::CompilerOptions,
    preferences: &UserPreferences,
) -> Vec<String> {
    let mut tabstop_counter = 1;
    if !is_js {
        return vec![get_jsdoc_param_annotation(
            path,
            initializer,
            dot_dot_dot_token,
            is_js,
            false,
            is_snippet,
            type_checker,
            options,
            preferences,
            &mut tabstop_counter,
        )];
    }
    jsdoc_param_pattern_worker(
        path,
        pattern,
        initializer,
        dot_dot_dot_token,
        is_js,
        is_snippet,
        type_checker,
        options,
        preferences,
        &mut tabstop_counter,
    )
}

pub fn jsdoc_param_pattern_worker(
    path: &str,
    pattern: &Arc<Node>,
    initializer: Option<&Arc<Node>>,
    dot_dot_dot_token: Option<&Arc<Node>>,
    is_js: bool,
    is_snippet: bool,
    type_checker: &mut Checker,
    options: &core::CompilerOptions,
    preferences: &UserPreferences,
    counter: &mut i32,
) -> Vec<String> {
    if ast::is_object_binding_pattern(pattern) && dot_dot_dot_token.is_none() {
        let mut child_counter = *counter;
        let root_param = get_jsdoc_param_annotation(
            path,
            initializer,
            dot_dot_dot_token,
            is_js,
            true,
            is_snippet,
            type_checker,
            options,
            preferences,
            &mut child_counter,
        );
        let mut child_tags: Vec<String> = Vec::new();
        let mut ok = true;
        for element in m3b::elements(pattern) {
            let element_tags = jsdoc_param_element_worker(
                path,
                element,
                initializer,
                dot_dot_dot_token,
                is_js,
                is_snippet,
                type_checker,
                options,
                preferences,
                &mut child_counter,
            );
            if element_tags.is_empty() {
                ok = false;
                break;
            }
            child_tags.extend(element_tags);
        }
        if ok && !child_tags.is_empty() {
            *counter = child_counter;
            let mut result = vec![root_param];
            result.extend(child_tags);
            return result;
        }
    }
    vec![get_jsdoc_param_annotation(
        path,
        initializer,
        dot_dot_dot_token,
        is_js,
        false,
        is_snippet,
        type_checker,
        options,
        preferences,
        counter,
    )]
}

pub fn jsdoc_param_element_worker(
    path: &str,
    element: &Arc<Node>,
    initializer: Option<&Arc<Node>>,
    dot_dot_dot_token: Option<&Arc<Node>>,
    is_js: bool,
    is_snippet: bool,
    type_checker: &mut Checker,
    options: &core::CompilerOptions,
    preferences: &UserPreferences,
    counter: &mut i32,
) -> Vec<String> {
    if element.name().is_some_and(|n| ast::is_identifier(n)) {
        // `{ b }` or `{ b: newB }`
        let property_name = if let Some(property_name) = m3b::property_name(element) {
            m3h::try_get_text_of_property_name(property_name).unwrap_or_default()
        } else {
            element.name().map_or(String::new(), |n| n.text().to_string())
        };
        if property_name.is_empty() {
            return Vec::new();
        }
        let param_name = format!("{}.{}", path, property_name);
        return vec![get_jsdoc_param_annotation(
            &param_name,
            m3b::initializer(element),
            match &element.data {
                ast::NodeData::BindingElement(d) => d.dot_dot_dot_token.as_ref(),
                _ => None,
            },
            is_js,
            false,
            is_snippet,
            type_checker,
            options,
            preferences,
            counter,
        )];
    } else if let Some(property_name) = m3b::property_name(element) {
        // `{ b: {...} }` or `{ b: [...] }`
        let property_name = m3h::try_get_text_of_property_name(property_name).unwrap_or_default();
        if property_name.is_empty() {
            return Vec::new();
        }
        let Some(name) = element.name() else {
            return Vec::new();
        };
        return jsdoc_param_pattern_worker(
            &format!("{}.{}", path, property_name),
            name,
            m3b::initializer(element),
            match &element.data {
                ast::NodeData::BindingElement(d) => d.dot_dot_dot_token.as_ref(),
                _ => None,
            },
            is_js,
            is_snippet,
            type_checker,
            options,
            preferences,
            counter,
        );
    }
    Vec::new()
}
