#![allow(dead_code, unused_imports, unused_variables)]

use std::sync::Arc;

use tsox_checker::checker::Checker;
use tsox_core::core::text::TextRange;
use tsox_frontend::ast::{self, Node, NodeData, NodeList, SourceFile, Symbol, SyntaxKind};

use super::m5w::{
    M5wArgumentListInfo, M5wArgumentOrParameterListAndIndex, M5wArgumentOrParameterListInfo,
    M5wCallInvocation, M5wContextualInvocation, M5wContextualSignatureLocationInfo,
    M5wInvocation, M5wSignatureHelpItemInfo, M5wSignatureHelpParameter, M5wTypeArgsInvocation,
    SIGNATURE_HELP_NODE_BUILDER_FLAGS,
};
use crate::ls::types_symbols::{ParameterInformation, SignatureInformation};

pub fn get_argument_index(
    node: &Arc<Node>,
    arguments: &Arc<NodeList>,
    source_file: &Arc<SourceFile>,
    c: &mut Checker,
) -> usize {
    let tokens = super::m5w::get_token_from_node_list(
        Some(arguments),
        node.parent().as_ref(),
        source_file,
    );
    get_argument_index_or_count(&tokens, Some(node), c)
}

pub fn get_argument_count(
    node: &Arc<Node>,
    arguments: &Arc<NodeList>,
    source_file: &Arc<SourceFile>,
    c: &mut Checker,
) -> usize {
    let tokens = super::m5w::get_token_from_node_list(
        Some(arguments),
        node.parent().as_ref(),
        source_file,
    );
    get_argument_index_or_count(&tokens, None, c)
}

pub fn get_argument_index_or_count(
    arguments: &[Arc<Node>],
    node: Option<&Arc<Node>>,
    c: &mut Checker,
) -> usize {
    let mut argument_index = 0usize;
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
        if ast::is_spread_element(arg) {
            argument_index += super::m5w::get_spread_element_count(arg, c);
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
    let Some(_) = node else {
        return argument_index;
    };
    let mut argument_count = argument_index;
    if let Some(last) = arguments.last() {
        if last.kind == SyntaxKind::CommaToken {
            argument_count = argument_index + 1;
        }
    }
    argument_count
}

pub fn get_argument_or_parameter_list_info(
    node: &Arc<Node>,
    source_file: &Arc<SourceFile>,
    c: &mut Checker,
) -> Option<M5wArgumentOrParameterListInfo> {
    let info = get_argument_or_parameter_list_and_index(node, source_file, c)?;
    let list = info.list.clone();
    let argument_index = info.argument_index;
    let argument_count = get_argument_count(node, &list, source_file, c);
    let arguments_span = get_applicable_span_for_arguments(Some(&list), Some(node), source_file);
    Some(M5wArgumentOrParameterListInfo {
        list,
        argument_index,
        argument_count,
        arguments_span,
    })
}

pub fn get_applicable_span_for_arguments(
    argument_list: Option<&Arc<NodeList>>,
    node: Option<&Arc<Node>>,
    source_file: &Arc<SourceFile>,
) -> TextRange {
    if argument_list.is_none() {
        if let Some(node) = node {
            let span_start = node.end();
            let span_end = tsox_frontend::scanner::skip_trivia(&source_file.text, node.end());
            let span_end = ensure_minimum_span_size(span_start, span_end);
            return TextRange::new(span_start, span_end);
        }
    }
    let argument_list = argument_list.unwrap();
    let applicable_span_start = argument_list.pos();
    let applicable_span_end =
        tsox_frontend::scanner::skip_trivia(&source_file.text, argument_list.end());
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

pub fn get_argument_or_parameter_list_and_index(
    node: &Arc<Node>,
    source_file: &Arc<SourceFile>,
    c: &mut Checker,
) -> Option<M5wArgumentOrParameterListAndIndex> {
    if node.kind == SyntaxKind::LessThanToken || node.kind == SyntaxKind::OpenParenToken {
        let parent = node.parent()?;
        let list = get_child_list_that_starts_with_opener_token(&parent, node)?;
        return Some(M5wArgumentOrParameterListAndIndex {
            list,
            argument_index: 0,
        });
    }
    let list = find_containing_list(node, source_file)?;
    let argument_index = get_argument_index(node, &list, source_file, c);
    Some(M5wArgumentOrParameterListAndIndex {
        list,
        argument_index,
    })
}

pub fn get_child_list_that_starts_with_opener_token(
    parent: &Arc<Node>,
    opener_token: &Arc<Node>,
) -> Option<Arc<NodeList>> {
    match &parent.data {
        NodeData::CallExpression(d) => {
            if opener_token.kind == SyntaxKind::LessThanToken {
                d.type_arguments.clone()
            } else {
                Some(d.arguments.clone())
            }
        }
        NodeData::NewExpression(d) => {
            if opener_token.kind == SyntaxKind::LessThanToken {
                d.type_arguments.clone()
            } else {
                d.arguments.clone()
            }
        }
        _ => None,
    }
}

pub fn find_containing_list(
    node: &Arc<Node>,
    _file: &Arc<SourceFile>,
) -> Option<Arc<NodeList>> {
    let parent = node.parent()?;
    let list_contains_node = |list: &Option<Arc<NodeList>>| -> Option<Arc<NodeList>> {
        let list = list.as_ref()?;
        if list.pos() <= node.pos() && node.end() <= list.end() {
            Some(Arc::clone(list))
        } else {
            None
        }
    };
    match &parent.data {
        NodeData::CallExpression(d) => list_contains_node(&d.type_arguments)
            .or_else(|| list_contains_node(&Some(d.arguments.clone()))),
        NodeData::NewExpression(d) => list_contains_node(&d.type_arguments)
            .or_else(|| list_contains_node(&d.arguments)),
        _ => None,
    }
}

pub fn get_adjusted_node(node: &Arc<Node>) -> Option<Arc<Node>> {
    match node.kind {
        SyntaxKind::OpenParenToken | SyntaxKind::CommaToken => Some(node.clone()),
        _ => {
            let mut current = node.parent();
            while let Some(cur) = current {
                if ast::is_parameter_declaration(&cur) {
                    return Some(cur);
                }
                current = cur.parent();
            }
            None
        }
    }
}

pub fn try_get_parameter_info(
    starting_token: &Arc<Node>,
    source_file: &Arc<SourceFile>,
    c: &mut Checker,
) -> Option<M5wArgumentListInfo> {
    let node = get_adjusted_node(starting_token)?;
    let info = get_contextual_signature_location_info(&node, source_file, c)?;

    let non_nullable_contextual_type = c.get_non_nullable_type(&info.contextual_type);
    let symbol = non_nullable_contextual_type.symbol()?;
    let signatures = c.get_signatures_of_type(
        &non_nullable_contextual_type,
        tsox_checker::checker::SignatureKind::Call,
    );
    if signatures.is_empty() {
        return None;
    }
    let signature = signatures[signatures.len() - 1].clone();
    Some(M5wArgumentListInfo {
        is_type_parameter_list: false,
        invocation: M5wInvocation {
            contextual_invocation: Some(M5wContextualInvocation {
                signature: Some(signature),
                node: starting_token.clone(),
                symbol: Some(choose_better_symbol(&symbol)),
            }),
            ..Default::default()
        },
        arguments_span: info.arguments_span,
        argument_index: info.argument_index,
        argument_count: info.argument_count,
    })
}

pub fn choose_better_symbol(s: &Arc<Symbol>) -> Arc<Symbol> {
    if s.name == ast::INTERNAL_SYMBOL_NAME_TYPE {
        for d in &s.declarations {
            if ast::is_function_type_node(d) {
                if let (Some(parent), true) = (d.parent(), ast::can_have_symbol(&d.parent().unwrap()))
                {
                    if let Some(parent_symbol) =
                        tsox_checker::checker::mig::m1a::r19k2_defs::symbol_of_node(&parent)
                    {
                        return parent_symbol;
                    }
                }
            }
        }
    }
    s.clone()
}

pub fn get_contextual_signature_location_info(
    node: &Arc<Node>,
    source_file: &Arc<SourceFile>,
    c: &mut Checker,
) -> Option<M5wContextualSignatureLocationInfo> {
    let parent = node.parent()?;
    match parent.kind {
        SyntaxKind::ParenthesizedExpression
        | SyntaxKind::MethodDeclaration
        | SyntaxKind::FunctionExpression
        | SyntaxKind::ArrowFunction => {
            let info = get_argument_or_parameter_list_info(node, source_file, c)?;
            let contextual_type = if ast::is_method_declaration(&parent) {
                c.get_contextual_type_for_object_literal_element(
                    &parent,
                    tsox_checker::checker::ContextFlags::None,
                )
            } else {
                c.get_contextual_type(&parent, tsox_checker::checker::ContextFlags::None)
            };
            contextual_type.map(|contextual_type| M5wContextualSignatureLocationInfo {
                contextual_type,
                argument_index: info.argument_index,
                argument_count: info.argument_count,
                arguments_span: info.arguments_span,
            })
        }
        SyntaxKind::BinaryExpression => {
            let highest_binary = super::m5w::get_highest_binary(&parent);
            let contextual_type =
                c.get_contextual_type(&highest_binary, tsox_checker::checker::ContextFlags::None);
            if node.kind == SyntaxKind::OpenParenToken {
                return None;
            }
            let argument_index = count_binary_expression_parameters(&parent) - 1;
            let argument_count = count_binary_expression_parameters(&highest_binary);
            contextual_type.map(|contextual_type| M5wContextualSignatureLocationInfo {
                contextual_type,
                argument_index,
                argument_count,
                arguments_span: TextRange::new(parent.pos(), parent.end()),
            })
        }
        _ => None,
    }
}

pub fn count_binary_expression_parameters(b: &Arc<Node>) -> usize {
    if let NodeData::BinaryExpression(d) = &b.data {
        if let NodeData::BinaryExpression(_) = &d.left.data {
            return count_binary_expression_parameters(&d.left) + 1;
        }
    }
    2
}

pub fn is_inside_template_literal(
    node: &Arc<Node>,
    position: usize,
    source_file: &Arc<SourceFile>,
) -> bool {
    tsox_frontend::ast::mig::m3g_2::is_template_literal_kind(node.kind)
        && (tsox_frontend::scanner::mig::x5a::get_token_pos_of_node(node, source_file, false)
            < position
            && position < node.end()
            || (is_unterminated_literal(node) && position == node.end()))
}

pub fn is_unterminated_literal(node: &Arc<Node>) -> bool {
    let text = ast::node_text(node);
    match text.chars().last() {
        Some(last) => {
            (last == '{' && node.kind == SyntaxKind::TemplateHead)
                || (last == '$' && node.kind == SyntaxKind::TemplateMiddle)
                || (node.kind == SyntaxKind::TemplateTail
                    && text.chars().filter(|c| *c == '`').count() == 0)
                || (node.kind == SyntaxKind::NoSubstitutionTemplateLiteral
                    && text.starts_with('`')
                    && (text.len() < 2 || !text.ends_with('`')))
        }
        None => true,
    }
}

pub fn get_immediately_containing_argument_info(
    node: &Arc<Node>,
    position: usize,
    source_file: &Arc<SourceFile>,
    c: &mut Checker,
) -> Option<M5wArgumentListInfo> {
    let parent = node.parent()?;
    if tsox_frontend::ast::mig::m3f_4::is_call_or_new_expression(&parent) {
        let info = get_argument_or_parameter_list_info(node, source_file, c)?;
        let list = info.list.clone();
        let argument_index = info.argument_index;
        let argument_count = info.argument_count;
        let arguments_span = info.arguments_span;
        let mut is_type_parameter_list = false;
        if let Some(parent_type_argument_list) =
            super::m5w::call_or_new_expression_type_argument_list(&parent)
        {
            if parent_type_argument_list.pos() == list.pos() {
                is_type_parameter_list = true;
            }
        }
        return Some(M5wArgumentListInfo {
            is_type_parameter_list,
            invocation: M5wInvocation {
                call_invocation: Some(M5wCallInvocation { node: parent }),
                ..Default::default()
            },
            arguments_span,
            argument_index,
            argument_count,
        });
    }
    if ast::is_no_substitution_template_literal(node) && ast::is_tagged_template_expression(&parent) {
        if is_inside_template_literal(node, position, source_file) {
            return get_argument_list_info_for_template(&parent, 0, source_file);
        }
        return None;
    }
    if ast::is_template_head(node)
        && parent.parent().map(|p| p.kind) == Some(SyntaxKind::TaggedTemplateExpression)
    {
        let tag_expression = parent
            .parent()
            .and_then(|p| p.parent())
            .filter(|p| p.kind == SyntaxKind::TaggedTemplateExpression)?;
        let argument_index = if is_inside_template_literal(node, position, source_file) {
            0
        } else {
            1
        };
        return get_argument_list_info_for_template(&tag_expression, argument_index, source_file);
    }
    if ast::is_template_span(&parent)
        && parent
            .parent()
            .and_then(|p| p.parent())
            .map(|p| ast::is_tagged_template_expression(&p))
            == Some(true)
    {
        let tag_expression = parent.parent().and_then(|p| p.parent())?;
        if ast::is_template_tail(node) && !is_inside_template_literal(node, position, source_file) {
            return None;
        }
        let template_spans = parent
            .parent()
            .and_then(|p| template_expression_template_spans(&p))?;
        let span_index =
            tsox_frontend::ast::mig::m3f_2::index_of_node(&template_spans.nodes, &parent)
                .unwrap_or(0);
        let argument_index =
            get_argument_index_for_template_piece(span_index, node, position, source_file);
        return get_argument_list_info_for_template(&tag_expression, argument_index, source_file);
    }
    if tsox_frontend::ast::mig::m3g::is_jsx_opening_like_element(&parent) {
        let attributes = jsx_opening_element_attributes(&parent)?;
        let attribute_span_start = attributes.pos();
        let attribute_span_end =
            tsox_frontend::scanner::skip_trivia(&source_file.text, attributes.end());
        return Some(M5wArgumentListInfo {
            is_type_parameter_list: false,
            invocation: M5wInvocation {
                call_invocation: Some(M5wCallInvocation { node: parent }),
                ..Default::default()
            },
            arguments_span: TextRange::new(
                attribute_span_start,
                attribute_span_end - attribute_span_start,
            ),
            argument_index: 0,
            argument_count: 1,
        });
    }
    if let Some(type_arg_info) =
        super::m5x_5::get_possible_type_arguments_info(node, source_file)
    {
        let called = type_arg_info.called;
        let n_type_arguments = type_arg_info.n_type_arguments;
        let argument_range = TextRange::new(called.pos(), node.end());
        return Some(M5wArgumentListInfo {
            is_type_parameter_list: true,
            invocation: M5wInvocation {
                type_args_invocation: Some(M5wTypeArgsInvocation {
                    called: node_as_identifier(&called),
                }),
                ..Default::default()
            },
            arguments_span: argument_range,
            argument_index: n_type_arguments,
            argument_count: n_type_arguments + 1,
        });
    }
    None
}

pub fn node_as_identifier(node: &Arc<Node>) -> Arc<Node> {
    Arc::clone(node)
}

pub fn jsx_opening_element_attributes(node: &Arc<Node>) -> Option<Arc<Node>> {
    match &node.data {
        NodeData::JsxOpeningElement(d) => Some(d.attributes.clone()),
        NodeData::JsxSelfClosingElement(d) => Some(d.attributes.clone()),
        _ => None,
    }
}

pub fn template_expression_template_spans(node: &Arc<Node>) -> Option<Arc<NodeList>> {
    match &node.data {
        NodeData::TemplateExpression(d) => Some(d.template_spans.clone()),
        _ => None,
    }
}

pub fn template_span_literal(node: &Arc<Node>) -> Option<Arc<Node>> {
    match &node.data {
        NodeData::TemplateSpan(d) => Some(d.literal.clone()),
        _ => None,
    }
}

pub fn tagged_template_expression_template(node: &Arc<Node>) -> Option<Arc<Node>> {
    match &node.data {
        NodeData::TaggedTemplateExpression(d) => Some(d.template.clone()),
        _ => None,
    }
}

pub fn get_argument_index_for_template_piece(
    span_index: usize,
    node: &Arc<Node>,
    position: usize,
    source_file: &Arc<SourceFile>,
) -> usize {
    if tsox_frontend::ast::mig::m3g_2::is_template_literal_token(node) {
        if is_inside_template_literal(node, position, source_file) {
            return 0;
        }
        return span_index + 2;
    }
    span_index + 1
}

pub fn get_argument_list_info_for_template(
    tag_expression: &Arc<Node>,
    argument_index: usize,
    source_file: &Arc<SourceFile>,
) -> Option<M5wArgumentListInfo> {
    let template = tagged_template_expression_template(tag_expression)?;
    let argument_count = if ast::is_no_substitution_template_literal(&template) {
        1
    } else {
        template_expression_template_spans(&template)?.nodes.len() + 1
    };
    Some(M5wArgumentListInfo {
        is_type_parameter_list: false,
        invocation: M5wInvocation {
            call_invocation: Some(M5wCallInvocation {
                node: tag_expression.clone(),
            }),
            ..Default::default()
        },
        argument_index,
        argument_count,
        arguments_span: get_applicable_range_for_tagged_template(tag_expression, source_file),
    })
}

pub fn get_applicable_range_for_tagged_template(
    tagged_template: &Arc<Node>,
    source_file: &Arc<SourceFile>,
) -> TextRange {
    let template = tagged_template_expression_template(tagged_template).unwrap();
    let applicable_span_start =
        tsox_frontend::scanner::mig::x5a::get_token_pos_of_node(&template, source_file, false);
    let mut applicable_span_end = template.end();
    if template.kind == SyntaxKind::TemplateExpression {
        if let Some(template_spans) = template_expression_template_spans(&template) {
            if let Some(last_span) = template_spans.nodes.last() {
                if let Some(literal) = template_span_literal(last_span) {
                    if literal.end() - literal.pos() == 0 {
                        applicable_span_end =
                            tsox_frontend::scanner::skip_trivia(&source_file.text, applicable_span_end);
                    }
                }
            }
        }
    }
    TextRange::new(applicable_span_start, applicable_span_end - applicable_span_start)
}
