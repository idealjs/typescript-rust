#![allow(dead_code, unused_imports, unused_variables)]

use std::sync::Arc;

use crate::ls::lsutil_user_preferences::{QuotePreference, UserPreferences};
use tsox_checker::checker::Checker;
use tsox_core::core;
use tsox_frontend::ast::{self, Node, SourceFile, Symbol, SyntaxKind};
use tsox_frontend::ast::mig::m3b_2::NodeFactory;
use tsox_frontend::astnav;
use tsox_frontend::scanner;

use super::m5q2::{get_jsdoc_param_name_with_initializer, generate_jsdoc_param_tags_for_destructuring, ImportStatementCompletionInfo, get_filter_text};
use super::m5q_3::{escape_snippet_text, SORT_TEXT_LOCATION_PRIORITY, get_completions_symbol_kind};
use super::m5r::{str_ptr_to, get_dot_accessor as get_dot_accessor_m5r, get_word_length_and_start, is_module_specifier_missing_or_empty};
use super::m5q2b::SortText;
use super::m5q2b_2::{
    LSUTIL_SCRIPT_ELEMENT_KIND_MODIFIER_DEPRECATED, LSUTIL_SCRIPT_ELEMENT_KIND_MODIFIER_OPTIONAL,
    can_complete_from_named_bindings, could_be_type_only_import_specifier_m5q2b,
    get_potentially_invalid_import_specifier_m5q2b, lsutil_script_element_kind_modifier_flags,
    supplemental_file_index_m5q2b,
};

/// r58B：本片引用的 lsproto 缺失类型，按 Go internal/lsp/lsproto 最小等价移植，
/// 复用 `lsproto::` 路径写法；合并期归位到 crate::lsp::lsproto。
pub(crate) mod lsproto {
    pub use crate::lsp::lsproto::*;

    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub enum CompletionItemKind {
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
        Unit = 11,
        Value = 12,
        Enum = 13,
        Keyword = 14,
        Snippet = 15,
        Color = 16,
        File = 17,
        Reference = 18,
        Folder = 19,
        EnumMember = 20,
        Constant = 21,
        Struct = 22,
        Event = 23,
        Operator = 24,
        TypeParameter = 25,
    }

    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub enum CompletionItemTag {
        Deprecated = 1,
    }

    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub enum InsertTextFormat {
        PlainText = 1,
        Snippet = 2,
    }

    #[derive(Debug, Clone, Default, PartialEq, Eq)]
    pub struct CompletionItemLabelDetails {
        pub detail: Option<String>,
        pub description: Option<String>,
    }

    #[derive(Debug, Clone, Default)]
    pub struct InsertReplaceEdit {
        pub new_text: String,
        pub insert: crate::lsp::lsproto::Range,
        pub replace: crate::lsp::lsproto::Range,
    }

    #[derive(Debug, Clone, Default)]
    pub struct TextEditOrInsertReplaceEdit {
        pub text_edit: Option<crate::lsp::lsproto::TextEdit>,
        pub insert_replace_edit: Option<InsertReplaceEdit>,
    }

    /// Go lsproto.AutoImportFix。
    #[derive(Debug, Clone, Default)]
    pub struct AutoImportFix {
        pub kind: i32,
        pub name: String,
        pub import_kind: i32,
        pub use_require: bool,
        pub add_as_type_only: i32,
        pub module_specifier: String,
        pub import_index: i32,
        pub usage_position: Option<crate::lsp::lsproto::Position>,
        pub namespace_prefix: String,
    }

    /// Go lsproto.CompletionItemData。
    #[derive(Debug, Clone, Default)]
    pub struct CompletionItemData {
        pub file_name: String,
        pub position: i32,
        pub supplemental_file_index: Option<i32>,
        pub source: String,
        pub name: String,
        pub auto_import: Option<AutoImportFix>,
        pub is_import_statement_completion: bool,
    }

    /// Go lsproto.CompletionItem。
    #[derive(Debug, Clone, Default)]
    pub struct CompletionItem {
        pub label: String,
        pub label_details: Option<CompletionItemLabelDetails>,
        pub kind: Option<CompletionItemKind>,
        pub tags: Option<Vec<CompletionItemTag>>,
        pub detail: Option<String>,
        pub documentation: Option<crate::lsp::lsproto::StringOrMarkupContent>,
        pub deprecated: Option<bool>,
        pub preselect: Option<bool>,
        pub sort_text: Option<String>,
        pub filter_text: Option<String>,
        pub insert_text: Option<String>,
        pub insert_text_format: Option<InsertTextFormat>,
        pub text_edit: Option<TextEditOrInsertReplaceEdit>,
        pub text_edit_text: Option<String>,
        pub additional_text_edits: Option<Vec<crate::lsp::lsproto::TextEdit>>,
        pub commit_characters: Option<Vec<String>>,
        pub data: Option<CompletionItemData>,
    }

    /// Go lsproto.CompletionList。
    #[derive(Debug, Clone, Default)]
    pub struct CompletionList {
        pub is_incomplete: bool,
        pub items: Vec<CompletionItem>,
    }
}

impl crate::ls::language_service::LanguageService {
    pub fn create_lsp_completion_item(
        &self,
        name: &str,
        insert_text: &str,
        filter_text: &str,
        sort_text: SortText,
        element_kind: crate::ls::lsutil_symbol_display::ScriptElementKind,
        kind_modifiers: Vec<String>,
        replacement_span: Option<lsproto::Range>,
        commit_characters: Option<Vec<String>>,
        label_details: Option<lsproto::CompletionItemLabelDetails>,
        file: &Arc<SourceFile>,
        position: usize,
        is_member_completion: bool,
        is_snippet: bool,
        has_action: bool,
        preselect: bool,
        source: &str,
        auto_import_fix: Option<&lsproto::AutoImportFix>,
        additional_text_edits: Option<&Vec<lsproto::TextEdit>>,
        detail: Option<&str>,
    ) -> lsproto::CompletionItem {
        let kind = get_completions_symbol_kind_m5q2b3(element_kind);
        let mut name = name.to_string();
        let mut insert_text = insert_text.to_string();
        let mut filter_text = filter_text.to_string();
        let mut insert_text_format: Option<lsproto::InsertTextFormat> = None;

        let mut text_edit: Option<lsproto::TextEditOrInsertReplaceEdit> = None;
        if let Some(replacement_span) = &replacement_span {
            text_edit = Some(lsproto::TextEditOrInsertReplaceEdit {
                text_edit: Some(lsproto::TextEdit {
                    new_text: if insert_text.is_empty() {
                        name.clone()
                    } else {
                        insert_text.clone()
                    },
                    range: replacement_span.clone(),
                }),
                insert_replace_edit: None,
            });
        }

        let (word_size, word_start) = get_word_length_and_start(file, position);
        let dot_accessor = get_dot_accessor_m5r(file, position - word_size);
        if filter_text.is_empty() {
            filter_text = get_filter_text(
                file,
                position,
                &insert_text,
                &name,
                word_start,
                &dot_accessor,
            );
        }

        let mut tags: Option<Vec<lsproto::CompletionItemTag>> = None;
        let kind_modifiers_flags = lsutil_script_element_kind_modifier_flags(&kind_modifiers);
        if is_member_completion && kind_modifiers_flags & LSUTIL_SCRIPT_ELEMENT_KIND_MODIFIER_OPTIONAL != 0 {
            if insert_text.is_empty() {
                insert_text = name.clone();
            }
            if filter_text.is_empty() || is_snippet {
                filter_text = name.clone();
            }
            name.push('?');
        }
        if kind_modifiers_flags & LSUTIL_SCRIPT_ELEMENT_KIND_MODIFIER_DEPRECATED != 0 {
            tags = Some(vec![lsproto::CompletionItemTag::Deprecated]);
        }

        if is_snippet {
            insert_text_format = Some(lsproto::InsertTextFormat::Snippet);
        }

        let mut item = lsproto::CompletionItem::default();
        item.label = name;
        item.label_details = label_details;
        item.kind = Some(kind);
        item.tags = tags;
        item.detail = detail.map(|d| d.to_string());
        item.preselect = Some(preselect);
        item.sort_text = Some(sort_text);
        item.filter_text = str_ptr_to(&filter_text);
        item.insert_text = str_ptr_to(&insert_text);
        item.insert_text_format = insert_text_format;
        item.text_edit = text_edit;
        item.commit_characters = commit_characters;
        item.additional_text_edits = additional_text_edits.cloned();
        item.data = Some(lsproto::CompletionItemData {
            file_name: ast::mig::m3b_2::original_file_name(file).to_string(),
            position: position as i32,
            supplemental_file_index: supplemental_file_index_m5q2b(file),
            source: source.to_string(),
            name: item.label.clone(),
            auto_import: auto_import_fix.cloned(),
            is_import_statement_completion: false,
            ..Default::default()
        });
        item
    }

    pub fn get_import_statement_completion_info(
        &self,
        context_token: &Arc<Node>,
        source_file: &Arc<SourceFile>,
    ) -> ImportStatementCompletionInfo { ::tsox_core::fntrace::enter("get_import_statement_completion_info"); 
        let mut result = ImportStatementCompletionInfo::default();
        let mut candidate: Option<Arc<Node>> = None;
        let parent = context_token.parent();
        if let Some(parent) = parent.as_ref() {
            if ast::is_import_equals_declaration(parent) {
                let last_token = crate::ls::lsutil_children::get_last_token(Some(parent), source_file);
                if context_token.kind == SyntaxKind::Identifier
                    && !last_token.map(|t| Arc::ptr_eq(&t, context_token)).unwrap_or(false)
                {
                    result.keyword_completion = Some(SyntaxKind::FromKeyword);
                    result.is_keyword_only_completion = true;
                } else {
                    if context_token.kind != SyntaxKind::TypeKeyword {
                        result.keyword_completion = Some(SyntaxKind::TypeKeyword);
                    }
                    let module_reference = match &parent.data {
                        ast::node_data_generated::NodeData::ImportEqualsDeclaration(d) => {
                            Some(Arc::clone(&d.module_reference))
                        }
                        _ => None,
                    };
                    if module_reference
                        .as_ref()
                        .is_some_and(|m| is_module_specifier_missing_or_empty(Some(m)))
                    {
                        candidate = Some(Arc::clone(parent));
                    }
                }
            } else if could_be_type_only_import_specifier_m5q2b(parent, context_token)
                && parent
                    .parent()
                    .as_ref()
                    .is_some_and(|nb| can_complete_from_named_bindings(nb))
            {
                candidate = Some(Arc::clone(parent));
            } else if ast::is_named_imports(parent) || ast::is_namespace_import(parent) {
                if !parent
                    .parent()
                    .as_ref()
                    .is_some_and(|pp| node_is_type_only_m5q2b3(pp))
                    && (context_token.kind == SyntaxKind::OpenBraceToken
                        || context_token.kind == SyntaxKind::ImportKeyword
                        || context_token.kind == SyntaxKind::CommaToken)
                {
                    result.keyword_completion = Some(SyntaxKind::TypeKeyword);
                }
                if can_complete_from_named_bindings(parent) {
                    if context_token.kind == SyntaxKind::CloseBraceToken
                        || context_token.kind == SyntaxKind::Identifier
                    {
                        result.is_keyword_only_completion = true;
                        result.keyword_completion = Some(SyntaxKind::FromKeyword);
                    } else {
                        candidate = parent.parent().and_then(|p| p.parent());
                    }
                }
            } else if (ast::is_export_declaration(parent)
                && context_token.kind == SyntaxKind::AsteriskToken)
                || (ast::is_named_exports(parent)
                    && context_token.kind == SyntaxKind::CloseBraceToken)
            {
                result.is_keyword_only_completion = true;
                result.keyword_completion = Some(SyntaxKind::FromKeyword);
            } else if context_token.kind == SyntaxKind::ImportKeyword {
                if ast::is_source_file(parent) {
                    result.keyword_completion = Some(SyntaxKind::TypeKeyword);
                    candidate = Some(Arc::clone(context_token));
                } else if ast::is_import_declaration(parent) {
                    result.keyword_completion = Some(SyntaxKind::TypeKeyword);
                    if ast::mig::m3b::module_specifier(parent)
                        .is_some_and(|m| is_module_specifier_missing_or_empty(Some(m)))
                    {
                        candidate = Some(Arc::clone(parent));
                    }
                }
            }
        }

        if let Some(candidate) = candidate {
            result.is_new_identifier_location = true;
            result.replacement_span =
                self.get_single_line_replacement_span_for_import_completion_node(&candidate);
            result.could_be_type_only_import_specifier =
                could_be_type_only_import_specifier_m5q2b(&candidate, context_token);
            if ast::is_import_declaration(&candidate) {
                if let Some(import_clause) = import_clause_of_m5q2b3(&candidate) {
                    result.is_top_level_type_only = node_is_type_only_m5q2b3(&import_clause);
                }
            } else if candidate.kind == SyntaxKind::ImportEqualsDeclaration {
                result.is_top_level_type_only = node_is_type_only_m5q2b3(&candidate);
            }
        } else {
            result.is_new_identifier_location = result.keyword_completion == Some(SyntaxKind::TypeKeyword);
        }
        result
    }

    pub fn get_single_line_replacement_span_for_import_completion_node(
        &self,
        node: &Arc<Node>,
    ) -> Option<lsproto::Range> { ::tsox_core::fntrace::enter("get_single_line_replacement_span_for_import_completion_node"); 
        let mut node = Arc::clone(node);
        if let Some(ancestor) = ast::find_ancestor(&node, |n| {
            ast::is_import_declaration(n) || ast::is_import_equals_declaration(n) || ast::is_jsdoc_import_tag(n)
        }) {
            node = ancestor;
        }
        let source_file = crate::ls::api::get_source_file_of_node(&node)?;
        let token_pos = scanner::mig::x5a::get_token_pos_of_node(&node, &source_file, false);
        if tsox_frontend::format::mig::m4t_3::get_lines_between_positions(&source_file, token_pos as i64, node.end() as i64) == 0 {
            let (lsp_range, fidelity) = self.create_lsp_range_from_node(&node, &source_file);
            if fidelity != 0 {
                return None;
            }
            return Some(lsp_range);
        }

        if node.kind == SyntaxKind::ImportKeyword || node.kind == SyntaxKind::ImportSpecifier {
            panic!("ImportKeyword was necessarily on one line; ImportSpecifier was necessarily parented in an ImportDeclaration");
        }

        let potential_split_point: Arc<Node>;
        if node.kind == SyntaxKind::ImportDeclaration || node.kind == SyntaxKind::JSDocImportTag {
            let specifier = import_clause_of_m5q2b3(&node).and_then(|import_clause| {
                let named_bindings = match &import_clause.data {
                    ast::node_data_generated::NodeData::ImportClause(d) => d.named_bindings.clone(),
                    _ => None,
                };
                named_bindings
                    .as_ref()
                    .and_then(|nb| get_potentially_invalid_import_specifier_m5q2b(Some(nb)))
            });
            potential_split_point = match specifier {
                Some(specifier) => specifier,
                None => ast::mig::m3b::module_specifier(&node)?.clone(),
            };
        } else {
            potential_split_point = match &node.data {
                ast::node_data_generated::NodeData::ImportEqualsDeclaration(d) => {
                    Arc::clone(&d.module_reference)
                }
                _ => panic!("node is not an import equals declaration"),
            };
        }

        let first_token = crate::ls::lsutil_children::get_first_token(&node, &source_file)?;
        let without_module_specifier = core::text::TextRange::new(
            scanner::mig::x5a::get_token_pos_of_node(&first_token, &source_file, false),
            potential_split_point.pos(),
        );
        if tsox_frontend::format::mig::m4t_3::get_lines_between_positions(&source_file, without_module_specifier.pos() as i64, without_module_specifier.end() as i64) == 0 {
            let (lsp_range, fidelity) = self.m5x_create_lsp_range_from_bounds(
                without_module_specifier.pos(),
                without_module_specifier.end(),
                &source_file,
            );
            if fidelity != 0 {
                return None;
            }
            return Some(lsp_range);
        }
        None
    }

}

fn get_completions_symbol_kind_m5q2b3(
    kind: crate::ls::lsutil_symbol_display::ScriptElementKind,
) -> lsproto::CompletionItemKind { ::tsox_core::fntrace::enter("get_completions_symbol_kind_m5q2b3"); 
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

fn node_is_type_only_m5q2b3(node: &Arc<Node>) -> bool { ::tsox_core::fntrace::enter("node_is_type_only_m5q2b3"); 
    match &node.data {
        ast::node_data_generated::NodeData::ImportClause(d) => {
            d.phase_modifier == Some(SyntaxKind::TypeKeyword)
        }
        ast::node_data_generated::NodeData::ImportEqualsDeclaration(d) => d.is_type_only,
        ast::node_data_generated::NodeData::ImportSpecifier(d) => d.is_type_only,
        _ => false,
    }
}

fn import_clause_of_m5q2b3(node: &Arc<Node>) -> Option<Arc<Node>> { ::tsox_core::fntrace::enter("import_clause_of_m5q2b3"); 
    match &node.data {
        ast::node_data_generated::NodeData::ImportDeclaration(d) => d.import_clause.clone(),
        _ => None,
    }
}
