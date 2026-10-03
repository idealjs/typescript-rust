#![allow(dead_code, unused_imports, unused_variables)]

use std::collections::HashMap;
use std::sync::Arc;

use crate::ls::lsutil_user_preferences::{QuotePreference, UserPreferences};
use tsox_checker::checker::{Checker, Type};
use tsox_core::core;
use tsox_frontend::ast::node_node_list::ModifierList;
use tsox_frontend::ast::{self, Node, NodeData, SourceFile, Symbol, SyntaxKind};
use tsox_frontend::ast::mig::m3b_2::NodeFactory;

fn m5q3_m5u_converters() -> crate::mig::m5u_conv::M5uConverters { ::tsox_core::fntrace::enter("m5q3_m5u_converters"); 
    crate::mig::m5u_conv::new_converters(
        crate::ls::lsconv_converters::PositionEncodingKind::Utf16,
        Box::new(crate::ls::lsconv_linemap::compute_lsp_line_starts),
    )
}

fn new_block_m5q3(factory: &NodeFactory, statements: Vec<Arc<Node>>, multi_line: bool) -> Arc<Node> { ::tsox_core::fntrace::enter("new_block_m5q3"); 
    let _ = factory;
    Arc::new(Node::new(
        SyntaxKind::Block,
        NodeData::Block(tsox_frontend::ast::node_data_generated::BlockData {
            statements: Arc::new(factory.new_node_list(statements)),
            multi_line,
        }),
    ))
}

/// Go completions.go jsDocTagNameCompletionItems/jsDocTagCompletionItems 的
/// 条目形状（Label/Kind=Keyword/SortText=SortTextLocationPriority）
fn jsdoc_tag_items_m5q3(prefixed: bool) -> Vec<lsproto::CompletionItem> { ::tsox_core::fntrace::enter("jsdoc_tag_items_m5q3"); 
    crate::ls::completions_jsdoc::JSDOC_TAG_NAMES
        .iter()
        .map(|tag_name| {
            let mut item = lsproto::CompletionItem::default();
            item.label = if prefixed {
                format!("@{}", tag_name)
            } else {
                (*tag_name).to_string()
            };
            item.kind = Some(lsproto::CompletionItemKind::Keyword);
            item.sort_text = Some(SORT_TEXT_LOCATION_PRIORITY.to_string());
            item
        })
        .collect()
}

pub(crate) mod lsproto {
    pub use crate::lsp::lsproto::*;

    pub use crate::mig::m5m::get_client_capabilities;
    pub use crate::mig::m5m::ResolvedClientCapabilitiesContext as Context;

    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub enum CompletionItemKind {
        Empty = 0,
        Text = 1,
        Method = 2,
        Function = 3,
        Constructor = 4,
        Field = 5,
        Variable = 6,
        Class = 7,
        Interface = 8,
        Module = 9,
        Property = 10,
        Enum = 13,
        Keyword = 14,
        File = 17,
        Folder = 19,
        EnumMember = 20,
        Constant = 21,
    }

    impl CompletionItemKind {
        pub fn empty_value() -> Self { ::tsox_core::fntrace::enter("empty_value"); 
            CompletionItemKind::Empty
        }
    }

    #[derive(Debug, Clone, Default)]
    pub struct CompletionItem {
        pub label: String,
        pub kind: Option<CompletionItemKind>,
        pub detail: Option<String>,
        pub documentation: Option<Documentation>,
        pub sort_text: Option<String>,
        pub filter_text: Option<String>,
        pub insert_text: Option<String>,
        pub commit_characters: Option<Vec<String>>,
        pub data: Option<CompletionItemData>,
    }

    #[derive(Debug, Clone, Default)]
    pub struct CompletionItemData {
        pub file_name: String,
        pub position: i32,
        pub name: String,
        pub supplemental_file_index: Option<i32>,
    }

    #[derive(Debug, Clone)]
    pub enum Documentation {
        String(String),
        MarkupContent(MarkupContent),
    }
}

pub const SORT_TEXT_LOCATION_PRIORITY: &str = "\u{0}0";
pub const SORT_TEXT_GLOBALS_OR_KEYWORDS: &str = "\u{0}9";

pub fn deprecate_sort_text(original: &str) -> String { ::tsox_core::fntrace::enter("deprecate_sort_text"); 
    format!("z{}", original)
}

pub fn object_literal_property_sort_text(
    preset_sort_text: &str,
    symbol_display_name: &str,
) -> String { ::tsox_core::fntrace::enter("object_literal_property_sort_text"); 
    format!("{}\u{0}{}\u{0}", preset_sort_text, symbol_display_name)
}

pub fn sort_below(original: &str) -> String { ::tsox_core::fntrace::enter("sort_below"); 
    format!("{}1", original)
}

pub fn get_default_commit_characters(is_new_identifier_location: bool) -> Vec<String> { ::tsox_core::fntrace::enter("get_default_commit_characters"); 
    if is_new_identifier_location {
        return Vec::new();
    }
    all_commit_characters().to_vec()
}

pub fn all_commit_characters() -> &'static [String] { ::tsox_core::fntrace::enter("all_commit_characters"); 
    static CHARS: std::sync::OnceLock<Vec<String>> = std::sync::OnceLock::new();
    CHARS.get_or_init(|| {
        ['.', ',', ';']
            .iter()
            .map(|c| c.to_string())
            .collect()
    })
}

pub fn completion_name_for_literal(
    file: &Arc<SourceFile>,
    preferences: &UserPreferences,
    literal: &LiteralValue,
) -> String { ::tsox_core::fntrace::enter("completion_name_for_literal"); 
    match literal {
        LiteralValue::String(s) => quote(file, preferences, s),
        LiteralValue::Number(n) => tsox_core::core::mig::m3j::stringify_json(n, "", "").unwrap_or_default(),
        LiteralValue::PseudoBigInt(b) => format!("{}n", b.to_string()),
    }
}

pub enum LiteralValue {
    String(String),
    Number(f64),
    PseudoBigInt(tsox_core::jsnum::PseudoBigInt),
}

pub fn create_completion_item_for_literal(
    file: &Arc<SourceFile>,
    preferences: &UserPreferences,
    literal: &LiteralValue,
) -> lsproto::CompletionItem { ::tsox_core::fntrace::enter("create_completion_item_for_literal"); 
    let mut item = lsproto::CompletionItem::default();
    item.label = completion_name_for_literal(file, preferences, literal);
    item.kind = Some(lsproto::CompletionItemKind::empty_value());
    item.sort_text = Some(SORT_TEXT_LOCATION_PRIORITY.to_string());
    item.commit_characters = Some(Vec::new());
    item
}

pub fn create_modifier_list(
    factory: &NodeFactory,
    flags: ast::ModifierFlags,
    decorators: &[Arc<Node>],
) -> Option<Arc<ModifierList>> { ::tsox_core::fntrace::enter("create_modifier_list"); 
    let mut nodes: Vec<Arc<Node>> = Vec::new();
    for decorator in decorators {
        nodes.push(ast::deep_clone_node(decorator));
    }
    nodes.extend(tsox_frontend::ast::mig::w2::create_modifiers_from_modifier_flags(
        flags,
        &|kind| factory.new_modifier(kind),
    ));
    if nodes.is_empty() {
        return None;
    }
    Some(Arc::new(factory.new_modifier_list(nodes)))
}

pub fn create_snippet_tab_stop_body(
    factory: &NodeFactory,
    emit_context: &mut tsox_emit::printer::EmitContext,
) -> Option<Arc<Node>> { ::tsox_core::fntrace::enter("create_snippet_tab_stop_body"); 
    use tsox_emit::printer::mig::m4m_2::{SnippetElement, SNIPPET_KIND_TAB_STOP};
    let empty_statement = Arc::new(Node::new(SyntaxKind::EmptyStatement, NodeData::EmptyStatement));
    emit_context.set_snippet_element(
        &empty_statement,
        SnippetElement {
            kind: SNIPPET_KIND_TAB_STOP,
            order: 0,
        },
    );
    Some(new_block_m5q3(factory, vec![empty_statement], true))
}

pub fn get_dot_accessor(file: &Arc<SourceFile>, position: usize) -> String { ::tsox_core::fntrace::enter("get_dot_accessor"); 
    let text = &file.text;
    let before = &text[..position.min(text.len())];
    if before.ends_with("?.") {
        return text[position - 2..position].to_string();
    }
    if before.ends_with('.') {
        return text[position - 1..position].to_string();
    }
    String::new()
}

pub fn bool_to_ptr(v: bool) -> Option<bool> { ::tsox_core::fntrace::enter("bool_to_ptr"); 
    if v {
        Some(true)
    } else {
        Some(false)
    }
}

pub fn binary_expression_may_be_open_tag(binary_expression: &Node) -> bool { ::tsox_core::fntrace::enter("binary_expression_may_be_open_tag"); 
    match &binary_expression.data {
        NodeData::BinaryExpression(d) => ast::node_is_missing(Some(&d.left)),
        _ => true,
    }
}

pub fn get_first_symbol_in_chain(
    symbol: &Arc<Symbol>,
    enclosing_declaration: &Arc<Node>,
    type_checker: &mut Checker,
) -> Option<Arc<Symbol>> { ::tsox_core::fntrace::enter("get_first_symbol_in_chain"); 
    let chain = type_checker.get_accessible_symbol_chain_public(
        symbol,
        Some(enclosing_declaration),
        ast::SymbolFlags::all(),
        false,
    );
    if let Some(first) = chain.first() {
        return Some(Arc::clone(first));
    }
    let parent = symbol.parent()?;
    if is_module_symbol(&parent) {
        return Some(Arc::clone(symbol));
    }
    get_first_symbol_in_chain(&parent, enclosing_declaration, type_checker)
}

pub fn is_module_symbol(symbol: &Symbol) -> bool { ::tsox_core::fntrace::enter("is_module_symbol"); 
    symbol.flags.intersects(ast::SymbolFlags::MODULE)
}

pub fn get_contextual_type_for_conditional_expression(
    conditional_expr: &Arc<Node>,
    position: i32,
    file: &Arc<SourceFile>,
    type_checker: &mut Checker,
) -> Option<Arc<Type>> { ::tsox_core::fntrace::enter("get_contextual_type_for_conditional_expression"); 
    if let Some(arg_info) = get_argument_info_for_completions(conditional_expr, position, file, type_checker) {
        return type_checker.get_contextual_type_for_argument_at_index(
            &arg_info.invocation,
            arg_info.argument_index,
        );
    }
    if let Some(contextual_type) =
        type_checker.get_contextual_type(conditional_expr, tsox_checker::checker::ContextFlags::IgnoreNodeInferences)
    {
        return Some(contextual_type);
    }
    type_checker.get_contextual_type(conditional_expr, tsox_checker::checker::ContextFlags::None)
}

pub struct ArgumentInfoForCompletions {
    pub invocation: Arc<Node>,
    pub argument_index: usize,
    pub argument_count: usize,
}

pub fn get_argument_info_for_completions(
    node: &Arc<Node>,
    position: i32,
    file: &Arc<SourceFile>,
    type_checker: &mut Checker,
) -> Option<ArgumentInfoForCompletions> { ::tsox_core::fntrace::enter("get_argument_info_for_completions"); 
    let info = crate::ls::mig::m5w_2::get_immediately_containing_argument_info(
        node,
        position.max(0) as usize,
        file,
        type_checker,
    )?;
    if info.is_type_parameter_list || info.invocation.call_invocation.is_none() {
        return None;
    }
    Some(ArgumentInfoForCompletions {
        invocation: info.invocation.call_invocation.as_ref()?.node.clone(),
        argument_index: info.argument_index,
        argument_count: info.argument_count,
    })
}

pub fn get_closest_symbol_declaration(
    context_token: Option<&Arc<Node>>,
    location: &Arc<Node>,
) -> Option<Arc<Node>> { ::tsox_core::fntrace::enter("get_closest_symbol_declaration"); 
    let context_token = context_token?;
    tsox_frontend::ast::mig::m3e_4::find_ancestor_or_quit(Some(context_token), &|node: &Arc<Node>| {
        use tsox_frontend::ast::mig::m3e_4::FindAncestorResult;
        if ast::is_function_block(node) || is_arrow_function_body(node) || ast::is_binding_pattern(node) {
            return FindAncestorResult::Quit;
        }
        if (ast::is_parameter_declaration(node) || ast::is_type_parameter_declaration(node))
            && !node.parent().is_some_and(|p| ast::is_index_signature_declaration(&p))
        {
            return FindAncestorResult::True;
        }
        FindAncestorResult::False
    })
}

pub fn is_arrow_function_body(node: &Arc<Node>) -> bool { ::tsox_core::fntrace::enter("is_arrow_function_body"); 
    let Some(parent) = node.parent() else {
        return false;
    };
    if parent.kind != SyntaxKind::ArrowFunction {
        return false;
    }
    matches!(&parent.data,
        NodeData::ArrowFunction(d) if Arc::ptr_eq(&d.body, node))
}

pub fn create_range_from_string_literal_like_content(
    ls: &crate::ls::language_service::LanguageService,
    file: &Arc<SourceFile>,
    node: &Arc<Node>,
    position: i32,
) -> Option<lsproto::Range> { ::tsox_core::fntrace::enter("create_range_from_string_literal_like_content"); 
    let position = position.max(0) as usize;
    let mut replacement_end = node.end().saturating_sub(1);
    let node_start = tsox_frontend::astnav::get_start_of_node(node, file, false);
    if tsox_frontend::ast::mig::m3g_3::is_unterminated_literal(node) {
        if node_start == replacement_end {
            return None;
        }
        replacement_end = position.min(node.end());
    }
    let script_view = crate::mig::m5u_conv::SourceFileScriptView { file: Arc::clone(file) };
    let (lsp_range, fidelity) = m5q3_m5u_converters().to_lsp_range(
        &script_view,
        tsox_core::core::text::TextRange::new(node_start + 1, replacement_end),
    );
    if !crate::ls::mig::m5u_3::fidelity_is_exact(&fidelity) {
        return None;
    }
    Some(lsp_range)
}

pub fn are_intersected_types_avoiding_string_reduction(
    type_checker: &Checker,
    t1: &Arc<Type>,
    t2: &Arc<Type>,
) -> bool { ::tsox_core::fntrace::enter("are_intersected_types_avoiding_string_reduction"); 
    t1.is_string() && type_checker.is_empty_anonymous_object_type(t2)
}

pub fn is_string_and_empty_anonymous_object_intersection(
    type_checker: &Checker,
    t: &Arc<Type>,
) -> bool { ::tsox_core::fntrace::enter("is_string_and_empty_anonymous_object_intersection"); 
    if !t.is_intersection() {
        return false;
    }
    let types = t.types().unwrap_or(&[]);
    types.len() == 2
        && (are_intersected_types_avoiding_string_reduction(type_checker, &types[0], &types[1])
            || are_intersected_types_avoiding_string_reduction(type_checker, &types[1], &types[0]))
}

pub fn escape_snippet_text(text: &str) -> String { ::tsox_core::fntrace::enter("escape_snippet_text"); 
    text.replace('$', "\\$")
}

pub fn get_completions_symbol_kind(kind: crate::ls::lsutil_symbol_display::ScriptElementKind) -> lsproto::CompletionItemKind { ::tsox_core::fntrace::enter("get_completions_symbol_kind"); 
    use crate::ls::lsutil_symbol_display::ScriptElementKind as K;
    match kind {
        K::PrimitiveType | K::Keyword => lsproto::CompletionItemKind::Keyword,
        K::ConstElement | K::LetElement | K::VariableElement | K::LocalVariableElement
        | K::Alias | K::ParameterElement => lsproto::CompletionItemKind::Variable,
        K::MemberVariableElement | K::MemberGetAccessorElement | K::MemberSetAccessorElement => {
            lsproto::CompletionItemKind::Field
        }
        K::FunctionElement | K::LocalFunctionElement => lsproto::CompletionItemKind::Function,
        K::MemberFunctionElement | K::ConstructSignatureElement | K::CallSignatureElement
        | K::IndexSignatureElement => lsproto::CompletionItemKind::Method,
        K::EnumElement => lsproto::CompletionItemKind::Enum,
        K::EnumMemberElement => lsproto::CompletionItemKind::EnumMember,
        K::ModuleElement | K::ExternalModuleName => lsproto::CompletionItemKind::Module,
        K::ClassElement | K::TypeElement => lsproto::CompletionItemKind::Class,
        K::InterfaceElement => lsproto::CompletionItemKind::Interface,
        K::Warning => lsproto::CompletionItemKind::Text,
        K::ScriptElement => lsproto::CompletionItemKind::File,
        K::Directory => lsproto::CompletionItemKind::Folder,
        K::String => lsproto::CompletionItemKind::Constant,
        _ => lsproto::CompletionItemKind::Property,
    }
}

pub fn clone_items(items: &[lsproto::CompletionItem]) -> Vec<lsproto::CompletionItem> { ::tsox_core::fntrace::enter("clone_items"); 
    items.to_vec()
}

pub fn get_apparent_properties(
    t: &Arc<Type>,
    node: &Arc<Node>,
    type_checker: &mut Checker,
) -> Vec<Arc<Symbol>> { ::tsox_core::fntrace::enter("get_apparent_properties"); 
    if !t.is_union() {
        return type_checker.get_apparent_properties(t);
    }
    let member_types: Vec<Arc<Type>> = t
        .types()
        .unwrap_or(&[])
        .iter()
        .filter(|member_type| {
            !(member_type.flags
                & tsox_checker::checker::TYPE_FLAGS_PRIMITIVE
                != tsox_checker::checker::TypeFlags::None
                || type_checker.is_array_like_type(member_type)
                || type_checker.is_type_invalid_due_to_union_discriminant(member_type, node)
                || type_checker.type_has_call_or_construct_signatures(member_type)
                || (member_type.is_class()
                    && contains_non_public_properties(&type_checker.get_apparent_properties(member_type))))
        })
        .cloned()
        .collect();
    type_checker.get_all_possible_properties_of_types(&member_types)
}

pub fn contains_non_public_properties(props: &[Arc<Symbol>]) -> bool { ::tsox_core::fntrace::enter("contains_non_public_properties"); 
    props.iter().any(|p| {
        tsox_checker::checker::get_declaration_modifier_flags_from_symbol(p)
            .intersects(ast::ModifierFlags::NonPublicAccessibilityModifier)
    })
}

fn completion_item_capability_pointer(
    ctx: &lsproto::Context,
    leaf: &str,
) -> Option<bool> { ::tsox_core::fntrace::enter("completion_item_capability_pointer"); 
    lsproto::get_client_capabilities(ctx)
        .raw
        .pointer(&format!("/textDocument/completion/completionItem/{leaf}"))
        .and_then(|v| v.as_bool())
}

fn completion_list_item_defaults(ctx: &lsproto::Context) -> Vec<String> { ::tsox_core::fntrace::enter("completion_list_item_defaults"); 
    lsproto::get_client_capabilities(ctx)
        .raw
        .pointer("/textDocument/completion/completionList/itemDefaults")
        .and_then(|v| v.as_array())
        .map(|array| {
            array
                .iter()
                .filter_map(|v| v.as_str().map(|s| s.to_string()))
                .collect()
        })
        .unwrap_or_default()
}

pub fn client_supports_item_label_details(ctx: &lsproto::Context) -> bool { ::tsox_core::fntrace::enter("client_supports_item_label_details"); 
    completion_item_capability_pointer(ctx, "labelDetailsSupport").unwrap_or(false)
}

pub fn client_supports_item_snippet(ctx: &lsproto::Context) -> bool { ::tsox_core::fntrace::enter("client_supports_item_snippet"); 
    completion_item_capability_pointer(ctx, "snippetSupport").unwrap_or(false)
}

pub fn client_supports_item_commit_characters(ctx: &lsproto::Context) -> bool { ::tsox_core::fntrace::enter("client_supports_item_commit_characters"); 
    completion_item_capability_pointer(ctx, "commitCharactersSupport").unwrap_or(false)
}

pub fn client_supports_item_insert_replace(ctx: &lsproto::Context) -> bool { ::tsox_core::fntrace::enter("client_supports_item_insert_replace"); 
    completion_item_capability_pointer(ctx, "insertReplaceSupport").unwrap_or(false)
}

pub fn client_supports_default_commit_characters(ctx: &lsproto::Context) -> bool { ::tsox_core::fntrace::enter("client_supports_default_commit_characters"); 
    completion_list_item_defaults(ctx).iter().any(|d| d == "commitCharacters")
}

pub fn client_supports_default_edit_range(ctx: &lsproto::Context) -> bool { ::tsox_core::fntrace::enter("client_supports_default_edit_range"); 
    completion_list_item_defaults(ctx).iter().any(|d| d == "editRange")
}

pub fn get_completion_documentation_format(ctx: &lsproto::Context) -> lsproto::MarkupKind { ::tsox_core::fntrace::enter("get_completion_documentation_format"); 
    let formats: Vec<lsproto::MarkupKind> = lsproto::get_client_capabilities(ctx)
        .raw
        .pointer("/textDocument/completion/completionItem/documentationFormat")
        .and_then(|v| v.as_array())
        .map(|array| {
            array
                .iter()
                .filter_map(|v| match v.as_str()? {
                    "markdown" => Some(lsproto::MarkupKind::Markdown),
                    "plaintext" => Some(lsproto::MarkupKind::PlainText),
                    _ => None,
                })
                .collect()
        })
        .unwrap_or_default();
    lsproto::preferred_markup_kind(&formats)
}

pub fn create_simple_details<'a>(
    item: &'a mut lsproto::CompletionItem,
    name: &str,
    doc_format: lsproto::MarkupKind,
) -> &'a mut lsproto::CompletionItem { ::tsox_core::fntrace::enter("create_simple_details"); 
    create_completion_details(item, name, "", doc_format)
}

pub fn create_completion_details<'a>(
    item: &'a mut lsproto::CompletionItem,
    detail: &str,
    documentation: &str,
    doc_format: lsproto::MarkupKind,
) -> &'a mut lsproto::CompletionItem { ::tsox_core::fntrace::enter("create_completion_details"); 
    if item.detail.is_none() && !detail.is_empty() {
        item.detail = Some(detail.to_string());
    }
    if !documentation.is_empty() {
        item.documentation = Some(lsproto::Documentation::MarkupContent(lsproto::MarkupContent {
            kind: doc_format,
            value: documentation.to_string(),
        }));
    }
    item
}

impl crate::ls::language_service::LanguageService {
    pub fn create_completion_details_for_symbol<'a>(
        &self,
        item: &'a mut lsproto::CompletionItem,
        symbol: &Arc<Symbol>,
        checker: &mut Checker,
        location: &Arc<Node>,
        position: i32,
        doc_format: lsproto::MarkupKind,
    ) -> &'a mut lsproto::CompletionItem { ::tsox_core::fntrace::enter("create_completion_details_for_symbol"); 
        let (quick_info, documentation, _, _) = self.get_quick_info_and_documentation_for_symbol(
            checker,
            Some(symbol),
            location,
            super::m5w_3::markup_kind_str(&doc_format),
            None,
            false,
        );
        create_completion_details(item, &quick_info, &documentation, doc_format)
    }

    pub fn resolve_completion_item(
        &self,
        item: &lsproto::CompletionItem,
        data: &lsproto::CompletionItemData,
    ) -> Result<lsproto::CompletionItem, String> { ::tsox_core::fntrace::enter("resolve_completion_item"); 
        let (program, file) = self.try_get_program_and_file(&data.file_name);
        let file = match file {
            Some(file) => file,
            None => return Err(format!("file not found: {}", data.file_name)),
        };
        let file = match source_file_for_supplemental_file_index(&file, data.supplemental_file_index) {
            Some(file) => file,
            None => return Err(format!(
                "supplemental source file index not found: {:?}",
                data.supplemental_file_index
            )),
        };
        let checker = program.get_type_checker_for_file(&file);
        Ok(self.get_completion_item_details(&program, &checker, data.position, &file, item, data))
    }

    /// Go completions.go getCompletionItemDetails 的浅层落位：字符串字面量位、
    /// auto-import 编辑与 getSymbolCompletionFromItemData 符号重放所依赖的
    /// completions 管线（completionData 载荷/getStringLiteralCompletionDetails/
    /// autoimport Fix Edits）尚未移植，见 progress_notes_r59A.md 交接；
    /// 无载荷分支按 Go isImportStatementCompletion 语义原样返回 item
    pub fn get_completion_item_details(
        &self,
        _program: &tsox_compile::compiler::Program,
        _checker: &Checker,
        _position: i32,
        _file: &Arc<SourceFile>,
        item: &lsproto::CompletionItem,
        _data: &lsproto::CompletionItemData,
    ) -> lsproto::CompletionItem { ::tsox_core::fntrace::enter("get_completion_item_details"); 
        item.clone()
    }
}

pub const SOURCE_THIS_PROPERTY: &str = "ThisProperty/";
pub const SOURCE_CLASS_MEMBER_SNIPPET: &str = "ClassMemberSnippet/";
pub const SOURCE_TYPE_ONLY_ALIAS: &str = "TypeOnlyAlias/";
pub const SOURCE_OBJECT_LITERAL_METHOD_SNIPPET: &str = "ObjectLiteralMethodSnippet/";
pub const SOURCE_SWITCH_CASES: &str = "SwitchCases/";
pub const SOURCE_OBJECT_LITERAL_MEMBER_WITH_COMMA: &str = "ObjectLiteralMemberWithComma/";

pub fn could_be_type_only_import_specifier(
    import_specifier: &Arc<Node>,
    context_token: Option<&Arc<Node>>,
) -> bool { ::tsox_core::fntrace::enter("could_be_type_only_import_specifier"); 
    ast::is_import_specifier(import_specifier)
        && (tsox_frontend::ast::mig::m3b::is_type_only(import_specifier)
            || context_token.is_some()
                && import_specifier
                    .name()
                    .is_some_and(|n| Arc::ptr_eq(n, context_token.unwrap()))
                && is_type_keyword_token_or_identifier(context_token.unwrap()))
}

pub fn is_type_keyword_token_or_identifier(node: &Arc<Node>) -> bool { ::tsox_core::fntrace::enter("is_type_keyword_token_or_identifier"); 
    matches!(
        node.kind,
        SyntaxKind::TypeKeyword | SyntaxKind::Identifier
    )
}

pub fn can_complete_from_named_bindings(named_bindings: &Arc<Node>) -> bool { ::tsox_core::fntrace::enter("can_complete_from_named_bindings"); 
    let Some(import_declaration) = named_bindings.parent().and_then(|clause| clause.parent()) else {
        return false;
    };
    if !is_module_specifier_missing_or_empty(node_module_specifier(&import_declaration).as_ref())
        || named_bindings
            .parent()
            .and_then(|clause| clause.name().cloned())
            .is_some()
    {
        return false;
    }
    if ast::is_named_imports(named_bindings) {
        let invalid_named_import = get_potentially_invalid_import_specifier(named_bindings);
        let elements = tsox_frontend::ast::mig::m3b::elements(named_bindings);
        let mut valid_imports = elements.len() as i64;
        if let Some(invalid) = invalid_named_import {
            valid_imports = elements
                .iter()
                .position(|e| Arc::ptr_eq(e, &invalid))
                .map(|i| i as i64)
                .unwrap_or(-1);
        }
        return valid_imports < 2 && valid_imports > -1;
    }
    true
}

pub fn is_module_specifier_missing_or_empty(specifier: Option<&Arc<Node>>) -> bool { ::tsox_core::fntrace::enter("is_module_specifier_missing_or_empty"); 
    match specifier {
        None => true,
        Some(specifier) => ast::is_string_literal(specifier) && specifier.text().is_empty(),
    }
}

pub fn get_potentially_invalid_import_specifier(named_bindings: &Arc<Node>) -> Option<Arc<Node>> { ::tsox_core::fntrace::enter("get_potentially_invalid_import_specifier"); 
    let elements = tsox_frontend::ast::mig::m3b::elements(named_bindings);
    elements
        .first()
        .and_then(|element| match &element.data {
            tsox_frontend::ast::node_data_generated::NodeData::ImportSpecifier(d) => {
                d.property_name.clone()
            }
            _ => None,
        })
}

fn node_module_specifier(node: &Arc<Node>) -> Option<Arc<Node>> { ::tsox_core::fntrace::enter("node_module_specifier"); 
    match &node.data {
        tsox_frontend::ast::node_data_generated::NodeData::ImportDeclaration(d) => {
            Some(d.module_specifier.clone())
        }
        tsox_frontend::ast::node_data_generated::NodeData::ExportDeclaration(d) => {
            d.module_specifier.clone()
        }
        _ => None,
    }
}

pub fn get_jsdoc_tag_name_completions() -> Vec<lsproto::CompletionItem> { ::tsox_core::fntrace::enter("get_jsdoc_tag_name_completions"); 
    jsdoc_tag_items_m5q3(false)
}

pub fn get_jsdoc_tag_completions() -> Vec<lsproto::CompletionItem> { ::tsox_core::fntrace::enter("get_jsdoc_tag_completions"); 
    jsdoc_tag_items_m5q3(true)
}

pub fn get_jsdoc_parameter_name_completions(
    tag: &Arc<Node>,
) -> Vec<lsproto::CompletionItem> { ::tsox_core::fntrace::enter("get_jsdoc_parameter_name_completions"); 
    let Some(name) = tag.name() else {
        return Vec::new();
    };
    if !ast::is_identifier(name) {
        return Vec::new();
    }
    let name_thus_far = name.text();
    let Some(js_doc) = tag.parent() else {
        return Vec::new();
    };
    let Some(fn_node) = js_doc.parent() else {
        return Vec::new();
    };
    if !ast::is_function_like(&fn_node) {
        return Vec::new();
    }
    let tags: Vec<Arc<Node>> = match &js_doc.data {
        NodeData::JSDoc(d) => d
            .tags
            .as_ref()
            .map(|t| t.nodes.clone())
            .unwrap_or_default(),
        _ => Vec::new(),
    };
    let parameters = tsox_frontend::ast::mig::m3b::parameters(&fn_node);
    parameters
        .iter()
        .filter_map(|param| {
            let param_name = param.name()?;
            if !ast::is_identifier(param_name) {
                return None;
            }
            let name = param_name.text();
            let tagged_elsewhere = tags.iter().any(|t| {
                !Arc::ptr_eq(t, tag)
                    && t.kind == SyntaxKind::JSDocParameterTag
                    && t.name().is_some_and(|n| ast::is_identifier(n) && n.text() == name)
            });
            if tagged_elsewhere || (!name_thus_far.is_empty() && !name.starts_with(name_thus_far)) {
                return None;
            }
            let mut item = lsproto::CompletionItem::default();
            item.label = name.to_string();
            item.kind = Some(lsproto::CompletionItemKind::Variable);
            item.sort_text = Some(SORT_TEXT_LOCATION_PRIORITY.to_string());
            Some(item)
        })
        .collect()
}

pub fn entity_name_to_expression(
    entity_name: &Arc<Node>,
    target: tsox_core::core::compiler_options_kinds::ScriptTarget,
    quote_preference: QuotePreference,
    factory: &NodeFactory,
) -> Arc<Node> { ::tsox_core::fntrace::enter("entity_name_to_expression"); 
    if ast::is_identifier(entity_name) {
        return entity_name.clone();
    }
    let (left, right) = match &entity_name.data {
        NodeData::QualifiedName(d) => (d.left.clone(), d.right.clone()),
        _ => return entity_name.clone(),
    };
    new_property_access_expression_m5q3(
        factory,
        entity_name_to_expression(&left, target, quote_preference, factory),
        None,
        right,
    )
}

fn new_property_access_expression_m5q3(
    factory: &NodeFactory,
    expression: Arc<Node>,
    question_dot_token: Option<Arc<Node>>,
    name: Arc<Node>,
) -> Arc<Node> { ::tsox_core::fntrace::enter("new_property_access_expression_m5q3"); 
    let _ = factory;
    Arc::new(Node::new(
        SyntaxKind::PropertyAccessExpression,
        NodeData::PropertyAccessExpression(
            tsox_frontend::ast::node_data_generated::PropertyAccessExpressionData {
                expression,
                question_dot_token,
                name,
            },
        ),
    ))
}

pub struct SnippetPrinter {
    pub printer: tsox_frontend::format::mig::m4o_2::Printer,
    pub emit_context: tsox_frontend::format::mig::m4o_2::EmitContext,
}

impl SnippetPrinter {
    pub fn print_node(&mut self, node: &Arc<Node>) -> String { ::tsox_core::fntrace::enter("print_node"); 
        escape_snippet_text(&self.print_unescaped_node(node))
    }

    pub fn print_unescaped_node(&mut self, node: &Arc<Node>) -> String { ::tsox_core::fntrace::enter("print_unescaped_node"); 
        self.printer.emit(node, None)
    }
}

pub fn create_snippet_printer(
    options: tsox_frontend::format::mig::m4o_2::PrinterOptions,
    emit_context: Option<tsox_frontend::format::mig::m4o_2::EmitContext>,
) -> SnippetPrinter { ::tsox_core::fntrace::enter("create_snippet_printer"); 
    let emit_context = emit_context.unwrap_or_default();
    let printer = tsox_frontend::format::mig::m4o_2::new_printer(
        options,
        tsox_frontend::format::mig::m4o_2::PrintHandlers {
            has_global_name: None,
            map_source_position: None,
            on_before_emit_node: None,
            on_after_emit_node: None,
            on_before_emit_node_list: None,
            on_after_emit_node_list: None,
            on_before_emit_token: None,
            on_after_emit_token: None,
        },
        emit_context.clone(),
    );
    SnippetPrinter {
        printer,
        emit_context,
    }
}

pub fn source_file_for_supplemental_file_index(
    file: &Arc<SourceFile>,
    index: Option<i32>,
) -> Option<Arc<SourceFile>> { ::tsox_core::fntrace::enter("source_file_for_supplemental_file_index"); 
    match index {
        Some(index) => file
            .supplemental_source_files
            .get(index as usize)
            .cloned(),
        None => Some(file.clone()),
    }
}

pub fn quote(file: &Arc<SourceFile>, preferences: &UserPreferences, text: &str) -> String { ::tsox_core::fntrace::enter("quote"); 
    let quote_preference =
        crate::ls::lsutil_utilities::get_quote_preference(file, preferences);
    let quoted = tsox_core::core::mig::m3j::stringify_json(&text, "", "").unwrap_or_default();
    if quote_preference == QuotePreference::Single {
        let inner = quoted.trim_matches('"');
        format!("'{}'", quote_replacer(inner))
    } else {
        quoted
    }
}

fn quote_replacer(text: &str) -> String { ::tsox_core::fntrace::enter("quote_replacer"); 
    let mut out = String::with_capacity(text.len());
    for c in text.chars() {
        match c {
            '"' => out.push_str("\\\""),
            '\'' => out.push_str("\\'"),
            '`' => out.push_str("\\`"),
            other => out.push(other),
        }
    }
    out
}
