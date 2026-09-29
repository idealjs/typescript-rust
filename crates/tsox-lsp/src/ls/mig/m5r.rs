#![allow(dead_code, unused_imports, unused_variables)]

use std::collections::HashSet;
use std::sync::Arc;

use super::m5q2b_3::lsproto;
use crate::ls::lsutil_user_preferences::{QuotePreference, UserPreferences};
use tsox_checker::checker::Checker;
use tsox_checker::checker::mig::r19k5_flags_ext::TypeFlagsExt as _;
use tsox_checker::checker::types::TypeFlags;
use tsox_core::core;
use tsox_frontend::ast::{self, Node, SourceFile, Symbol};
use tsox_frontend::ast::mig::m3b_2::NodeFactory;
use tsox_frontend::astnav;
use tsox_frontend::scanner;

use super::m5q_3::{
    all_commit_characters, client_supports_default_commit_characters, client_supports_default_edit_range,
    client_supports_item_commit_characters, client_supports_item_insert_replace,
    get_default_commit_characters,
};

pub use super::m5q_3::{could_be_type_only_import_specifier, get_dot_accessor};
pub use crate::ls::lsutil_utilities::is_non_contextual_keyword;

pub type SymbolOriginInfoKind = u32;

pub const SYMBOL_ORIGIN_INFO_KIND_THIS_TYPE: SymbolOriginInfoKind = 1;
pub const SYMBOL_ORIGIN_INFO_KIND_SYMBOL_MEMBER: SymbolOriginInfoKind = 2;
pub const SYMBOL_ORIGIN_INFO_KIND_PROMISE: SymbolOriginInfoKind = 4;
pub const SYMBOL_ORIGIN_INFO_KIND_NULLABLE: SymbolOriginInfoKind = 8;
pub const SYMBOL_ORIGIN_INFO_KIND_TYPE_ONLY_ALIAS: SymbolOriginInfoKind = 16;
pub const SYMBOL_ORIGIN_INFO_KIND_OBJECT_LITERAL_METHOD: SymbolOriginInfoKind = 32;
pub const SYMBOL_ORIGIN_INFO_KIND_IGNORE: SymbolOriginInfoKind = 64;
pub const SYMBOL_ORIGIN_INFO_KIND_COMPUTED_PROPERTY_NAME: SymbolOriginInfoKind = 128;

pub const COMPLETION_SOURCE_THIS_PROPERTY: &str = "ThisProperty/";
pub const COMPLETION_SOURCE_CLASS_MEMBER_SNIPPET: &str = "ClassMemberSnippet/";
pub const COMPLETION_SOURCE_TYPE_ONLY_ALIAS: &str = "TypeOnlyAlias/";
pub const COMPLETION_SOURCE_OBJECT_LITERAL_METHOD_SNIPPET: &str = "ObjectLiteralMethodSnippet/";
pub const COMPLETION_SOURCE_SWITCH_CASES: &str = "SwitchCases/";
pub const COMPLETION_SOURCE_OBJECT_LITERAL_MEMBER_WITH_COMMA: &str = "ObjectLiteralMemberWithComma/";

#[derive(Default, Clone)]
pub struct SymbolOriginInfo {
    pub kind: SymbolOriginInfoKind,
    pub is_default_export: bool,
    pub is_from_package_json: bool,
    pub file_name: String,
    pub data: Option<SymbolOriginInfoData>,
}

#[derive(Clone)]
pub enum SymbolOriginInfoData {
    ObjectLiteralMethod(SymbolOriginInfoObjectLiteralMethod),
    TypeOnlyAlias(Box<Arc<Node>>),
    ComputedPropertyName { symbol_name: String },
}

#[derive(Clone, Default)]
pub struct SymbolOriginInfoObjectLiteralMethod {
    pub insert_text: String,
    pub label_details: Option<lsproto::CompletionItemLabelDetails>,
    pub is_snippet: bool,
}

impl SymbolOriginInfo {
    pub fn as_object_literal_method(&self) -> &SymbolOriginInfoObjectLiteralMethod {
        match &self.data {
            Some(SymbolOriginInfoData::ObjectLiteralMethod(d)) => d,
            _ => panic!("symbolOriginInfo: data is not objectLiteralMethod"),
        }
    }
}

pub fn origin_is_ignore(origin: Option<&SymbolOriginInfo>) -> bool {
    origin.map_or(false, |o| o.kind & SYMBOL_ORIGIN_INFO_KIND_IGNORE != 0)
}

pub fn origin_includes_symbol_name(origin: Option<&SymbolOriginInfo>) -> bool {
    origin_is_computed_property_name(origin)
}

pub fn origin_is_computed_property_name(origin: Option<&SymbolOriginInfo>) -> bool {
    origin.map_or(false, |o| o.kind & SYMBOL_ORIGIN_INFO_KIND_COMPUTED_PROPERTY_NAME != 0)
}

pub fn origin_is_object_literal_method(origin: Option<&SymbolOriginInfo>) -> bool {
    origin.map_or(false, |o| o.kind & SYMBOL_ORIGIN_INFO_KIND_OBJECT_LITERAL_METHOD != 0)
}

pub fn origin_is_this_type_node(origin: Option<&SymbolOriginInfo>) -> bool {
    origin.map_or(false, |o| o.kind & SYMBOL_ORIGIN_INFO_KIND_THIS_TYPE != 0)
}

pub fn origin_is_type_only_alias(origin: Option<&SymbolOriginInfo>) -> bool {
    origin.map_or(false, |o| o.kind & SYMBOL_ORIGIN_INFO_KIND_TYPE_ONLY_ALIAS != 0)
}

pub fn origin_is_symbol_member(origin: Option<&SymbolOriginInfo>) -> bool {
    origin.map_or(false, |o| o.kind & SYMBOL_ORIGIN_INFO_KIND_SYMBOL_MEMBER != 0)
}

pub fn origin_is_nullable_member(origin: Option<&SymbolOriginInfo>) -> bool {
    origin.map_or(false, |o| o.kind & SYMBOL_ORIGIN_INFO_KIND_NULLABLE != 0)
}

pub fn origin_is_promise(origin: Option<&SymbolOriginInfo>) -> bool {
    origin.map_or(false, |o| o.kind & SYMBOL_ORIGIN_INFO_KIND_PROMISE != 0)
}

pub fn symbol_name(origin: &SymbolOriginInfo) -> String {
    match &origin.data {
        Some(SymbolOriginInfoData::ComputedPropertyName { symbol_name }) => symbol_name.clone(),
        _ => panic!("symbolOriginInfo: unknown data type for symbol_name()"),
    }
}

pub fn get_source_from_origin(origin: Option<&SymbolOriginInfo>) -> String {
    if origin_is_this_type_node(origin) {
        return COMPLETION_SOURCE_THIS_PROPERTY.to_string();
    }
    if origin_is_type_only_alias(origin) {
        return COMPLETION_SOURCE_TYPE_ONLY_ALIAS.to_string();
    }
    String::new()
}

pub fn get_nullable_symbol_origin_info_kind(
    kind: SymbolOriginInfoKind,
    insert_question_dot: bool,
) -> SymbolOriginInfoKind {
    if insert_question_dot {
        return kind | SYMBOL_ORIGIN_INFO_KIND_NULLABLE;
    }
    kind
}

pub fn str_ptr_is_empty(v: Option<&String>) -> bool {
    match v {
        None => true,
        Some(s) => s.is_empty(),
    }
}

pub fn str_ptr_to(v: &str) -> Option<String> {
    if v.is_empty() {
        None
    } else {
        Some(v.to_string())
    }
}

pub fn starts_with_quote(s: &str) -> bool {
    matches!(s.chars().next(), Some('"') | Some('\''))
}

pub fn trim_element_access(text: &str) -> String {
    let mut text = text.strip_prefix('[').unwrap_or(text);
    text = text.strip_suffix(']').unwrap_or(text);
    if text.starts_with('\'') && text.ends_with('\'') && text.len() >= 2 {
        text = &text[1..text.len() - 1];
    }
    if text.starts_with('"') && text.ends_with('"') && text.len() >= 2 {
        text = &text[1..text.len() - 1];
    }
    text.to_string()
}

pub fn is_string_literal_or_template(node: &Arc<Node>) -> bool {
    matches!(
        node.kind,
        ast::SyntaxKind::StringLiteral
            | ast::SyntaxKind::NoSubstitutionTemplateLiteral
            | ast::SyntaxKind::TemplateExpression
            | ast::SyntaxKind::TaggedTemplateExpression
    )
}

pub fn is_named_imports_or_exports(node: &Arc<Node>) -> bool {
    ast::is_named_imports(node) || ast::is_named_exports(node)
}

pub fn is_type_script_only_keyword(kind: ast::SyntaxKind) -> bool {
    use ast::SyntaxKind as K;
    matches!(
        kind,
        K::AbstractKeyword
            | K::AnyKeyword
            | K::BigIntKeyword
            | K::BooleanKeyword
            | K::DeclareKeyword
            | K::EnumKeyword
            | K::GlobalKeyword
            | K::ImplementsKeyword
            | K::InferKeyword
            | K::InterfaceKeyword
            | K::IsKeyword
            | K::KeyOfKeyword
            | K::ModuleKeyword
            | K::NamespaceKeyword
            | K::NeverKeyword
            | K::NumberKeyword
            | K::ObjectKeyword
            | K::OverrideKeyword
            | K::PrivateKeyword
            | K::ProtectedKeyword
            | K::PublicKeyword
            | K::ReadonlyKeyword
            | K::StringKeyword
            | K::SymbolKeyword
            | K::TypeKeyword
            | K::UniqueKeyword
            | K::UnknownKeyword
    )
}

pub fn is_function_like_body_keyword(kind: ast::SyntaxKind) -> bool {
    use ast::SyntaxKind as K;
    kind == K::AsyncKeyword
        || kind == K::AwaitKeyword
        || kind == K::UsingKeyword
        || kind == K::AsKeyword
        || kind == K::SatisfiesKeyword
        || kind == K::TypeKeyword
        || (!ast::mig::m3f_4::is_contextual_keyword(kind) && !is_class_member_completion_keyword(kind))
}

pub fn is_class_member_completion_keyword(kind: ast::SyntaxKind) -> bool {
    use ast::SyntaxKind as K;
    matches!(
        kind,
        K::AbstractKeyword
            | K::AccessorKeyword
            | K::ConstructorKeyword
            | K::GetKeyword
            | K::SetKeyword
            | K::AsyncKeyword
            | K::DeclareKeyword
            | K::OverrideKeyword
    ) || ast::mig::m3f_4::is_class_member_modifier(kind)
}

pub fn is_interface_or_type_literal_completion_keyword(kind: ast::SyntaxKind) -> bool {
    kind == ast::SyntaxKind::ReadonlyKeyword
}

pub fn is_contextual_keyword_in_auto_importable_expression_space(keyword: &str) -> bool {
    matches!(
        keyword,
        "abstract" | "async" | "await" | "declare" | "module" | "namespace" | "type" | "satisfies" | "as"
    )
}

pub fn is_equality_operator_kind(kind: ast::SyntaxKind) -> bool {
    use ast::SyntaxKind as K;
    matches!(
        kind,
        K::EqualsEqualsEqualsToken | K::EqualsEqualsToken | K::ExclamationEqualsEqualsToken | K::ExclamationEqualsToken
    )
}

pub fn is_member_completion_kind(kind: crate::ls::completions::CompletionKind) -> bool {
    use crate::ls::completions::CompletionKind as K;
    matches!(kind, K::ObjectLiteralMember | K::Member | K::PropertyAccess)
}

pub fn get_line_end_of_position(file: &Arc<SourceFile>, pos: usize) -> usize {
    let line = tsox_emit::mig::m4m_2::get_ecma_line_of_position(file, pos);
    let line_starts = tsox_frontend::format::mig::m4t_3::get_ecma_line_starts(file);
    let last_char_pos = if line + 1 >= line_starts.len() {
        file.text.len()
    } else {
        line_starts[line + 1] as usize - 1
    };
    let full_text = file.text.as_bytes();
    if last_char_pos > 0
        && last_char_pos < full_text.len()
        && full_text[last_char_pos] == b'\n'
        && full_text[last_char_pos - 1] == b'\r'
    {
        return last_char_pos - 1;
    }
    last_char_pos
}

pub fn is_abstract_constructor_symbol(symbol: &Arc<Symbol>) -> bool {
    if symbol.flags & ast::SymbolFlags::Class != ast::SymbolFlags::None {
        let declaration = ast::mig::x4ast::get_class_like_declaration_of_symbol(symbol);
        return declaration.is_some()
            && ast::has_syntactic_modifier(&declaration.unwrap(), ast::ModifierFlags::Abstract);
    }
    false
}

pub fn is_deprecated(symbol: &Arc<Symbol>, type_checker: &mut Checker) -> bool {
    let aliased = type_checker.skip_alias(symbol);
    let declarations = &aliased.declarations;
    !declarations.is_empty()
        && declarations.iter().all(|decl| type_checker.is_deprecated_declaration(decl))
}

pub fn is_recommended_completion_match(
    local_symbol: &Arc<Symbol>,
    recommended_completion: &Arc<Symbol>,
    type_checker: &mut Checker,
) -> bool {
    Arc::ptr_eq(local_symbol, recommended_completion)
        || local_symbol.flags & ast::SymbolFlags::ExportValue != ast::SymbolFlags::None
            && Arc::ptr_eq(&type_checker.get_export_symbol_of_symbol(local_symbol), recommended_completion)
}

pub fn is_static_property(symbol: &Arc<Symbol>) -> bool {
    symbol
        .value_declaration
        .as_ref()
        .map_or(false, |decl| {
            decl.syntactic_modifier_flags().intersects(ast::ModifierFlags::Static)
                && decl.parent().map_or(false, |p| ast::is_class_like(&p))
        })
}

pub fn is_literal(t: &tsox_checker::checker::Type) -> bool {
    t.is_string_literal() || t.is_number_literal() || t.is_big_int_literal()
}

pub fn is_checked_file(file: &Arc<SourceFile>, compiler_options: &core::compiler_options::CompilerOptions) -> bool {
    !ast::is_source_file_js(file) || ast::mig::m3f_4::is_check_js_enabled_for_file(file, compiler_options)
}

pub fn is_context_token_value_location(context_token: &Arc<Node>) -> bool {
    let parent_kind = context_token.parent().map(|p| p.kind);
    context_token.kind == ast::SyntaxKind::TypeOfKeyword
        && parent_kind == Some(ast::SyntaxKind::TypeQuery)
        || context_token.kind == ast::SyntaxKind::AssertsKeyword
            && parent_kind == Some(ast::SyntaxKind::TypePredicate)
}

pub fn is_context_token_type_location(context_token: Option<&Arc<Node>>) -> bool {
    let Some(context_token) = context_token else {
        return false;
    };
    let Some(parent) = context_token.parent() else {
        return false;
    };
    let parent_kind = parent.kind;
    use ast::SyntaxKind as K;
    match context_token.kind {
        K::ColonToken => matches!(
            parent_kind,
            K::PropertyDeclaration | K::PropertySignature | K::Parameter | K::VariableDeclaration
        ) || ast::is_function_like_kind(parent_kind),
        K::EqualsToken => parent_kind == K::TypeAliasDeclaration || parent_kind == K::TypeParameter,
        K::AsKeyword => parent_kind == K::AsExpression,
        K::LessThanToken => parent_kind == K::TypeReference || parent_kind == K::TypeAssertionExpression,
        K::ExtendsKeyword => parent_kind == K::TypeParameter,
        K::SatisfiesKeyword => parent_kind == K::SatisfiesExpression,
        _ => false,
    }
}

pub fn non_alias_can_be_referenced_at_type_location(
    symbol: &Arc<Symbol>,
    type_checker: &mut Checker,
    seen_modules: &mut HashSet<u64>,
) -> bool {
    symbol.flags & ast::SymbolFlags::TYPE != ast::SymbolFlags::None
        || type_checker.is_unknown_symbol(symbol)
            && symbol.flags & ast::SymbolFlags::MODULE != ast::SymbolFlags::None
            && seen_modules.insert(ast::get_symbol_id(symbol))
            && type_checker
                .get_exports_of_module(symbol)
                .iter()
                .any(|e| symbol_can_be_referenced_at_type_location(e, type_checker, seen_modules))
}

pub fn symbol_can_be_referenced_at_type_location(
    symbol: &Arc<Symbol>,
    type_checker: &mut Checker,
    seen_modules: &mut HashSet<u64>,
) -> bool {
    non_alias_can_be_referenced_at_type_location(symbol, type_checker, seen_modules)
        || non_alias_can_be_referenced_at_type_location(
            &type_checker.skip_alias(
                symbol
                    .export_symbol
                    .as_ref()
                    .unwrap_or(symbol),
            ),
            type_checker,
            seen_modules,
        )
}

pub fn get_properties_for_completion(t: &Arc<tsox_checker::checker::Type>, type_checker: &mut Checker) -> Vec<Arc<Symbol>> {
    if t.is_union() {
        type_checker.get_all_possible_properties_of_types(t.types().unwrap_or(&[]))
    } else {
        type_checker.get_apparent_properties(t)
    }
}

pub fn get_left_most_name(e: &Arc<Node>) -> Option<Arc<Node>> {
    if ast::is_identifier(e) {
        Some(e.clone())
    } else if ast::is_property_access_expression(e) {
        e.expression()
            .and_then(get_left_most_name)
    } else {
        None
    }
}

pub fn is_in_type_parameter_default(context_token: &Arc<Node>) -> bool {
    let mut node = context_token.clone();
    let mut parent = context_token.parent();
    while let Some(p) = parent {
        if ast::is_type_parameter_declaration(&p) {
            let default_is_node = match &p.data {
                ast::node_data_generated::NodeData::TypeParameterDeclaration(d) => d
                    .default_type
                    .as_ref()
                    .map_or(false, |dt| Arc::ptr_eq(dt, &node)),
                _ => false,
            };
            return default_is_node || node.kind == ast::SyntaxKind::EqualsToken;
        }
        node = p.clone();
        parent = p.parent();
    }
    false
}

pub fn keyword_for_node(node: &Arc<Node>) -> ast::SyntaxKind {
    if ast::is_identifier(node) {
        return scanner::mig::m3i::identifier_to_keyword_kind(node);
    }
    node.kind
}

pub fn is_snippet_scope(scope_node: &Arc<Node>) -> bool {
    use ast::SyntaxKind as K;
    match scope_node.kind {
        K::SourceFile | K::TemplateExpression | K::JsxExpression | K::Block => true,
        _ => ast::is_statement(scope_node),
    }
}

pub fn quote_property_name(
    file: &Arc<SourceFile>,
    preferences: &UserPreferences,
    name: &str,
) -> String {
    if name.chars().next().map_or(false, |c| c.is_ascii_digit()) {
        return name.to_string();
    }
    super::m5q_3::quote(file, preferences, name)
}

pub fn modifier_like_kind(node: Option<&Arc<Node>>) -> ast::SyntaxKind {
    let Some(node) = node else {
        return ast::SyntaxKind::Unknown;
    };
    if ast::mig::m3g::is_modifier(node) {
        return node.kind;
    }
    if ast::is_identifier(node) {
        let keyword_kind = scanner::mig::m3i::identifier_to_keyword_kind(node);
        if keyword_kind != ast::SyntaxKind::Unknown && ast::is_modifier_kind(keyword_kind) {
            return keyword_kind;
        }
    }
    ast::SyntaxKind::Unknown
}

pub fn get_word_length_and_start(file: &Arc<SourceFile>, position: usize) -> (usize, char) {
    const WORD_SEPARATORS: &str = "`~!@%^&*()-=+[{]}\\|;:'\",.<>/?";
    let text = &file.text[..position.min(file.text.len())];
    let mut total_size = 0usize;
    let mut first_char: Option<char> = None;
    for ch in text.chars().rev() {
        if WORD_SEPARATORS.contains(ch) || ch.is_whitespace() {
            break;
        }
        total_size += ch.len_utf8();
        first_char = Some(ch);
    }
    let mut word_start = first_char.unwrap_or('\0');
    if word_start == '@' {
        total_size = total_size.saturating_sub(1);
        word_start = text[text.len() - total_size..].chars().next().unwrap_or('\0');
    }
    (total_size, word_start)
}

pub fn is_object_literal_method_completion_candidate_declaration(declaration: Option<&Arc<Node>>) -> bool {
    use ast::SyntaxKind as K;
    match declaration {
        None => false,
        Some(declaration) => matches!(
            declaration.kind,
            K::PropertySignature | K::PropertyDeclaration | K::MethodSignature | K::MethodDeclaration
        ),
    }
}

pub fn is_object_literal_method_symbol(symbol: &Arc<Symbol>) -> bool {
    symbol.flags & (ast::SymbolFlags::Property | ast::SymbolFlags::Method) != ast::SymbolFlags::None
}

pub fn is_type_keyword_token_or_identifier(node: &Arc<Node>) -> bool {
    ast::mig::m3g_2::is_type_keyword_token(node)
        || ast::is_identifier(node) && scanner::mig::m3i::identifier_to_keyword_kind(node) == ast::SyntaxKind::TypeKeyword
}

pub fn is_variable_declaration_list_but_not_type_argument(
    node: &Arc<Node>,
    file: &Arc<SourceFile>,
    type_checker: &mut Checker,
) -> bool {
    node.parent()
        .map_or(false, |p| {
            p.kind == ast::SyntaxKind::VariableDeclarationList
                && !is_possibly_type_argument_position(node, file, type_checker)
        })
}

pub fn is_possibly_type_argument_position(
    token: &Arc<Node>,
    source_file: &Arc<SourceFile>,
    type_checker: &mut Checker,
) -> bool {
    crate::ls::mig::m5x_5::get_possible_type_arguments_info(token, source_file)
        .map_or(false, |info| {
            ast::mig::m3g_3::is_part_of_type_node(&info.called)
                || !crate::ls::mig::m5x_5::get_possible_generic_signatures(
                    &info.called,
                    info.n_type_arguments,
                    type_checker,
                )
                .is_empty()
                || is_possibly_type_argument_position(&info.called, source_file, type_checker)
        })
}

pub fn is_module_specifier_missing_or_empty(specifier: Option<&Arc<Node>>) -> bool {
    let Some(specifier) = specifier else {
        return true;
    };
    if ast::node_is_missing(Some(specifier)) {
        return true;
    }
    let mut node = specifier.clone();
    if ast::is_external_module_reference(&node) {
        match node.expression() {
            Some(expr) => node = expr.clone(),
            None => return true,
        }
    }
    if !ast::is_string_literal_like(&node) {
        return true;
    }
    node.text().is_empty()
}

pub fn is_tag_with_type_expression(tag: &Arc<Node>) -> bool {
    use ast::SyntaxKind as K;
    match tag.kind {
        K::JSDocParameterTag
        | K::JSDocPropertyTag
        | K::JSDocReturnTag
        | K::JSDocTypeTag
        | K::JSDocTypedefTag
        | K::JSDocThrowsTag
        | K::JSDocSatisfiesTag => true,
        K::JSDocTemplateTag => match &tag.data {
            ast::node_data_generated::NodeData::JSDocTemplateTag(d) => {
                !ast::node_is_missing(Some(&d.constraint))
            }
            _ => false,
        },
        _ => false,
    }
}

fn jsdoc_tag_type_expression(tag: &Arc<Node>) -> Option<Arc<Node>> {
    use ast::node_data_generated::NodeData;
    match &tag.data {
        NodeData::JSDocParameterOrPropertyTag(d) => d.type_expression.clone(),
        NodeData::JSDocReturnTag(d) => d.type_expression.clone(),
        NodeData::JSDocTypeTag(d) => Some(d.type_expression.clone()),
        NodeData::JSDocTypedefTag(d) => d.type_expression.clone(),
        NodeData::JSDocThrowsTag(d) => d.type_expression.clone(),
        NodeData::JSDocSatisfiesTag(d) => Some(d.type_expression.clone()),
        _ => None,
    }
}

pub fn try_get_type_expression_from_tag(tag: &Arc<Node>) -> Option<Arc<Node>> {
    let mut type_expression: Option<Arc<Node>> = None;
    if is_tag_with_type_expression(tag) {
        if ast::is_jsdoc_template_tag(tag) {
            type_expression = match &tag.data {
                ast::node_data_generated::NodeData::JSDocTemplateTag(d) => {
                    Some(d.constraint.clone())
                }
                _ => None,
            };
        } else {
            type_expression = jsdoc_tag_type_expression(tag);
        }
        if let Some(te) = &type_expression {
            if te.kind == ast::SyntaxKind::JSDocTypeExpression {
                return type_expression;
            }
        }
    }
    if ast::is_jsdoc_augments_tag(tag) || ast::is_jsdoc_implements_tag(tag) {
        return match &tag.data {
            ast::node_data_generated::NodeData::JSDocAugmentsTag(d) => Some(d.class_name.clone()),
            ast::node_data_generated::NodeData::JSDocImplementsTag(d) => Some(d.class_name.clone()),
            _ => None,
        };
    }
    None
}

pub fn get_potentially_invalid_import_specifier(named_bindings: Option<&Arc<Node>>) -> Option<Arc<Node>> {
    let named_bindings = named_bindings?;
    if named_bindings.kind != ast::SyntaxKind::NamedImports {
        return None;
    }
    let source_file = ast::get_source_file_of_node(named_bindings)?;
    ast::mig::m3b::elements(named_bindings)
        .iter()
        .find(|e| {
            let name = e.name();
            name.is_some_and(|n| {
                e.property_name().is_none()
                    && crate::ls::lsutil_utilities::is_non_contextual_keyword(scanner::string_to_token(&n.text()))
                    && astnav::find_preceding_token(&source_file, n.pos())
                        .map_or(false, |t| t.kind != ast::SyntaxKind::CommaToken)
            })
        })
        .map(|e| Arc::clone(e))
}

pub fn supplemental_file_index(file: &Arc<SourceFile>) -> Option<i32> {
    let canonical = canonical_source_file(file)?;
    for (i, supplemental) in canonical.supplemental_source_files().iter().enumerate() {
        if Arc::ptr_eq(supplemental, file) {
            return Some(i as i32);
        }
    }
    panic!("supplemental source file is not linked from its canonical source file")
}

fn canonical_source_file(_file: &Arc<SourceFile>) -> Option<Arc<SourceFile>> {
    None
}

pub fn source_file_for_supplemental_file_index(
    file: &Arc<SourceFile>,
    index: Option<i32>,
) -> Option<Arc<SourceFile>> {
    let index = index?;
    let supplemental = file.supplemental_source_files();
    if index >= 0 && (index as usize) < supplemental.len() {
        Some(supplemental[index as usize].clone())
    } else {
        None
    }
}

pub fn keyword_for_node_pub(node: &Arc<Node>) -> ast::SyntaxKind {
    keyword_for_node(node)
}

pub fn set_member_declared_by_spread_assignment(
    declaration: &Arc<Node>,
    members: &mut HashSet<String>,
    type_checker: &mut Checker,
) {
    let Some(expression) = declaration.expression() else {
        return;
    };
    let symbol = type_checker.get_symbol_at_location(&expression);
    let mut t: Option<Arc<tsox_checker::checker::Type>> = None;
    if let Some(symbol) = &symbol {
        t = Some(type_checker.get_type_of_symbol_at_location(symbol, &expression));
    }
    let mut properties: Vec<Arc<Symbol>> = Vec::new();
    if let Some(t) = &t {
        if t.flags.intersects(TypeFlags::StructuredType) {
            properties = t
                .as_structured()
                .map(|s| s.properties.clone())
                .unwrap_or_default();
        }
    }
    for property in properties {
        members.insert(property.name.clone());
    }
}

impl crate::ls::language_service::LanguageService {
    pub fn get_optional_replacement_span(
        &self,
        location: Option<&Arc<Node>>,
        file: &Arc<SourceFile>,
    ) -> Option<lsproto::Range> {
        let location = location?;
        if location.kind == ast::SyntaxKind::Identifier || location.kind == ast::SyntaxKind::PrivateIdentifier {
            let start = astnav::get_start_of_node(location, file, false);
            let (lsp_range, fidelity) = self.m5x_create_lsp_range_from_bounds(start, location.end(), file);
            if fidelity == crate::ls::mig::m5v_7::SPANMAP_FIDELITY_EXACT {
                return Some(lsp_range);
            }
        }
        None
    }

    pub fn get_replacement_range_for_context_token(
        &self,
        file: &Arc<SourceFile>,
        context_token: Option<&Arc<Node>>,
        position: usize,
    ) -> Option<lsproto::Range> {
        let context_token = context_token?;
        use ast::SyntaxKind as K;
        match context_token.kind {
            K::StringLiteral | K::NoSubstitutionTemplateLiteral => {
                super::m5q_3::create_range_from_string_literal_like_content(self, file, &context_token, position as i32)
            }
            _ => {
                let (lsp_range, fidelity) = self.create_lsp_range_from_node(&context_token, file);
                if fidelity != crate::ls::mig::m5v_7::SPANMAP_FIDELITY_EXACT {
                    return None;
                }
                Some(lsp_range)
            }
        }
    }
}
