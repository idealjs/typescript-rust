#![allow(unused_imports)]
#![allow(dead_code)]
use std::sync::Arc;

use tsox_frontend::ast::node::Node;
use tsox_frontend::ast::syntax_kind_generated::SyntaxKind;
use tsox_frontend::ast::{
    get_source_file_of_node, is_binding_pattern, is_identifier, is_keyword_kind,
    is_numeric_literal, is_string_literal_like,
};

use crate::mig::m4m_2::get_ecma_line_of_position;
use crate::printer::{EmitContext, NodeFactory};
#[path = "r38k9_defs.rs"]
pub mod r38k9_defs;
use self::r38k9_defs::{R38K9NodeCastExt, r38k9_same_node};

fn node_opt_is(opt: Option<&Arc<Node>>, name: &Arc<Node>) -> bool {
    opt.is_some_and(|e| Arc::ptr_eq(e, name))
}

pub fn is_identifier_reference(name: &Arc<Node>, parent: &Arc<Node>) -> bool {
    match parent.kind {
        SyntaxKind::BinaryExpression
        | SyntaxKind::PrefixUnaryExpression
        | SyntaxKind::PostfixUnaryExpression
        | SyntaxKind::YieldExpression
        | SyntaxKind::AsExpression
        | SyntaxKind::SatisfiesExpression
        | SyntaxKind::ElementAccessExpression
        | SyntaxKind::NonNullExpression
        | SyntaxKind::SpreadElement
        | SyntaxKind::SpreadAssignment
        | SyntaxKind::ParenthesizedExpression
        | SyntaxKind::ArrayLiteralExpression
        | SyntaxKind::DeleteExpression
        | SyntaxKind::TypeOfExpression
        | SyntaxKind::VoidExpression
        | SyntaxKind::AwaitExpression
        | SyntaxKind::TypeAssertionExpression
        | SyntaxKind::ExpressionWithTypeArguments
        | SyntaxKind::JsxSelfClosingElement
        | SyntaxKind::JsxSpreadAttribute
        | SyntaxKind::JsxExpression
        | SyntaxKind::PartiallyEmittedExpression => true,
        SyntaxKind::ComputedPropertyName
        | SyntaxKind::Decorator
        | SyntaxKind::IfStatement
        | SyntaxKind::DoStatement
        | SyntaxKind::WhileStatement
        | SyntaxKind::WithStatement
        | SyntaxKind::ReturnStatement
        | SyntaxKind::SwitchStatement
        | SyntaxKind::CaseClause
        | SyntaxKind::ThrowStatement
        | SyntaxKind::ExpressionStatement
        | SyntaxKind::ExportAssignment
        | SyntaxKind::PropertyAccessExpression
        | SyntaxKind::TemplateSpan => node_opt_is(parent.expression(), name),
        SyntaxKind::VariableDeclaration
        | SyntaxKind::Parameter
        | SyntaxKind::BindingElement
        | SyntaxKind::PropertyDeclaration
        | SyntaxKind::PropertySignature
        | SyntaxKind::PropertyAssignment
        | SyntaxKind::EnumMember
        | SyntaxKind::JsxAttribute => node_opt_is(parent.initializer(), name),
        SyntaxKind::ShorthandPropertyAssignment => {
            node_opt_is(parent.as_shorthand_property_assignment().object_assignment_initializer.as_ref(), name)
        }
        SyntaxKind::ForStatement => {
            node_opt_is(parent.initializer(), name)
                || node_opt_is(parent.as_for_statement().condition.as_ref(), name)
                || node_opt_is(parent.as_for_statement().incrementor.as_ref(), name)
        }
        SyntaxKind::ForInStatement | SyntaxKind::ForOfStatement => {
            node_opt_is(parent.initializer(), name) || node_opt_is(parent.expression(), name)
        }
        SyntaxKind::ImportEqualsDeclaration => {
            Arc::ptr_eq(&parent.as_import_equals_declaration().module_reference, name)
        }
        SyntaxKind::ArrowFunction => node_opt_is(parent.body(), name),
        SyntaxKind::ConditionalExpression => {
            let c = parent.as_conditional_expression();
            Arc::ptr_eq(&c.condition, name) || Arc::ptr_eq(&c.when_true, name) || Arc::ptr_eq(&c.when_false, name)
        }
        SyntaxKind::CallExpression | SyntaxKind::NewExpression => {
            node_opt_is(parent.expression(), name)
                || parent.call_arguments().iter().any(|a| Arc::ptr_eq(a, name))
        }
        SyntaxKind::TaggedTemplateExpression => {
            Arc::ptr_eq(&parent.as_tagged_template_expression().tag, name)
        }
        SyntaxKind::ImportAttribute => Arc::ptr_eq(&parent.as_import_attribute().value, name),
        SyntaxKind::JsxOpeningElement | SyntaxKind::JsxClosingElement => {
            Arc::ptr_eq(&parent.tag_name(), name)
        }
        _ => false,
    }
}

pub fn convert_binding_element_to_array_assignment_element(
    emit_context: &EmitContext,
    element: &Arc<Node>,
) -> Arc<Node> {
    let binding = element.as_binding_element();
    if binding.name.is_none() {
        let elision = emit_context.factory().new_omitted_expression();
        emit_context.set_original(&elision, element);
        emit_context.assign_comment_and_source_map_ranges(&elision, element);
        return elision;
    }
    if binding.dot_dot_dot_token.is_some() {
        let spread = emit_context
            .factory()
            .r38k9_new_spread_element(binding.name.as_ref().unwrap());
        emit_context.set_original(&spread, element);
        emit_context.assign_comment_and_source_map_ranges(&spread, element);
        return spread;
    }
    let expression =
        convert_binding_name_to_assignment_element_target(emit_context, &binding.name.clone().unwrap());
    if binding.initializer.is_some() {
        let assignment = emit_context.factory().new_assignment_expression(
            &expression,
            binding.initializer.as_ref().unwrap(),
        );
        emit_context.set_original(&assignment, element);
        emit_context.assign_comment_and_source_map_ranges(&assignment, element);
        return assignment;
    }
    expression
}

pub fn convert_binding_element_to_object_assignment_element(
    emit_context: &EmitContext,
    element: &Arc<Node>,
) -> Arc<Node> {
    let binding = element.as_binding_element();
    if binding.dot_dot_dot_token.is_some() {
        let spread = emit_context
            .factory()
            .r38k9_new_spread_assignment(binding.name.as_ref().unwrap());
        emit_context.set_original(&spread, element);
        emit_context.assign_comment_and_source_map_ranges(&spread, element);
        return spread;
    }
    if binding.property_name.is_some() {
        let mut expression =
            convert_binding_name_to_assignment_element_target(emit_context, &binding.name.clone().unwrap());
        if binding.initializer.is_some() {
            expression = emit_context.factory().new_assignment_expression(
                &expression,
                binding.initializer.as_ref().unwrap(),
            );
        }
        let assignment = emit_context.factory().new_property_assignment(
            None,
            binding.property_name.as_ref().unwrap(),
            None,
            None,
            &expression,
        );
        emit_context.set_original(&assignment, element);
        emit_context.assign_comment_and_source_map_ranges(&assignment, element);
        return assignment;
    }
    let equals_token = if binding.initializer.is_some() {
        Some(emit_context.factory().new_token(SyntaxKind::EqualsToken))
    } else {
        None
    };
    let assignment = emit_context.factory().new_shorthand_property_assignment(
        None,
        binding.name.as_ref().unwrap(),
        None,
        None,
        equals_token.as_ref(),
        binding.initializer.clone(),
    );
    emit_context.set_original(&assignment, element);
    emit_context.assign_comment_and_source_map_ranges(&assignment, element);
    assignment
}

pub fn convert_binding_pattern_to_assignment_pattern(
    emit_context: &EmitContext,
    element: &Arc<Node>,
) -> Arc<Node> {
    match element.kind {
        SyntaxKind::ArrayBindingPattern => {
            convert_binding_element_to_array_assignment_pattern(emit_context, element)
        }
        SyntaxKind::ObjectBindingPattern => {
            convert_binding_element_to_object_assignment_pattern(emit_context, element)
        }
        _ => panic!("Unknown binding pattern"),
    }
}

pub fn convert_binding_element_to_object_assignment_pattern(
    emit_context: &EmitContext,
    element: &Arc<Node>,
) -> Arc<Node> {
    let mut properties: Vec<Arc<Node>> = vec![];
    let elements = element.as_binding_pattern().elements.clone();
    for child in elements.nodes.iter() {
        properties.push(convert_binding_element_to_object_assignment_element(
            emit_context,
            child,
        ));
    }
    let mut property_list = emit_context.factory().new_node_list(properties);
    if let Some(list) = Arc::get_mut(&mut property_list) {
        list.loc = elements.loc;
    }
    let object = emit_context
        .factory()
        .new_object_literal_expression(&property_list, false);
    emit_context.set_original(&object, element);
    emit_context.assign_comment_and_source_map_ranges(&object, element);
    object
}

pub fn convert_binding_element_to_array_assignment_pattern(
    emit_context: &EmitContext,
    element: &Arc<Node>,
) -> Arc<Node> {
    let mut elements_out: Vec<Arc<Node>> = vec![];
    let elements = element.as_binding_pattern().elements.clone();
    for child in elements.nodes.iter() {
        elements_out.push(convert_binding_element_to_array_assignment_element(
            emit_context, child,
        ));
    }
    let mut element_list = emit_context.factory().new_node_list(elements_out);
    if let Some(list) = Arc::get_mut(&mut element_list) {
        list.loc = elements.loc;
    }
    let object = emit_context
        .factory()
        .new_array_literal_expression(&element_list, false);
    emit_context.set_original(&object, element);
    emit_context.assign_comment_and_source_map_ranges(&object, element);
    object
}

pub fn convert_binding_name_to_assignment_element_target(
    emit_context: &EmitContext,
    element: &Arc<Node>,
) -> Arc<Node> {
    if is_binding_pattern(element) {
        return convert_binding_pattern_to_assignment_pattern(emit_context, element);
    }
    element.clone()
}

pub fn convert_variable_declaration_to_assignment_expression(
    emit_context: &EmitContext,
    element: &Arc<Node>,
) -> Option<Arc<Node>> {
    let declaration = element.as_variable_declaration();
    declaration.initializer.clone()?;
    let expression = convert_binding_name_to_assignment_element_target(
        emit_context,
        &declaration.name,
    );
    let assignment = emit_context.factory().new_assignment_expression(
        &expression,
        declaration.initializer.as_ref().unwrap(),
    );
    emit_context.set_original(&assignment, element);
    emit_context.assign_comment_and_source_map_ranges(&assignment, element);
    Some(assignment)
}

pub fn single_or_many(nodes: Option<&[Arc<Node>]>, factory: &NodeFactory) -> Option<Arc<Node>> {
    let nodes = nodes?;
    if nodes.len() == 1 {
        return Some(nodes[0].clone());
    }
    Some(factory.new_syntax_list(nodes.to_vec()))
}

pub fn is_simple_copiable_expression(expression: &Arc<Node>) -> bool {
    is_string_literal_like(expression)
        || is_numeric_literal(expression)
        || is_keyword_kind(expression.kind)
        || is_identifier(expression)
}

pub fn is_original_node_single_line(emit_context: &EmitContext, node: Option<&Arc<Node>>) -> bool {
    let node = match node {
        Some(n) => n,
        None => return false,
    };
    let original = emit_context.most_original(node);
    let source = match get_source_file_of_node(&original) {
        Some(s) => s,
        None => return false,
    };
    let source = match crate::mig::m4m_2::registered_source_file_of_node(&source) {
        Some(s) => s,
        None => return false,
    };
    let start_line = get_ecma_line_of_position(&source, original.loc.pos());
    let end_line = get_ecma_line_of_position(&source, original.loc.end());
    start_line == end_line
}

pub fn is_simple_inlineable_expression(expression: &Arc<Node>) -> bool {
    !is_identifier(expression) && is_simple_copiable_expression(expression)
}
