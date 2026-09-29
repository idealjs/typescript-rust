#![allow(unused_imports)]
#![allow(dead_code)]
use std::sync::Arc;

use tsox_core::core::core::find_last;
use tsox_core::core::mig::m3j::last_or_nil;
use tsox_core::core::text::TextRange;
use tsox_core::jsnum::Number;
use tsox_frontend::ast::node::Node;
use tsox_frontend::ast::syntax_kind_generated::SyntaxKind;
use tsox_frontend::ast::{
    can_have_modifiers, is_decorator, is_expression_statement, is_method_declaration,
    is_property_declaration, is_super_call, is_try_statement, position_is_synthesized,
    skip_parentheses, NodeData,
};
use tsox_frontend::scanner::TOKEN_FLAGS_NONE;

use crate::mig::m4j::r36k3_defs::R36K3NodeFactoryExt;
use crate::printer::NodeFactory;

pub fn find_super_statement_index_path(statements: &[Arc<Node>], start: usize) -> Vec<usize> {
    let mut indices =
        find_super_statement_index_path_worker(statements, start, Vec::new()).unwrap_or_default();
    indices.reverse();
    indices
}

pub fn find_super_statement_index_path_worker(
    statements: &[Arc<Node>],
    start: usize,
    indices: Vec<usize>,
) -> Option<Vec<usize>> {
    let mut indices = indices;
    for i in start..statements.len() {
        let statement = &statements[i];
        if get_super_call_from_statement(statement).is_some() {
            indices.push(i);
            return Some(indices);
        } else if is_try_statement(statement) {
            let try_block = match &statement.data {
                NodeData::TryStatement(d) => d.try_block.clone(),
                _ => unreachable!(),
            };
            let block_statements = match &try_block.data {
                NodeData::Block(d) => &d.statements.nodes,
                _ => panic!("Block expected"),
            };
            if let Some(mut result) =
                find_super_statement_index_path_worker(block_statements, 0, indices.clone())
            {
                result.push(i);
                return Some(result);
            }
        }
    }
    None
}

pub fn get_super_call_from_statement(statement: &Arc<Node>) -> Option<Arc<Node>> {
    if !is_expression_statement(statement) {
        return None;
    }
    let expression = skip_parentheses(&statement.expression().clone().unwrap());
    if is_super_call(&expression) {
        return Some(expression);
    }
    None
}

pub fn move_range_past_modifiers(node: &Arc<Node>) -> TextRange {
    if is_property_declaration(node) || is_method_declaration(node) {
        return TextRange::new(node.name().unwrap().pos(), node.end());
    }

    let mut last_modifier: Option<Arc<Node>> = None;
    if can_have_modifiers(node) {
        last_modifier = last_or_nil(node.modifier_nodes());
    }

    if let Some(last_modifier) = last_modifier {
        if !position_is_synthesized(last_modifier.end()) {
            return TextRange::new(last_modifier.end(), node.end());
        }
    }
    move_range_past_decorators(node)
}

pub fn move_range_past_decorators(node: &Arc<Node>) -> TextRange {
    let mut last_decorator: Option<Arc<Node>> = None;
    if can_have_modifiers(node) {
        let nodes = node.modifier_nodes();
        if !nodes.is_empty() {
            last_decorator = find_last(nodes, |n: &Arc<Node>| is_decorator(n)).cloned();
        }
    }

    if let Some(last_decorator) = last_decorator {
        if !position_is_synthesized(last_decorator.end()) {
            return TextRange::new(last_decorator.end(), node.end());
        }
    }
    node.loc
}

pub fn get_non_assignment_operator_for_compound_assignment(kind: SyntaxKind) -> SyntaxKind {
    match kind {
        SyntaxKind::PlusEqualsToken => SyntaxKind::PlusToken,
        SyntaxKind::MinusEqualsToken => SyntaxKind::MinusToken,
        SyntaxKind::AsteriskEqualsToken => SyntaxKind::AsteriskToken,
        SyntaxKind::AsteriskAsteriskEqualsToken => SyntaxKind::AsteriskAsteriskToken,
        SyntaxKind::SlashEqualsToken => SyntaxKind::SlashToken,
        SyntaxKind::PercentEqualsToken => SyntaxKind::PercentToken,
        SyntaxKind::LessThanLessThanEqualsToken => SyntaxKind::LessThanLessThanToken,
        SyntaxKind::GreaterThanGreaterThanEqualsToken => SyntaxKind::GreaterThanGreaterThanToken,
        SyntaxKind::GreaterThanGreaterThanGreaterThanEqualsToken => {
            SyntaxKind::GreaterThanGreaterThanGreaterThanToken
        }
        SyntaxKind::AmpersandEqualsToken => SyntaxKind::AmpersandToken,
        SyntaxKind::BarEqualsToken => SyntaxKind::BarToken,
        SyntaxKind::CaretEqualsToken => SyntaxKind::CaretToken,
        SyntaxKind::BarBarEqualsToken => SyntaxKind::BarBarToken,
        SyntaxKind::AmpersandAmpersandEqualsToken => SyntaxKind::AmpersandAmpersandToken,
        SyntaxKind::QuestionQuestionEqualsToken => SyntaxKind::QuestionQuestionToken,
        _ => kind,
    }
}

pub fn constant_expression(value: &ConstantValue, factory: &NodeFactory) -> Option<Arc<Node>> {
    match value {
        ConstantValue::String(s) => Some(factory.new_string_literal(s, TOKEN_FLAGS_NONE)),
        ConstantValue::Number(n) => {
            if n.is_inf() {
                if *n > Number::from(0.0_f64) {
                    return Some(factory.new_identifier("Infinity"));
                }
                return Some(factory.new_prefix_unary_expression(
                    SyntaxKind::MinusToken,
                    &factory.new_identifier("Infinity"),
                ));
            }
            if n.is_nan() {
                return Some(factory.new_identifier("NaN"));
            }
            if *n < Number::from(0.0_f64) {
                return Some(factory.new_prefix_unary_expression(
                    SyntaxKind::MinusToken,
                    &constant_expression(&ConstantValue::Number(-*n), factory)
                        .unwrap(),
                ));
            }
            Some(factory.new_numeric_literal(&n.to_string(), TOKEN_FLAGS_NONE))
        }
    }
}

pub enum ConstantValue {
    String(String),
    Number(Number),
}
