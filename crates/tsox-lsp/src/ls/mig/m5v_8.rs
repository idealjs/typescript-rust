#![allow(dead_code, unused_imports, unused_variables)]

use std::sync::Arc;

use tsox_checker::checker::Checker;
use tsox_checker::checker::types::ElementFlags;
use tsox_core::core::text::TextRange;
use tsox_frontend::ast::NodeData;
use tsox_frontend::ast::{self, Node, SourceFile, Symbol, SyntaxKind};
use tsox_frontend::astnav;
use tsox_frontend::scanner;

use crate::ls::language_service::LanguageService;
use crate::lsp::lsproto_lsp::Range;

fn signature_help_node_builder_flags() -> tsox_checker::checker::symboltracker::NodeBuilderFlags {
    use tsox_checker::checker::symboltracker::NodeBuilderFlags as Flags;
    Flags::OmitParameterModifiers | Flags::UseAliasDefinedOutsideCurrentScope
}

pub struct M5vSignatureHelpParameter {
    pub label: String,
    pub documentation: Option<String>,
    pub is_rest: bool,
    pub is_optional: bool,
}

pub struct M5vSignatureInformation {
    pub label: String,
    pub documentation: Option<String>,
    pub parameters: Vec<M5vSignatureHelpParameter>,
    pub is_variadic: bool,
    pub colorized_runs: Vec<()>,
}

pub struct M5vArgumentListInfo {
    pub is_type_parameter_list: bool,
    pub invocation_node: Arc<Node>,
    pub argument_index: usize,
    pub argument_count: usize,
    pub arguments_span: TextRange,
}

pub fn contains_preceding_token(
    starting_token: &Arc<Node>,
    source_file: &Arc<SourceFile>,
    container: &Arc<Node>,
) -> bool {
    let pos = starting_token.pos();
    let preceding_token = astnav::find_preceding_token(&source_file.node, pos);
    if let Some(preceding_token) = preceding_token {
        return crate::ls::mig::m5x_3::range_contains_range(
            container.loc,
            preceding_token.loc,
        );
    }
    false
}

pub fn get_argument_index_for_template_piece(
    span_index: usize,
    node: &Arc<Node>,
    position: usize,
    source_file: &Arc<SourceFile>,
) -> usize {
    if ast::mig::m3g_2::is_template_literal_token(node) {
        if is_inside_template_literal(node, position, source_file) {
            return 0;
        }
        return span_index + 2;
    }
    span_index + 1
}

fn m5v_find_ancestor(start: &Arc<Node>, callback: impl Fn(&Node) -> bool) -> Option<Arc<Node>> {
    let mut current = Some(Arc::clone(start));
    while let Some(n) = current {
        if callback(&n) {
            return Some(n);
        }
        current = n.parent();
    }
    None
}

pub fn get_adjusted_node(node: &Arc<Node>) -> Option<Arc<Node>> {
    match node.kind {
        SyntaxKind::OpenParenToken | SyntaxKind::CommaToken => Some(Arc::clone(node)),
        _ => match node.parent() {
            Some(parent) => m5v_find_ancestor(&parent, |n: &Node| n.kind == SyntaxKind::Parameter),
            None => None,
        },
    }
}

pub fn get_argument_index(
    node: &Arc<Node>,
    arguments: &ast::NodeList,
    source_file: &Arc<SourceFile>,
    c: &mut Checker,
) -> usize {
    get_argument_index_or_count(
        &get_token_from_node_list(arguments, node.parent().as_ref(), source_file),
        Some(node),
        c,
    )
}

pub fn get_argument_count(
    node: &Arc<Node>,
    arguments: &ast::NodeList,
    source_file: &Arc<SourceFile>,
    c: &mut Checker,
) -> usize {
    get_argument_index_or_count(
        &get_token_from_node_list(arguments, node.parent().as_ref(), source_file),
        None,
        c,
    )
}

pub fn get_argument_index_or_count(
    arguments: &[Arc<Node>],
    node: Option<&Arc<Node>>,
    c: &mut Checker,
) -> usize {
    let mut argument_index: usize = 0;
    let mut skip_comma = false;
    for arg in arguments {
        if let Some(node) = node {
            if Arc::ptr_eq(arg, node) {
                if !skip_comma && arg.kind == SyntaxKind::CommaToken {
                    argument_index += 1;
                }
                return argument_index;
            }
        }
        if let NodeData::SpreadElement(spread_data) = &arg.data {
            argument_index += get_spread_element_count(spread_data, c);
            skip_comma = true;
            continue;
        }
        if arg.kind != SyntaxKind::CommaToken {
            argument_index += 1;
            skip_comma = true;
            continue;
        }
        if skip_comma {
            skip_comma = false;
            continue;
        }
        argument_index += 1;
    }
    if node.is_some() {
        return argument_index;
    }
    let mut argument_count = argument_index;
    if let Some(last) = arguments.last() {
        if last.kind == SyntaxKind::CommaToken {
            argument_count = argument_index + 1;
        }
    }
    argument_count
}

pub fn get_spread_element_count(node: &ast::SpreadElementData, c: &mut Checker) -> usize {
    let spread_type = c.get_type_at_location(&node.expression);
    if !c.is_tuple_type(&spread_type) {
        return 0;
    }
    let Some(tuple_type) = spread_type.target().and_then(|t| t.as_tuple_type()) else {
        return 0;
    };
    let element_flags = tuple_type.element_flags();
    let fixed_length = tuple_type.fixed_length;
    if fixed_length == 0 {
        return 0;
    }
    let first_optional_index = element_flags
        .iter()
        .position(|f| !f.contains(ElementFlags::Required));
    match first_optional_index {
        None => fixed_length,
        Some(idx) => idx,
    }
}

fn get_token_from_node_list(
    node_list: &ast::NodeList,
    node_list_parent: Option<&Arc<Node>>,
    source_file: &Arc<SourceFile>,
) -> Vec<Arc<Node>> {
    let Some(node_list_parent) = node_list_parent else {
        return Vec::new();
    };
    let mut left = node_list.pos();
    let mut node_list_index = 0;
    let mut tokens: Vec<Arc<Node>> = Vec::new();
    while left < node_list.end() {
        if node_list.nodes.len() > node_list_index && left == node_list.nodes[node_list_index].pos()
        {
            tokens.push(Arc::clone(&node_list.nodes[node_list_index]));
            left = node_list.nodes[node_list_index].end();
            node_list_index += 1;
        } else {
            let scanner =
                tsox_frontend::scanner::mig::m4d_2::get_scanner_for_source_file(source_file, left);
            let token = scanner.token();
            let token_full_start = scanner.full_start_pos();
            let token_end = scanner.token_end();
            tokens.push(tsox_frontend::ast::mig::m3b_2::get_or_create_token(
                source_file,
                token,
                token_full_start,
                token_end,
                node_list_parent,
                scanner.token_flags(),
            ));
            left = token_end;
        }
    }
    tokens
}

pub fn get_applicable_span_for_arguments(
    argument_list: Option<&ast::NodeList>,
    node: Option<&Arc<Node>>,
    source_file: &Arc<SourceFile>,
) -> TextRange {
    if argument_list.is_none() && node.is_some() {
        let node = node.unwrap();
        let span_start = node.end();
        let span_end = scanner::skip_trivia(&source_file.text, node.end());
        let span_end = ensure_minimum_span_size(span_start, span_end);
        return TextRange::new(span_start, span_end);
    }
    let argument_list = argument_list.unwrap();
    let applicable_span_start = argument_list.pos();
    let applicable_span_end = scanner::skip_trivia(&source_file.text, argument_list.end());
    let applicable_span_end =
        ensure_minimum_span_size(applicable_span_start, applicable_span_end);
    TextRange::new(applicable_span_start, applicable_span_end)
}

pub fn ensure_minimum_span_size(start: usize, end: usize) -> usize {
    if end <= start {
        return start + 1;
    }
    end
}

pub fn choose_better_symbol(s: &Arc<Symbol>) -> Arc<Symbol> {
    if s.name == ast::INTERNAL_SYMBOL_NAME_TYPE {
        for d in &s.declarations {
            if ast::is_function_type_node(d) && ast::can_have_symbol(d.parent().as_ref().unwrap())
            {
                if let Some(parent) = d.parent() {
                    if let Some(sym) =
                        tsox_checker::checker::mig::m1a::r19k2_defs::symbol_of_node(&parent)
                    {
                        return sym;
                    }
                }
            }
        }
    }
    Arc::clone(s)
}

fn binary_expression_data_of(node: &Arc<Node>) -> Option<&ast::BinaryExpressionData> {
    match &node.data {
        NodeData::BinaryExpression(data) => Some(data),
        _ => None,
    }
}

pub fn count_binary_expression_parameters(b: &ast::BinaryExpressionData) -> usize {
    if let Some(left) = binary_expression_data_of(&b.left) {
        return count_binary_expression_parameters(left) + 1;
    }
    2
}

pub fn get_argument_list_info_for_template(
    tag_expression: &Arc<Node>,
    argument_index: usize,
    source_file: &Arc<SourceFile>,
) -> M5vArgumentListInfo {
    let NodeData::TaggedTemplateExpression(tag_data) = &tag_expression.data else {
        panic!("get_argument_list_info_for_template: node is not a tagged template expression");
    };
    let mut argument_count = 1;
    if !ast::is_no_substitution_template_literal(&tag_data.template) {
        let NodeData::TemplateExpression(template_data) = &tag_data.template.data else {
            panic!("tagged template literal is not a template expression");
        };
        argument_count = template_data.template_spans.nodes.len() + 1;
    }
    M5vArgumentListInfo {
        is_type_parameter_list: false,
        invocation_node: Arc::clone(tag_expression),
        argument_index,
        argument_count,
        arguments_span: get_applicable_range_for_tagged_template(tag_expression, source_file),
    }
}

pub fn get_applicable_range_for_tagged_template(
    tagged_template: &Arc<Node>,
    source_file: &Arc<SourceFile>,
) -> TextRange {
    let NodeData::TaggedTemplateExpression(tag_data) = &tagged_template.data else {
        panic!("get_applicable_range_for_tagged_template: node is not a tagged template expression");
    };
    let template = &tag_data.template;
    let applicable_span_start = tsox_frontend::scanner::mig::x5a::get_token_pos_of_node(
        template,
        source_file,
        false,
    );
    let mut applicable_span_end = template.end();

    if template.kind == SyntaxKind::TemplateExpression {
        let NodeData::TemplateExpression(template_data) = &template.data else {
            panic!("tagged template literal is not a template expression");
        };
        let template_spans = template_data.template_spans.clone();
        let last_span = &template_spans.nodes[template_spans.nodes.len() - 1];
        let NodeData::TemplateSpan(span_data) = &last_span.data else {
            panic!("template span node is not a template span");
        };
        let literal = &span_data.literal;
        if literal.end() - literal.pos() == 0 {
            applicable_span_end = scanner::skip_trivia(&source_file.text, applicable_span_end);
        }
    }

    TextRange::new(applicable_span_start, applicable_span_end)
}

fn m5v_markup_kind_str(doc_format: &crate::lsp::lsproto_lsp_basic::MarkupKind) -> &'static str {
    match doc_format {
        crate::lsp::lsproto_lsp_basic::MarkupKind::Markdown => "markdown",
        _ => "plaintext",
    }
}

impl LanguageService {
    pub fn m5v_create_signature_help_parameter_from_label(
        &self,
        parameter: &Arc<Symbol>,
        label: &str,
        c: &mut Checker,
        doc_format: crate::lsp::lsproto_lsp_basic::MarkupKind,
    ) -> M5vSignatureHelpParameter {
        let is_optional = parameter
            .check_flags
            .contains(ast::CheckFlags::OptionalParameter);
        let is_rest = parameter.check_flags.contains(ast::CheckFlags::RestParameter);
        let mut documentation: Option<String> = None;
        if let Some(value_declaration) = parameter.value_declaration.as_ref() {
            let mapper =
                self.documentation_location_mapper(crate::ls::mig::m5s::SpanFeature::Definition);
            let doc = crate::ls::mig::m5t2_2::get_documentation_from_declaration(
                &mapper,
                c,
                None,
                Some(value_declaration),
                value_declaration,
                m5v_markup_kind_str(&doc_format),
                true,
            );
            if !doc.is_empty() {
                documentation = Some(doc);
            }
        }
        M5vSignatureHelpParameter {
            label: label.to_string(),
            documentation,
            is_rest,
            is_optional,
        }
    }

    pub fn m5v_create_signature_help_parameter_for_parameter(
        &self,
        parameter: &Arc<Symbol>,
        enclosing_declaration: &Arc<Node>,
        p: &mut tsox_frontend::format::mig::m4o_2::Printer,
        source_file: &Arc<SourceFile>,
        c: &mut Checker,
        doc_format: crate::lsp::lsproto_lsp_basic::MarkupKind,
    ) -> M5vSignatureHelpParameter {
        let param_node = {
            let mut node_builder = tsox_checker::checker::mig::m2f::new_node_builder_ex(
                c,
                tsox_checker::checker::mig::m2f::new_emit_context(),
                std::collections::HashMap::new(),
            );
            node_builder.symbol_to_parameter_declaration(
                parameter,
                Some(enclosing_declaration),
                signature_help_node_builder_flags(),
                tsox_checker::checker::symboltracker::NodeBuilderInternalFlags::default(),
                None,
            )
        };
        let display = match param_node {
            Some(param_node) => p.emit(&param_node, Some(source_file)),
            None => parameter.name.clone(),
        };
        self.m5v_create_signature_help_parameter_from_label(parameter, &display, c, doc_format)
    }
}

pub fn create_signature_help_parameter_for_type_parameter(
    t: &Arc<tsox_checker::checker::types::Type>,
    source_file: &Arc<SourceFile>,
    enclosing_declaration: &Arc<Node>,
    c: &mut Checker,
    p: &mut tsox_frontend::format::mig::m4o_2::Printer,
) -> Option<M5vSignatureHelpParameter> {
    let type_parameter_node = {
        let mut node_builder = tsox_checker::checker::mig::m2f::new_node_builder_ex(
            c,
            tsox_checker::checker::mig::m2f::new_emit_context(),
            std::collections::HashMap::new(),
        );
        node_builder.type_parameter_to_declaration(
            t,
            Some(enclosing_declaration),
            signature_help_node_builder_flags(),
            tsox_checker::checker::symboltracker::NodeBuilderInternalFlags::default(),
            None,
        )
    };
    let type_parameter_node = type_parameter_node?;
    let display = p.emit(&type_parameter_node, Some(source_file));
    Some(M5vSignatureHelpParameter {
        label: display,
        documentation: None,
        is_rest: false,
        is_optional: false,
    })
}

fn is_inside_template_literal(
    node: &Arc<Node>,
    position: usize,
    source_file: &Arc<SourceFile>,
) -> bool {
    crate::ls::mig::m5x_4::is_inside_template_literal(node, position, source_file)
}
