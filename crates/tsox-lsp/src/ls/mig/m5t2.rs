#![allow(dead_code, unused_imports, unused_variables)]

use std::sync::Arc;

use crate::ls::language_service::LanguageService;
use crate::ls::types::FoldingRange;
use crate::ls::types_capabilities::ClientCapabilities;
use crate::lsp::lsproto_lsp_basic::{Position, Range};
use tsox_frontend::ast::mig::m3f_4::is_call_or_new_expression;
use tsox_frontend::ast::{self, Node, SourceFile, SyntaxKind};
use tsox_frontend::astnav;
use tsox_frontend::scanner;

use super::m5s::SpanFidelity;
use crate::ls::folding::parse_region_delimiter;

pub struct FoldingRangeKey {
    pub start_line: u32,
    pub start_character: Option<u32>,
    pub end_line: u32,
    pub end_character: Option<u32>,
    pub kind: Option<String>,
    pub collapsed_text: Option<String>,
}

pub fn key_for_folding_range(folding_range: &FoldingRange) -> FoldingRangeKey {
    FoldingRangeKey {
        start_line: folding_range.start_line,
        start_character: folding_range.start_character,
        end_line: folding_range.end_line,
        end_character: folding_range.end_character,
        kind: folding_range.kind.clone(),
        collapsed_text: folding_range.collapsed_text.clone(),
    }
}

impl LanguageService {
    pub fn adjust_folding_end(&self, ranges: Vec<FoldingRange>, source_file: &Arc<SourceFile>) -> Vec<FoldingRange> {
        let source_text = &source_file.text;
        let script = FoldingSourceFileScript(Arc::clone(source_file));
        let mut result = Vec::with_capacity(ranges.len());
        for mut r in ranges {
            if let Some(end_character) = r.end_character {
                if end_character > 0 {
                    let end_offset = self.converters.line_and_character_to_position(
                        &script,
                        &Position {
                            line: r.end_line,
                            character: end_character,
                        },
                    );
                    if end_offset > 0 && end_offset <= source_text.len() {
                        let fold_end_char = source_text.as_bytes()[end_offset - 1];
                        if fold_end_char == b'}' || fold_end_char == b']' || fold_end_char == b')' || fold_end_char == b'`' || fold_end_char == b'>' {
                            if r.end_line > r.start_line {
                                r.end_line -= 1;
                            }
                        }
                    }
                }
            }
            result.push(r);
        }
        result
    }
}

struct FoldingSourceFileScript(Arc<SourceFile>);

impl crate::ls::lsconv_converters::Script for FoldingSourceFileScript {
    fn file_name(&self) -> &str {
        &self.0.file_name
    }

    fn text(&self) -> &str {
        &self.0.text
    }
}

pub fn supports_collapsed_text(capabilities: &ClientCapabilities) -> bool {
    capabilities.text_document.folding_range.collapsed_text
}

pub fn create_folding_range(
    capabilities: &ClientCapabilities,
    text_range: &Range,
    folding_range_kind: &str,
    collapsed_text: &str,
) -> FoldingRange {
    let kind = if folding_range_kind.is_empty() {
        None
    } else {
        Some(folding_range_kind.to_string())
    };
    let mut result = FoldingRange {
        start_line: text_range.start.line,
        start_character: Some(text_range.start.character),
        end_line: text_range.end.line,
        end_character: Some(text_range.end.character),
        kind,
        collapsed_text: None,
    };
    if !collapsed_text.is_empty() && supports_collapsed_text(capabilities) {
        result.collapsed_text = Some(collapsed_text.to_string());
    }
    result
}

pub fn create_folding_range_from_bounds(
    l: &LanguageService,
    pos: usize,
    end: usize,
    folding_range_kind: &str,
    source_file: &Arc<SourceFile>,
) -> Option<FoldingRange> {
    let (text_range, fidelity) = l.m5x_create_lsp_range_from_bounds(pos, end, source_file);
    if fidelity == SpanFidelity::None_ as u32 {
        return None;
    }
    Some(create_folding_range(&ClientCapabilities::default(), &text_range, folding_range_kind, ""))
}

pub fn visit_node(
    n: &Arc<Node>,
    depth_remaining: u32,
    source_file: &Arc<SourceFile>,
    l: &LanguageService,
) -> Vec<FoldingRange> {
    if n.flags.contains(ast::NodeFlags::Reparsed) || depth_remaining == 0 {
        return Vec::new();
    }
    let mut folding_range: Vec<FoldingRange> = Vec::with_capacity(40);
    if (!ast::is_binary_expression(n) && ast::is_declaration(n))
        || ast::is_variable_statement(n)
        || ast::is_return_statement(n)
        || is_call_or_new_expression(n)
        || n.kind == SyntaxKind::EndOfFile
    {
        folding_range.extend(add_outlining_for_leading_comments_for_node(n, source_file, l));
    }
    if ast::is_function_like(n) {
        if let Some(parent) = n.parent() {
            if ast::is_binary_expression(&parent) && ast::is_property_access_expression(&parent.left()) {
                let left = parent.left();
                folding_range.extend(add_outlining_for_leading_comments_for_node(left, source_file, l));
            }
        }
    }
    if ast::is_block(n) {
        if let ast::NodeData::Block(d) = &n.data {
            folding_range.extend(add_outlining_for_leading_comments_for_pos(d.statements.pos(), source_file, l));
        }
    }
    if ast::is_module_block(n) {
        if let ast::NodeData::ModuleBlock(d) = &n.data {
            folding_range.extend(add_outlining_for_leading_comments_for_pos(d.statements.pos(), source_file, l));
        }
    }
    if ast::is_class_like(n) || ast::is_interface_declaration(n) {
        if let Some(members) = node_members_list(n) {
            folding_range.extend(add_outlining_for_leading_comments_for_pos(members.pos(), source_file, l));
        }
    }

    if let Some(span) = get_outlining_span_for_node(n, source_file, l) {
        folding_range.push(span);
    }

    let mut depth_remaining = depth_remaining - 1;
    if ast::is_call_expression(n) {
        depth_remaining += 1;
        if let Some(expression) = n.expression() {
            folding_range.extend(visit_node(expression, depth_remaining, source_file, l));
        }
        depth_remaining -= 1;
        if let Some(arguments) = n.arguments() {
            for arg in &arguments.nodes {
                folding_range.extend(visit_node(arg, depth_remaining, source_file, l));
            }
        }
        if let Some(type_arguments) = n.type_arguments() {
            for type_arg in &type_arguments.nodes {
                folding_range.extend(visit_node(type_arg, depth_remaining, source_file, l));
            }
        }
    } else if ast::is_if_statement(n)
        && matches!(&n.data, ast::NodeData::IfStatement(d) if d
            .else_statement
            .as_ref()
            .is_some_and(|e| ast::is_if_statement(e)))
    {
        let ast::NodeData::IfStatement(d) = &n.data else {
            return folding_range;
        };
        folding_range.extend(visit_node(&d.expression, depth_remaining, source_file, l));
        folding_range.extend(visit_node(&d.then_statement, depth_remaining, source_file, l));
        depth_remaining += 1;
        if let Some(else_node) = &d.else_statement {
            folding_range.extend(visit_node(else_node, depth_remaining, source_file, l));
        }
        depth_remaining -= 1;
    } else {
        tsox_frontend::ast::node_data_generated::for_each_child(n, |child| {
            folding_range.extend(visit_node(child, depth_remaining, source_file, l));
            false
        });
    }
    folding_range
}

fn node_members_list(n: &Node) -> Option<&Arc<ast::NodeList>> {
    match &n.data {
        ast::NodeData::ClassDeclaration(d) => Some(&d.members),
        ast::NodeData::ClassExpression(d) => Some(&d.members),
        ast::NodeData::InterfaceDeclaration(d) => Some(&d.members),
        _ => None,
    }
}

pub fn add_outlining_for_leading_comments_for_node(
    n: &Arc<Node>,
    source_file: &Arc<SourceFile>,
    l: &LanguageService,
) -> Vec<FoldingRange> {
    if ast::is_jsx_text(n) {
        return Vec::new();
    }
    add_outlining_for_leading_comments_for_pos(n.pos(), source_file, l)
}

pub fn add_outlining_for_leading_comments_for_pos(
    pos: usize,
    source_file: &Arc<SourceFile>,
    l: &LanguageService,
) -> Vec<FoldingRange> {
    let mut folding_range: Vec<FoldingRange> = Vec::with_capacity(40);
    let mut first_single_line_comment_start: i64 = -1;
    let mut last_single_line_comment_end: i64 = -1;
    let mut single_line_comment_count = 0;
    let folding_range_kind_comment = "comment";

    let source_text = &source_file.text;
    for comment in scanner::get_leading_comment_ranges(source_text, pos) {
        let comment_pos = comment.pos;
        let comment_end = comment.end;
        match comment.kind {
            scanner::CommentRangeKind::SingleLine => {
                let comment_text = &source_text[comment_pos..comment_end];
                if parse_region_delimiter(comment_text).is_some() {
                    if let Some(comments) = combine_and_add_multiple_single_line_comments(
                        single_line_comment_count,
                        first_single_line_comment_start,
                        last_single_line_comment_end,
                        folding_range_kind_comment,
                        source_file,
                        l,
                    ) {
                        folding_range.push(comments);
                    }
                    single_line_comment_count = 0;
                } else {
                    if single_line_comment_count == 0 {
                        first_single_line_comment_start = comment_pos as i64;
                    }
                    last_single_line_comment_end = comment_end as i64;
                    single_line_comment_count += 1;
                }
            }
            scanner::CommentRangeKind::MultiLine => {
                if let Some(comments) = combine_and_add_multiple_single_line_comments(
                    single_line_comment_count,
                    first_single_line_comment_start,
                    last_single_line_comment_end,
                    folding_range_kind_comment,
                    source_file,
                    l,
                ) {
                    folding_range.push(comments);
                }
                if let Some(comment) = create_folding_range_from_bounds(
                    l,
                    comment_pos,
                    comment_end,
                    folding_range_kind_comment,
                    source_file,
                ) {
                    folding_range.push(comment);
                }
                single_line_comment_count = 0;
            }
        }
    }
    if let Some(added_comments) = combine_and_add_multiple_single_line_comments(
        single_line_comment_count,
        first_single_line_comment_start,
        last_single_line_comment_end,
        folding_range_kind_comment,
        source_file,
        l,
    ) {
        folding_range.push(added_comments);
    }
    folding_range
}

fn combine_and_add_multiple_single_line_comments(
    single_line_comment_count: u32,
    first_single_line_comment_start: i64,
    last_single_line_comment_end: i64,
    kind: &str,
    source_file: &Arc<SourceFile>,
    l: &LanguageService,
) -> Option<FoldingRange> {
    if single_line_comment_count > 1 {
        return create_folding_range_from_bounds(
            l,
            first_single_line_comment_start as usize,
            last_single_line_comment_end as usize,
            kind,
            source_file,
        );
    }
    None
}

pub fn get_outlining_span_for_node(
    n: &Arc<Node>,
    source_file: &Arc<SourceFile>,
    l: &LanguageService,
) -> Option<FoldingRange> {
    match n.kind {
        SyntaxKind::Block => {
            let parent = n.parent()?;
            if ast::is_function_like(&parent) {
                return function_span(&parent, n, source_file, l);
            }
            match parent.kind {
                SyntaxKind::DoStatement
                | SyntaxKind::ForInStatement
                | SyntaxKind::ForOfStatement
                | SyntaxKind::ForStatement
                | SyntaxKind::IfStatement
                | SyntaxKind::WhileStatement
                | SyntaxKind::WithStatement
                | SyntaxKind::CatchClause => span_for_node(n, SyntaxKind::OpenBraceToken, true, source_file, l),
                SyntaxKind::TryStatement => {
                    let ast::NodeData::TryStatement(try_statement) = &parent.data else {
                        return standalone_block_folding_range(n, source_file, l);
                    };
                    if Arc::ptr_eq(&try_statement.try_block, n) {
                        return span_for_node(n, SyntaxKind::OpenBraceToken, true, source_file, l);
                    }
                    if try_statement
                        .finally_block
                        .as_ref()
                        .is_some_and(|fb| Arc::ptr_eq(fb, n))
                    {
                        if let Some(span) = span_for_node(n, SyntaxKind::OpenBraceToken, true, source_file, l) {
                            return Some(span);
                        }
                    }
                    standalone_block_folding_range(n, source_file, l)
                }
                _ => standalone_block_folding_range(n, source_file, l),
            }
        }
        SyntaxKind::ModuleBlock => span_for_node(n, SyntaxKind::OpenBraceToken, true, source_file, l),
        SyntaxKind::ClassDeclaration
        | SyntaxKind::ClassExpression
        | SyntaxKind::InterfaceDeclaration
        | SyntaxKind::EnumDeclaration
        | SyntaxKind::CaseBlock
        | SyntaxKind::TypeLiteral
        | SyntaxKind::ObjectBindingPattern => span_for_node(n, SyntaxKind::OpenBraceToken, true, source_file, l),
        SyntaxKind::TupleType => {
            let parent_is_tuple = n.parent().as_deref().map(ast::is_tuple_type_node).unwrap_or(false);
            span_for_node(n, SyntaxKind::OpenBracketToken, !parent_is_tuple, source_file, l)
        }
        SyntaxKind::CaseClause | SyntaxKind::DefaultClause => {
            let ast::NodeData::CaseOrDefaultClause(d) = &n.data else {
                return None;
            };
            span_for_node_array(&d.statements, source_file, l)
        }
        SyntaxKind::ObjectLiteralExpression | SyntaxKind::ArrayLiteralExpression => {
            let parent = n.parent()?;
            let use_full_start =
                !ast::is_array_literal_expression(&parent) && !is_call_or_new_expression(&parent);
            let open = if n.kind == SyntaxKind::ObjectLiteralExpression {
                SyntaxKind::OpenBraceToken
            } else {
                SyntaxKind::OpenBracketToken
            };
            span_for_node(n, open, use_full_start, source_file, l)
        }
        SyntaxKind::JsxElement | SyntaxKind::JsxFragment => span_for_jsx_element(n, source_file, l),
        SyntaxKind::JsxSelfClosingElement | SyntaxKind::JsxOpeningElement => {
            span_for_jsx_attributes(n, source_file, l)
        }
        SyntaxKind::TemplateExpression | SyntaxKind::NoSubstitutionTemplateLiteral => {
            span_for_template_literal(n, source_file, l)
        }
        SyntaxKind::ArrayBindingPattern => {
            let parent_is_binding_element = n
                .parent()
                .as_deref()
                .map(ast::is_binding_element)
                .unwrap_or(false);
            span_for_node(n, SyntaxKind::OpenBracketToken, !parent_is_binding_element, source_file, l)
        }
        SyntaxKind::ArrowFunction => span_for_arrow_function(n, source_file, l),
        SyntaxKind::CallExpression => span_for_call_expression(n, source_file, l),
        SyntaxKind::ParenthesizedExpression => span_for_parenthesized_expression(n, source_file, l),
        SyntaxKind::NamedImports | SyntaxKind::NamedExports | SyntaxKind::ImportAttributes => {
            span_for_import_export_elements(n, source_file, l)
        }
        _ => None,
    }
}

fn standalone_block_folding_range(
    n: &Arc<Node>,
    source_file: &Arc<SourceFile>,
    l: &LanguageService,
) -> Option<FoldingRange> {
    let (text_range, fidelity) =
        l.create_lsp_range_from_node_for_feature(n, source_file, super::m5s::SpanFeature::Definition as u32);
    if fidelity == SpanFidelity::None_ as u32 {
        return None;
    }
    Some(create_folding_range(&ClientCapabilities::default(), &text_range, "", ""))
}

pub fn span_for_import_export_elements(
    node: &Arc<Node>,
    source_file: &Arc<SourceFile>,
    l: &LanguageService,
) -> Option<FoldingRange> {
    let elements = match &node.data {
        ast::NodeData::NamedImports(d) => &d.elements,
        ast::NodeData::NamedExports(d) => &d.elements,
        ast::NodeData::ImportAttributes(d) => &d.attributes,
        _ => return None,
    };
    if elements.nodes.is_empty() {
        return None;
    }
    let open_token = astnav::find_child_of_kind(node, SyntaxKind::OpenBraceToken)?;
    let close_token = astnav::find_child_of_kind(node, SyntaxKind::CloseBraceToken)?;
    if positions_are_on_same_line(open_token.pos(), close_token.pos(), source_file) {
        return None;
    }
    range_between_tokens(&open_token, &close_token, source_file, false, l)
}

pub fn span_for_parenthesized_expression(
    node: &Arc<Node>,
    source_file: &Arc<SourceFile>,
    l: &LanguageService,
) -> Option<FoldingRange> {
    let start = astnav::get_start_of_node(node, source_file, false);
    if positions_are_on_same_line(start, node.end(), source_file) {
        return None;
    }
    create_folding_range_from_bounds(l, start, node.end(), "", source_file)
}

pub fn span_for_call_expression(
    node: &Arc<Node>,
    source_file: &Arc<SourceFile>,
    l: &LanguageService,
) -> Option<FoldingRange> {
    if node.arguments().map_or(true, |arguments| arguments.nodes.is_empty()) {
        return None;
    }
    let open_token = astnav::find_child_of_kind(node, SyntaxKind::OpenParenToken)?;
    let close_token = astnav::find_child_of_kind(node, SyntaxKind::CloseParenToken)?;
    if positions_are_on_same_line(open_token.pos(), close_token.pos(), source_file) {
        return None;
    }
    range_between_tokens(&open_token, &close_token, source_file, true, l)
}

pub fn span_for_arrow_function(
    node: &Arc<Node>,
    source_file: &Arc<SourceFile>,
    l: &LanguageService,
) -> Option<FoldingRange> {
    let body = node.body()?;
    if ast::is_block(&body) || ast::is_parenthesized_expression(&body) || positions_are_on_same_line(body.pos(), body.end(), source_file) {
        return None;
    }
    create_folding_range_from_bounds(l, body.pos(), body.end(), "", source_file)
}

pub fn span_for_template_literal(
    node: &Arc<Node>,
    source_file: &Arc<SourceFile>,
    l: &LanguageService,
) -> Option<FoldingRange> {
    if node.kind == SyntaxKind::NoSubstitutionTemplateLiteral && node.text().is_empty() {
        return None;
    }
    create_folding_range_from_bounds(
        l,
        astnav::get_start_of_node(node, source_file, false),
        node.end(),
        "",
        source_file,
    )
}

pub fn span_for_jsx_element(
    node: &Arc<Node>,
    source_file: &Arc<SourceFile>,
    l: &LanguageService,
) -> Option<FoldingRange> {
    if node.kind == SyntaxKind::JsxElement {
        let ast::NodeData::JsxElement(d) = &node.data else {
            return None;
        };
        let (text_range, fidelity) = l.m5x_create_lsp_range_from_bounds(
            astnav::get_start_of_node(&d.opening_element, source_file, false),
            d.closing_element.end(),
            source_file,
        );
        if fidelity == SpanFidelity::None_ as u32 {
            return None;
        }
        let tag_name_node = match &d.opening_element.data {
            ast::NodeData::JsxOpeningElement(od) => &od.tag_name,
            ast::NodeData::JsxSelfClosingElement(od) => &od.tag_name,
            _ => return None,
        };
        let tag_name = scanner::mig::m3i::get_text_of_node(tag_name_node);
        let banner_text = format!("<{}>...</{}>", tag_name, tag_name);
        return Some(create_folding_range(&ClientCapabilities::default(), &text_range, "", &banner_text));
    }
    let ast::NodeData::JsxFragment(d) = &node.data else {
        return None;
    };
    create_folding_range_from_bounds(
        l,
        astnav::get_start_of_node(&d.opening_fragment, source_file, false),
        d.closing_fragment.end(),
        "<>...</>",
        source_file,
    )
}

pub fn span_for_jsx_attributes(
    node: &Arc<Node>,
    source_file: &Arc<SourceFile>,
    l: &LanguageService,
) -> Option<FoldingRange> {
    let attributes = match &node.data {
        ast::NodeData::JsxSelfClosingElement(d) => &d.attributes,
        ast::NodeData::JsxOpeningElement(d) => &d.attributes,
        _ => return None,
    };
    let ast::NodeData::JsxAttributes(ad) = &attributes.data else {
        return None;
    };
    if ad.properties.nodes.is_empty() {
        return None;
    }
    create_folding_range_from_bounds(
        l,
        astnav::get_start_of_node(node, source_file, false),
        node.end(),
        "",
        source_file,
    )
}

pub fn span_for_node_array(
    statements: &ast::NodeList,
    source_file: &Arc<SourceFile>,
    l: &LanguageService,
) -> Option<FoldingRange> {
    if !statements.nodes.is_empty() {
        return create_folding_range_from_bounds(l, statements.pos(), statements.end(), "", source_file);
    }
    None
}

pub fn span_for_node(
    node: &Arc<Node>,
    open: SyntaxKind,
    use_full_start: bool,
    source_file: &Arc<SourceFile>,
    l: &LanguageService,
) -> Option<FoldingRange> {
    let close_brace = if open != SyntaxKind::OpenBraceToken {
        SyntaxKind::CloseBracketToken
    } else {
        SyntaxKind::CloseBraceToken
    };
    let open_token = astnav::find_child_of_kind(node, open)?;
    let close_token = astnav::find_child_of_kind(node, close_brace)?;
    range_between_tokens(&open_token, &close_token, source_file, use_full_start, l)
}

pub fn range_between_tokens(
    open_token: &Arc<Node>,
    close_token: &Arc<Node>,
    source_file: &Arc<SourceFile>,
    use_full_start: bool,
    l: &LanguageService,
) -> Option<FoldingRange> {
    let bounds = if use_full_start {
        (open_token.pos(), close_token.end())
    } else {
        (
            astnav::get_start_of_node(open_token, source_file, false),
            close_token.end(),
        )
    };
    create_folding_range_from_bounds(l, bounds.0, bounds.1, "", source_file)
}

pub fn function_span(
    node: &Arc<Node>,
    body: &Arc<Node>,
    source_file: &Arc<SourceFile>,
    l: &LanguageService,
) -> Option<FoldingRange> {
    let open_token = try_get_function_open_token(node, body, source_file)?;
    let close_token = astnav::find_child_of_kind(body, SyntaxKind::CloseBraceToken)?;
    range_between_tokens(&open_token, &close_token, source_file, true, l)
}

pub fn try_get_function_open_token(
    node: &Arc<Node>,
    body: &Arc<Node>,
    source_file: &Arc<SourceFile>,
) -> Option<Arc<Node>> {
    if let Some(parameters) = node.parameters() {
        if is_node_array_multi_line(parameters, source_file) {
            if let Some(open_paren_token) = astnav::find_child_of_kind(node, SyntaxKind::OpenParenToken) {
                return Some(open_paren_token);
            }
        }
    }
    astnav::find_child_of_kind(body, SyntaxKind::OpenBraceToken)
}

pub fn is_node_array_multi_line(list: &ast::NodeList, source_file: &Arc<SourceFile>) -> bool {
    if list.nodes.is_empty() {
        return false;
    }
    !positions_are_on_same_line(list.nodes[0].pos(), list.nodes[list.nodes.len() - 1].end(), source_file)
}

pub fn positions_are_on_same_line(pos1: usize, pos2: usize, source_file: &Arc<SourceFile>) -> bool {
    tsox_frontend::format::mig::m4t_3::positions_are_on_same_line(pos1 as i64, pos2 as i64, source_file)
}
