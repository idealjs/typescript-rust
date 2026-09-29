#![allow(invalid_reference_casting)]
use std::sync::Arc;
use tsox_frontend::ast::*;

use crate::mig::m4h::r39k13_defs;
use crate::mig::m4l_3::r39k10_defs::R39K10NodeFactoryExt;
use crate::mig::x6a::is_class_this_assignment_block;
use crate::printer::EmitContext;
use tsox_frontend::ast::mig::m3b::{member_list, members};
use tsox_frontend::ast::mig::m3g::is_named_evaluation_source;
use tsox_frontend::ast::mig::m3g_3::{skip_outer_expressions, OuterExpressionKinds};
use tsox_frontend::ast::utilities::get_heritage_clauses as heritage_clauses;

pub type AnonymousFunctionDefinition = Arc<Node>;

pub fn is_class_named_evaluation_helper_block(emit_context: &EmitContext, node: &Arc<Node>) -> bool {
    let body_statements = match &node.data {
        NodeData::ClassStaticBlockDeclaration(d) => block_statements(&d.body),
        _ => return false,
    };
    if body_statements.len() != 1 {
        return false;
    }
    let statement = &body_statements[0];
    if is_expression_statement(statement) {
        let expression = statement.expression().unwrap().clone();
        if emit_context.is_call_to_helper(&expression, "__setFunctionName") {
            let arguments = match &expression.data {
                NodeData::CallExpression(d) => d.arguments.nodes.clone(),
                _ => return false,
            };
            return arguments.len() >= 2
            && emit_context
                .assigned_name(node)
                .is_some_and(|assigned| Arc::ptr_eq(&arguments[1], &assigned));
        }
    }
    false
}

pub fn class_has_explicitly_assigned_name(emit_context: &EmitContext, node: &Arc<Node>) -> bool {
    if emit_context.assigned_name(node).is_some() {
        for member in members(node) {
            if is_class_named_evaluation_helper_block(emit_context, member) {
                return true;
            }
        }
    }
    false
}

pub fn class_has_declared_or_explicitly_assigned_name(emit_context: &EmitContext, node: &Arc<Node>) -> bool {
    node.name().is_some() || class_has_explicitly_assigned_name(emit_context, node)
}

pub fn is_anonymous_function_definition(
    emit_context: &EmitContext,
    node: &Arc<Node>,
    cb: Option<&dyn Fn(&AnonymousFunctionDefinition) -> bool>,
) -> bool {
    let node = skip_outer_expressions(node, OuterExpressionKinds::ALL);
    match node.kind {
        SyntaxKind::ClassExpression => {
            if class_has_declared_or_explicitly_assigned_name(emit_context, &node) {
                return false;
            }
        }
        SyntaxKind::FunctionExpression => {
            if node.name().is_some() {
                return false;
            }
        }
        SyntaxKind::ArrowFunction => {}
        _ => return false,
    }
    if let Some(cb) = cb {
        return cb(&node);
    }
    true
}

pub fn is_named_evaluation(emit_context: &EmitContext, node: &Arc<Node>) -> bool {
    is_named_evaluation_and(emit_context, node, None)
}

pub fn is_named_evaluation_and(
    emit_context: &EmitContext,
    node: &Arc<Node>,
    cb: Option<&dyn Fn(&AnonymousFunctionDefinition) -> bool>,
) -> bool {
    if !is_named_evaluation_source(node) {
        return false;
    }
    match &node.data {
        NodeData::ShorthandPropertyAssignment(d) => {
            let initializer = d.object_assignment_initializer.clone();
            let initializer = initializer.as_ref().unwrap();
            is_anonymous_function_definition(emit_context, initializer, cb)
        }
        NodeData::PropertyAssignment(d) => is_anonymous_function_definition(emit_context, &d.initializer, cb),
        NodeData::VariableDeclaration(d) => is_anonymous_function_definition(emit_context, d.initializer.as_ref().unwrap(), cb),
        NodeData::ParameterDeclaration(d) => is_anonymous_function_definition(emit_context, d.initializer.as_ref().unwrap(), cb),
        NodeData::BindingElement(d) => is_anonymous_function_definition(emit_context, d.initializer.as_ref().unwrap(), cb),
        NodeData::PropertyDeclaration(d) => is_anonymous_function_definition(emit_context, d.initializer.as_ref().unwrap(), cb),
        NodeData::BinaryExpression(d) => is_anonymous_function_definition(emit_context, &d.right, cb),
        NodeData::ExportAssignment(d) => is_anonymous_function_definition(emit_context, &d.expression, cb),
        _ => panic!("Unhandled case in is_named_evaluation"),
    }
}

pub fn get_assigned_name_of_identifier(
    emit_context: &EmitContext,
    name: &Arc<Node>,
    expression: &Arc<Node>,
) -> Arc<Node> {
    let factory = &emit_context.factory();
    let original = emit_context.most_original(&skip_outer_expressions(expression, OuterExpressionKinds::ALL));
    if (is_class_declaration(&original) || is_function_declaration(&original))
        && original.name().is_none()
        && has_syntactic_modifier(&original, ModifierFlags::Default)
    {
        return factory.new_string_literal("default", 0);
    }
    factory.new_string_literal_from_node(name)
}

pub fn get_assigned_name_of_property_name(
    emit_context: &EmitContext,
    name: &Arc<Node>,
    assigned_name_text: &str,
) -> (Arc<Node>, Arc<Node>) {
    let factory = &emit_context.factory();
    if !assigned_name_text.is_empty() {
        let assigned_name = factory.new_string_literal(assigned_name_text, 0);
        return (assigned_name, name.clone());
    }

    if is_property_name_literal(name) || is_private_identifier(name) {
        let assigned_name = factory.new_string_literal_from_node(name);
        return (assigned_name, name.clone());
    }

    let expression = name.expression().unwrap().clone();
    if is_property_name_literal(&expression) && !is_identifier(&expression) {
        let assigned_name = factory.new_string_literal_from_node(&expression);
        return (assigned_name, name.clone());
    }

    debug_assert!(is_computed_property_name(name), "Expected computed property name");

    let assigned_name = factory.generated_name_node(&factory.new_generated_name_for_node(name));
    emit_context_mut(emit_context).add_variable_declaration(&assigned_name);

    let key = factory.new_prop_key_helper(&expression);
    let assignment = factory.new_assignment_expression(&assigned_name, &key);
    let updated_name = factory.update_computed_property_name(name, &assignment);
    (assigned_name, updated_name)
}

pub fn create_class_named_evaluation_helper_block(
    emit_context: &EmitContext,
    assigned_name: &Arc<Node>,
    this_expression: Option<&Arc<Node>>,
) -> Arc<Node> {
    let factory = &emit_context.factory();
    let this_expression: Arc<Node> = match this_expression {
        Some(expr) => expr.clone(),
        None => factory.new_this_expression(),
    };

    let expression = r39k05_set_function_name_helper(&factory, &this_expression, assigned_name);
    let statement = factory.new_expression_statement(&expression);
    let body = factory.new_block(&factory.new_node_list(vec![statement]), false);
    let block = factory.new_class_static_block_declaration(None, body.clone());

    emit_context_mut(emit_context).set_assigned_name(&block, assigned_name);
    block
}

pub fn inject_class_named_evaluation_helper_block_if_missing(
    emit_context: &EmitContext,
    node: &Arc<Node>,
    assigned_name: &Arc<Node>,
    this_expression: Option<&Arc<Node>>,
) -> Arc<Node> {
    if class_has_explicitly_assigned_name(emit_context, node) {
        return node.clone();
    }

    let factory = &emit_context.factory();
    let named_evaluation_block =
        create_class_named_evaluation_helper_block(emit_context, assigned_name, this_expression);
    if let Some(name) = node.name() {
        emit_context_mut(emit_context).set_source_map_range(
            &block_statements(&block_body(&named_evaluation_block))[0],
            name.loc,
        );
    }

    let node_members = members(node).to_vec();
    let insertion_index = node_members
        .iter()
        .position(|n| is_class_this_assignment_block(emit_context, n))
        .map(|i| i + 1)
        .unwrap_or(0);
    let mut new_members: Vec<Arc<Node>> = node_members[..insertion_index].to_vec();
    new_members.push(named_evaluation_block);
    new_members.extend_from_slice(&node_members[insertion_index..]);
    let mut members_list = factory.new_node_list(new_members);
    if let Some(l) = member_list(node) {
        if let Some(ml) = Arc::get_mut(&mut members_list) {
            ml.loc = l.loc;
        }
    }

    let old_node = node.clone();
    let node = if is_class_declaration(node) {
        factory.update_class_declaration(
            node,
            node.modifiers().cloned(),
            node.name(),
            r39k13_defs::type_parameter_list(node),
            heritage_clauses(node).map(|l| &**l),
            &members_list,
        )
    } else {
        factory.update_class_expression(
            node,
            node.modifiers().cloned(),
            node.name(),
            r39k13_defs::type_parameter_list(node),
            heritage_clauses(node).map(|l| &**l),
            &members_list,
        )
    };

    emit_context_mut(emit_context).set_assigned_name(&node, assigned_name);

    if let Some(ct) = emit_context.class_this(&old_node) {
        emit_context_mut(emit_context).set_class_this(&node, &ct);
    }

    node
}

pub fn finish_transform_named_evaluation(
    emit_context: &EmitContext,
    expression: &Arc<Node>,
    assigned_name: &Arc<Node>,
    ignore_empty_string_literal: bool,
) -> Arc<Node> {
    if ignore_empty_string_literal
        && is_string_literal(assigned_name)
        && string_literal_text(assigned_name).is_empty()
    {
        return expression.clone();
    }

    let factory = &emit_context.factory();
    let inner_expression = skip_outer_expressions(expression, OuterExpressionKinds::ALL);

    let updated_expression = if is_class_expression(&inner_expression) {
        inject_class_named_evaluation_helper_block_if_missing(
            emit_context,
            &inner_expression,
            assigned_name,
            None,
        )
    } else {
        r39k05_set_function_name_helper(&factory, &inner_expression, assigned_name)
    };

    factory.restore_outer_expressions(expression, &updated_expression, OuterExpressionKinds::ALL)
}

pub fn transform_named_evaluation(
    emit_context: &EmitContext,
    node: &Arc<Node>,
    ignore_empty_string_literal: bool,
    assigned_name_text: &str,
) -> Arc<Node> {
    let factory = &emit_context.factory();
    match &node.data {
        NodeData::PropertyAssignment(d) => {
            let (assigned_name, name) =
                get_assigned_name_of_property_name(emit_context, &d.name, assigned_name_text);
            let initializer =
                finish_transform_named_evaluation(emit_context, &d.initializer, &assigned_name, ignore_empty_string_literal);
            factory.update_property_assignment_r39k13(node, None, &name, None, None, Some(&initializer))
        }
        NodeData::ShorthandPropertyAssignment(d) => {
            let assigned_name: Arc<Node> = if !assigned_name_text.is_empty() {
                factory.new_string_literal(assigned_name_text, 0)
            } else {
                let object_assignment_initializer = d
                    .object_assignment_initializer
                    .clone()
                    .unwrap_or_else(|| panic!("ShorthandPropertyAssignment missing object assignment initializer"));
                get_assigned_name_of_identifier(emit_context, &d.name, &object_assignment_initializer)
            };
            let object_assignment_initializer = d
                .object_assignment_initializer
                .clone()
                .unwrap_or_else(|| panic!("ShorthandPropertyAssignment missing object assignment initializer"));
            let initializer = finish_transform_named_evaluation(
                emit_context,
                &object_assignment_initializer,
                &assigned_name,
                ignore_empty_string_literal,
            );
            factory.update_shorthand_property_assignment_r39k13(
                node,
                None,
                &d.name,
                d.equals_token.as_ref(),
                Some(&initializer),
            )
        }
        NodeData::VariableDeclaration(d) => {
            let assigned_name: Arc<Node> = if !assigned_name_text.is_empty() {
                factory.new_string_literal(assigned_name_text, 0)
            } else {
                get_assigned_name_of_identifier(emit_context, &d.name, d.initializer.as_ref().unwrap())
            };
            let initializer =
                finish_transform_named_evaluation(emit_context, d.initializer.as_ref().unwrap(), &assigned_name, ignore_empty_string_literal);
            factory.update_variable_declaration_r39k13(node, &d.name, None, None, Some(&initializer))
        }
        NodeData::ParameterDeclaration(d) => {
            let assigned_name: Arc<Node> = if !assigned_name_text.is_empty() {
                factory.new_string_literal(assigned_name_text, 0)
            } else {
                get_assigned_name_of_identifier(emit_context, &d.name, d.initializer.as_ref().unwrap())
            };
            let initializer =
                finish_transform_named_evaluation(emit_context, d.initializer.as_ref().unwrap(), &assigned_name, ignore_empty_string_literal);
            factory.update_parameter_declaration(
                node,
                None,
                d.dot_dot_dot_token.as_ref(),
                &d.name,
                None,
                None,
                Some(&initializer),
            )
        }
        NodeData::BindingElement(d) => {
            let assigned_name: Arc<Node> = if !assigned_name_text.is_empty() {
                factory.new_string_literal(assigned_name_text, 0)
            } else {
                get_assigned_name_of_identifier(emit_context, d.name.as_ref().unwrap(), d.initializer.as_ref().unwrap())
            };
            let initializer =
                finish_transform_named_evaluation(emit_context, d.initializer.as_ref().unwrap(), &assigned_name, ignore_empty_string_literal);
            factory.update_binding_element_r39k13(
                node,
                d.dot_dot_dot_token.as_ref(),
                d.property_name.as_ref(),
                d.name.as_ref(),
                Some(&initializer),
            )
        }
        NodeData::PropertyDeclaration(d) => {
            let (assigned_name, name) =
                get_assigned_name_of_property_name(emit_context, &d.name, assigned_name_text);
            let initializer =
                finish_transform_named_evaluation(emit_context, d.initializer.as_ref().unwrap(), &assigned_name, ignore_empty_string_literal);
            factory.update_property_declaration_r39k13(
                node,
                node.modifiers().cloned(),
                &name,
                None,
                None,
                Some(&initializer),
            )
        }
        NodeData::BinaryExpression(_) => transform_named_evaluation_of_assignment_expression(
            emit_context,
            node,
            ignore_empty_string_literal,
            assigned_name_text,
        ),
        NodeData::ExportAssignment(d) => {
            let assigned_name: Arc<Node> = if !assigned_name_text.is_empty() {
                factory.new_string_literal(assigned_name_text, 0)
            } else if d.is_export_equals {
                factory.new_string_literal("", 0)
            } else {
                factory.new_string_literal("default", 0)
            };
            let expression =
                finish_transform_named_evaluation(emit_context, &d.expression, &assigned_name, ignore_empty_string_literal);
            factory.update_export_assignment_r39k13(node, None, d.is_export_equals, None, &expression)
        }
        _ => panic!("Unhandled case in transform_named_evaluation"),
    }
}

pub fn transform_named_evaluation_of_assignment_expression(
    emit_context: &EmitContext,
    node: &Arc<Node>,
    ignore_empty_string_literal: bool,
    assigned_name_text: &str,
) -> Arc<Node> {
    let factory = &emit_context.factory();
    let (left, right, operator_token) = match &node.data {
        NodeData::BinaryExpression(d) => (d.left.clone(), d.right.clone(), d.operator_token.clone()),
        _ => unreachable!(),
    };
    let assigned_name: Arc<Node> = if !assigned_name_text.is_empty() {
        factory.new_string_literal(assigned_name_text, 0)
    } else {
        get_assigned_name_of_identifier(emit_context, &left, &right)
    };
    let right = finish_transform_named_evaluation(emit_context, &right, &assigned_name, ignore_empty_string_literal);
    factory.update_binary_expression_r39k13(node, &left, &operator_token, &right)
}

fn block_statements(body: &Arc<Node>) -> Vec<Arc<Node>> {
    match &body.data {
        NodeData::Block(d) => d.statements.nodes.clone(),
        _ => vec![],
    }
}

fn block_body(block: &Arc<Node>) -> Arc<Node> {
    match &block.data {
        NodeData::ClassStaticBlockDeclaration(d) => d.body.clone(),
        _ => panic!("block_body: not a class static block declaration"),
    }
}

fn r39k05_set_function_name_helper(
    factory: &crate::printer::NodeFactory<'_>,
    fn_expr: &Arc<Node>,
    name: &Arc<Node>,
) -> Arc<Node> {
    crate::mig::wt1b::r39k05_defs::new_set_function_name_helper_r39k05(factory, fn_expr, name, None)
}

fn emit_context_mut(emit_context: &EmitContext) -> &mut EmitContext {
    unsafe { &mut *(emit_context as *const EmitContext as *mut EmitContext) }
}

fn string_literal_text(node: &Arc<Node>) -> &str {
    match &node.data {
        NodeData::StringLiteral(d) => &d.text,
        _ => "",
    }
}
