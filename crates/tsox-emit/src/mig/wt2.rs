#![allow(unused_imports)]
#![allow(dead_code)]

use std::sync::Arc;

use tsox_frontend::ast::node::Node;
use tsox_frontend::ast::node_data_generated::{
    is_postfix_unary_expression, is_prefix_unary_expression, NodeData,
    PostfixUnaryExpressionData,
};
use tsox_frontend::ast::syntax_kind_generated::SyntaxKind;

use crate::printer::generated_identifier_flags::NodeFactory;
use crate::printer::EmitContext;

impl<'a> NodeFactory<'a> {
    pub fn new_postfix_unary_expression(
        &self,
        operand: &Arc<Node>,
        operator: SyntaxKind,
    ) -> Arc<Node> { ::tsox_core::fntrace::enter("new_postfix_unary_expression"); 
        Arc::new(Node::new(
            SyntaxKind::PostfixUnaryExpression,
            NodeData::PostfixUnaryExpression(PostfixUnaryExpressionData {
                operand: operand.clone(),
                operator,
            }),
        ))
    }
}

pub fn expand_pre_or_postfix_increment_or_decrement_expression(
    factory: &NodeFactory<'_>,
    emit_context: &mut EmitContext,
    node: &Arc<Node>,
    expression: &Arc<Node>,
    result_variable: Option<&Arc<Node>>,
) -> Arc<Node> { ::tsox_core::fntrace::enter("expand_pre_or_postfix_increment_or_decrement_expression"); 
    let (operator, operand): (SyntaxKind, &Arc<Node>) = if is_prefix_unary_expression(node) {
        match &node.data {
            NodeData::PrefixUnaryExpression(d) => (d.operator, &d.operand),
            _ => unreachable!(),
        }
    } else {
        match &node.data {
            NodeData::PostfixUnaryExpression(d) => (d.operator, &d.operand),
            _ => unreachable!(),
        }
    };

    let temp = factory.generated_name_node(&factory.new_temp_variable());
    emit_context.add_variable_declaration(&temp);
    let mut expression = factory.new_assignment_expression(&temp, expression);
    if let Some(n) = Arc::get_mut(&mut expression) {
        n.loc = operand.loc;
    }

    let mut operation = if is_prefix_unary_expression(node) {
        factory.new_prefix_unary_expression(operator, &temp)
    } else {
        factory.new_postfix_unary_expression(&temp, operator)
    };
    if let Some(n) = Arc::get_mut(&mut operation) {
        n.loc = node.loc;
    }

    if let Some(result_variable) = result_variable {
        operation = factory.new_assignment_expression(result_variable, &operation);
        if let Some(n) = Arc::get_mut(&mut operation) {
            n.loc = node.loc;
        }
    }

    let mut expression = factory.new_comma_expression(&expression, &operation);
    if let Some(n) = Arc::get_mut(&mut expression) {
        n.loc = node.loc;
    }

    if is_postfix_unary_expression(node) {
        expression = factory.new_comma_expression(&expression, &temp);
        if let Some(n) = Arc::get_mut(&mut expression) {
            n.loc = node.loc;
        }
    }

    expression
}
