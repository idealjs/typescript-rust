use crate::ast::{Node, NodeFlags, SourceFile, SyntaxKind};
use crate::scanner::impl_chunk::*;
use crate::scanner::{
    is_hex_digit, keywords, ErrorCallback, Scanner, ScannerError, DiagnosticKind,
    TOKEN_FLAGS_EXTENDED_UNICODE_ESCAPE, TOKEN_FLAGS_NONE, TOKEN_FLAGS_UNICODE_ESCAPE,
};
use std::sync::Arc;
use tsox_core::core::compiler_options::ScriptTarget;
use tsox_core::core::text::TextRange;
pub fn get_identifier_token(s: &str) -> SyntaxKind { ::tsox_core::fntrace::enter("get_identifier_token"); 
    let bytes = s.as_bytes();
    if s.len() >= 2
        && s.len() <= 12
        && !bytes.is_empty()
        && bytes[0].is_ascii_lowercase()
    {
        if let Some(keyword) = keywords().get(s) {
            if *keyword != SyntaxKind::Unknown {
                return *keyword;
            }
        }
    }
    SyntaxKind::Identifier
}

pub fn is_valid_identifier(s: &str) -> bool { ::tsox_core::fntrace::enter("is_valid_identifier"); 
    if s.is_empty() {
        return false;
    }
    for (i, ch) in s.chars().enumerate() {
        if i == 0 && !crate::scanner::is_identifier_start(ch)
            || i != 0 && !crate::scanner::is_identifier_part(ch)
        {
            return false;
        }
    }
    true
}

pub fn is_identifier_part_ex(ch: char, language_variant: crate::ast::LanguageVariant) -> bool { ::tsox_core::fntrace::enter("is_identifier_part_ex"); 
    crate::scanner::regexp_reg_exp_flag_modifiers::is_word_character(ch)
        || ch == '$'
        || ch >= '\u{80}' && crate::scanner::is_unicode_identifier_part(ch)
        || language_variant == crate::ast::LanguageVariant::Jsx && ch == '-'
}

pub fn get_viable_keyword_suggestions() -> Vec<&'static str> { ::tsox_core::fntrace::enter("get_viable_keyword_suggestions"); 
    keywords()
        .keys()
        .filter(|text| text.len() > 2)
        .copied()
        .collect()
}

pub fn could_start_trivia(text: &str, pos: usize) -> bool { ::tsox_core::fntrace::enter("could_start_trivia"); 
    let ch = text.as_bytes()[pos];
    match ch {
        b'\r' | b'\n' | b'\t' | 0x0B | 0x0C | b' ' | b'/' | b'<' | b'|' | b'=' | b'>' => true,
        b'#' => pos == 0,
        _ => ch > 0x7F,
    }
}

pub fn get_scanner_for_source_file(source_file: &SourceFile, pos: usize) -> Scanner { ::tsox_core::fntrace::enter("get_scanner_for_source_file"); 
    let mut s = Scanner::new(source_file.text.clone());
    s.pos = pos;
    s.end = s.text.len();
    s.language_variant = source_file.language_variant;
    s.scan();
    s
}

pub fn scan_token_at_position(source_file: &SourceFile, pos: usize) -> SyntaxKind { ::tsox_core::fntrace::enter("scan_token_at_position"); 
    let s = get_scanner_for_source_file(source_file, pos);
    s.token
}

pub fn get_range_of_token_at_position(source_file: &SourceFile, pos: usize) -> TextRange { ::tsox_core::fntrace::enter("get_range_of_token_at_position"); 
    let s = get_scanner_for_source_file(source_file, pos);
    TextRange::new(s.token_pos, s.pos)
}

pub(crate) fn get_error_range_for_arrow_function(
    source_file: &SourceFile,
    node: &Arc<Node>,
) -> TextRange { ::tsox_core::fntrace::enter("get_error_range_for_arrow_function"); 
    let pos = crate::scanner::skip_trivia(&source_file.text, node.pos());
    let body = match &node.data {
        crate::ast::node_data_generated::NodeData::ArrowFunction(d) => Some(&d.body),
        _ => None,
    };
    if let Some(body) = body {
        if body.kind == SyntaxKind::Block {
            let start_line = crate::format::util::line_of_position(source_file, body.pos());
            let end_line = crate::format::util::line_of_position(source_file, body.end());
            if start_line < end_line {
                return TextRange::new(
                    pos,
                    crate::format::util::end_line_position(source_file, start_line) + 1,
                );
            }
        }
    }
    TextRange::new(pos, node.end())
}

pub(crate) fn find_originating_jsdoc_satisfies_tag(
    source_file: &SourceFile,
    node: &Arc<Node>,
) -> Option<Arc<Node>> { ::tsox_core::fntrace::enter("find_originating_jsdoc_satisfies_tag"); 
    let target_type: &Arc<Node> = match &node.data {
        crate::ast::node_data_generated::NodeData::SatisfiesExpression(d) => &d.type_node,
        _ => return None,
    };
    if !target_type.flags.intersects(NodeFlags::Reparsed) {
        return None;
    }
    let mut current = node.parent();
    while let Some(cur) = current {
        if cur.flags.intersects(NodeFlags::HasJSDoc) {
            let mut first_satisfies_tag: Option<Arc<Node>> = None;
            for js_doc in cur.jsdoc(source_file) {
                let crate::ast::node_data_generated::NodeData::JSDoc(js_doc_data) = &js_doc.data
                else {
                    continue;
                };
                let Some(tags) = js_doc_data.tags.clone() else {
                    continue;
                };
                for tag in tags.nodes.iter() {
                    if !crate::ast::is_jsdoc_satisfies_tag(tag) {
                        continue;
                    }
                    if first_satisfies_tag.is_none() {
                        first_satisfies_tag = Some(tag.clone());
                    }
                    if let crate::ast::node_data_generated::NodeData::JSDocSatisfiesTag(d) =
                        &tag.data
                    {
                        if let Some(t) = d.type_expression.type_node() {
                            if t.loc == target_type.loc {
                                return Some(tag.clone());
                            }
                        }
                    }
                }
            }
            return first_satisfies_tag;
        }
        current = cur.parent();
    }
    None
}

pub fn get_error_range_for_node(source_file: &SourceFile, node: &Arc<Node>) -> TextRange { ::tsox_core::fntrace::enter("get_error_range_for_node"); 
    let mut error_node: Option<Arc<Node>> = Some(node.clone());
    match node.kind {
        SyntaxKind::SourceFile => {
            let pos = crate::scanner::skip_trivia(&source_file.text, 0);
            if pos == source_file.text.len() {
                return TextRange::new(0, 0);
            }
            return get_range_of_token_at_position(source_file, pos);
        }
        SyntaxKind::FunctionDeclaration | SyntaxKind::MethodDeclaration
            if node.flags.intersects(NodeFlags::Reparsed) => {}
        SyntaxKind::FunctionDeclaration
        | SyntaxKind::MethodDeclaration
        | SyntaxKind::VariableDeclaration
        | SyntaxKind::BindingElement
        | SyntaxKind::ClassDeclaration
        | SyntaxKind::InterfaceDeclaration
        | SyntaxKind::ModuleDeclaration
        | SyntaxKind::EnumDeclaration
        | SyntaxKind::EnumMember
        | SyntaxKind::FunctionExpression
        | SyntaxKind::GetAccessor
        | SyntaxKind::SetAccessor
        | SyntaxKind::TypeAliasDeclaration
        | SyntaxKind::JSTypeAliasDeclaration
        | SyntaxKind::PropertyDeclaration
        | SyntaxKind::PropertySignature
        | SyntaxKind::NamespaceImport => {
            error_node = crate::ast::get_name_of_declaration(node);
        }
        SyntaxKind::ClassExpression => {
            error_node = node.name().cloned();
        }
        SyntaxKind::ArrowFunction => {
            return get_error_range_for_arrow_function(source_file, node);
        }
        SyntaxKind::CaseClause | SyntaxKind::DefaultClause => {
            let start = crate::scanner::skip_trivia(&source_file.text, node.pos());
            let mut end = node.end();
            if let crate::ast::node_data_generated::NodeData::CaseOrDefaultClause(d) = &node.data {
                let statements = &d.statements;
                if !statements.nodes.is_empty() {
                    end = statements.nodes[0].pos();
                }
            }
            return TextRange::new(start, end);
        }
        SyntaxKind::ReturnStatement | SyntaxKind::YieldExpression => {
            let pos = crate::scanner::skip_trivia(&source_file.text, node.pos());
            return get_range_of_token_at_position(source_file, pos);
        }
        SyntaxKind::SatisfiesExpression => {
            if let Some(js_doc_satisfies_tag) =
                find_originating_jsdoc_satisfies_tag(source_file, node)
            {
                let tag_name = match &js_doc_satisfies_tag.data {
                    crate::ast::node_data_generated::NodeData::JSDocSatisfiesTag(d) => {
                        Some(&d.tag_name)
                    }
                    _ => None,
                };
                if let Some(tag_name) = tag_name {
                    let pos = crate::scanner::skip_trivia(&source_file.text, tag_name.pos());
                    return get_range_of_token_at_position(source_file, pos);
                }
            }
            let expression_end = match &node.data {
                crate::ast::node_data_generated::NodeData::SatisfiesExpression(d) => {
                    d.expression.end()
                }
                _ => node.end(),
            };
            let pos =
                crate::scanner::skip_trivia(&source_file.text, expression_end);
            return get_range_of_token_at_position(source_file, pos);
        }
        SyntaxKind::Constructor if node.flags.intersects(NodeFlags::Reparsed) => {}
        SyntaxKind::Constructor => {
            let mut scanner = get_scanner_for_source_file(source_file, node.pos());
            let start = scanner.token_pos();
            while scanner.token != SyntaxKind::ConstructorKeyword
                && scanner.token != SyntaxKind::StringLiteral
                && scanner.token != SyntaxKind::EndOfFile
            {
                scanner.scan();
            }
            return TextRange::new(start, scanner.pos);
        }
        _ => {}
    }
    let Some(error_node) = error_node else {
        return get_range_of_token_at_position(source_file, node.pos());
    };
    let mut pos = error_node.pos();
    if !crate::astnav::is_missing_node(&error_node) && error_node.kind != SyntaxKind::JsxText {
        pos = crate::scanner::skip_trivia(&source_file.text, pos);
    }
    TextRange::new(pos, error_node.end())
}

fn values_of_non_binary_unicode_properties(
    property_name: &str,
) -> Option<std::collections::HashSet<&'static str>> { ::tsox_core::fntrace::enter("values_of_non_binary_unicode_properties"); 
    use crate::scanner::unicode_properties_general_category_values::GENERAL_CATEGORY_VALUES;
    use crate::scanner::unicode_properties_script_values_script_values::SCRIPT_VALUES;
    match property_name {
        "General_Category" => Some(GENERAL_CATEGORY_VALUES.clone()),
        "Script" | "Script_Extensions" => Some(SCRIPT_VALUES.clone()),
        _ => None,
    }
}

impl<'a> crate::scanner::regexp_reg_exp_flag_modifiers::RegExpParser<'a> {
    pub(super) fn get_spelling_suggestion_for_unicode_property_name(&self, name: &str) -> String { ::tsox_core::fntrace::enter("get_spelling_suggestion_for_unicode_property_name"); 
        use crate::scanner::unicode_properties_non_binary_unicode_properties::NON_BINARY_UNICODE_PROPERTIES;
        tsox_core::core::mig::m3j_2::get_spelling_suggestion_for_strings(
            name,
            NON_BINARY_UNICODE_PROPERTIES.iter().map(|(n, _)| n.to_string()),
        )
        .unwrap_or_default()
    }

    pub(super) fn get_spelling_suggestion_for_unicode_property_value(
        &self,
        property_name: &str,
        value: &str,
    ) -> String { ::tsox_core::fntrace::enter("get_spelling_suggestion_for_unicode_property_value"); 
        let Some(values) = values_of_non_binary_unicode_properties(property_name) else {
            return String::new();
        };
        tsox_core::core::mig::m3j_2::get_spelling_suggestion_for_strings(
            value,
            values.iter().copied().map(|v| v.to_string()),
        )
        .unwrap_or_default()
    }

    pub(super) fn get_spelling_suggestion_for_unicode_property_name_or_value(
        &self,
        name: &str,
    ) -> String { ::tsox_core::fntrace::enter("get_spelling_suggestion_for_unicode_property_name_or_value"); 
        use crate::scanner::unicode_properties_general_category_values::GENERAL_CATEGORY_VALUES;
        use crate::scanner::unicode_properties_non_binary_unicode_properties::{
            BINARY_UNICODE_PROPERTIES, BINARY_UNICODE_PROPERTIES_OF_STRINGS,
        };
        tsox_core::core::mig::m3j_2::get_spelling_suggestion_for_strings(
            name,
            GENERAL_CATEGORY_VALUES
                .iter()
                .copied()
                .chain(BINARY_UNICODE_PROPERTIES.iter().copied())
                .chain(BINARY_UNICODE_PROPERTIES_OF_STRINGS.iter().copied())
                .map(|v| v.to_string()),
        )
        .unwrap_or_default()
    }
}
