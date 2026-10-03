use std::sync::Arc;
use tsox_frontend::ast::*;

use crate::mig::m3m::TransformOptions;
use crate::mig::m4k_2::Transformer;
use crate::mig::m4m_2::is_simple_copiable_expression;
use tsox_frontend::ast::subtree_facts::SubtreeFacts;

#[path = "r37k18_defs.rs"]
pub mod r37k18_defs;
#[path = "r39k13_defs.rs"]
pub mod r39k13_defs;
#[path = "r40k17_defs.rs"]
pub mod r40k17_defs;

use r37k18_defs::R37K18NodeVisitorExt;
use r39k13_defs::set_loc;

pub struct ExponentiationTransformer;

impl ExponentiationTransformer {
    pub fn visit(tx: &mut Transformer, node: &Arc<Node>) -> Arc<Node> { ::tsox_core::fntrace::enter("visit"); 
        if !node.subtree_facts().intersects(SubtreeFacts::CONTAINS_EXPONENTIATION_OPERATOR) {
            return node.clone();
        }
        match node.kind {
            SyntaxKind::BinaryExpression => Self::visit_binary_expression(tx, node),
            _ => tx.visitor().visit_each_child(node),
        }
    }

    fn visit_binary_expression(tx: &mut Transformer, node: &Arc<Node>) -> Arc<Node> { ::tsox_core::fntrace::enter("visit_binary_expression"); 
        let operator_kind = match &node.data {
            NodeData::BinaryExpression(d) => d.operator_token.kind,
            _ => return tx.visitor().visit_each_child(node),
        };
        match operator_kind {
            SyntaxKind::AsteriskAsteriskEqualsToken => {
                Self::visit_exponentiation_assignment_expression(tx, node)
            }
            SyntaxKind::AsteriskAsteriskToken => Self::visit_exponentiation_expression(tx, node),
            _ => tx.visitor().visit_each_child(node),
        }
    }

    fn visit_exponentiation_assignment_expression(tx: &mut Transformer, node: &Arc<Node>) -> Arc<Node> { ::tsox_core::fntrace::enter("visit_exponentiation_assignment_expression"); 
        let (left_node, right_node) = match &node.data {
            NodeData::BinaryExpression(d) => (d.left.clone(), d.right.clone()),
            _ => unreachable!(),
        };
        let left = tx.visitor().visit_node(&left_node);
        let right = tx.visitor().visit_node(&right_node);
        let factory = tx.factory();
        let mut emit_context = tx.emit_context();
        let mut target: Arc<Node>;
        let mut value: Arc<Node>;
        if is_element_access_expression(&left) {
            let expression_temp = factory.generated_name_node(&factory.new_temp_variable());
            emit_context.add_variable_declaration(&expression_temp);
            let argument_expression_temp =
                factory.generated_name_node(&factory.new_temp_variable());
            emit_context.add_variable_declaration(&argument_expression_temp);

            let left_expression = expression(&left).clone();
            let argument_expression = argument_expression(&left).clone();

            let mut obj_expr = r37k18_new_assignment_expression(
                &factory,
                &expression_temp,
                &left_expression,
            );
            set_loc(&mut obj_expr, left_expression.loc);
            let mut access_expr = r37k18_new_assignment_expression(
                &factory,
                &argument_expression_temp,
                &argument_expression,
            );
            set_loc(&mut access_expr, argument_expression.loc);

            target = factory.new_element_access_expression(&obj_expr, None, &access_expr, NodeFlags::empty());

            value = factory.new_element_access_expression(&expression_temp, None, &argument_expression_temp, NodeFlags::empty());
            set_loc(&mut value, left.loc);
        } else if is_property_access_expression(&left) {
            let expression_temp = factory.generated_name_node(&factory.new_temp_variable());
            emit_context.add_variable_declaration(&expression_temp);
            let left_expression = expression(&left).clone();
            let mut assignment =
                r37k18_new_assignment_expression(&factory, &expression_temp, &left_expression);
            set_loc(&mut assignment, left_expression.loc);
            let name = left.name().unwrap().clone();
            target = factory.new_property_access_expression(&assignment, None, &name, NodeFlags::empty());
            set_loc(&mut target, left.loc);

            value = factory.new_property_access_expression(&expression_temp, None, &name, NodeFlags::empty());
            set_loc(&mut value, left.loc);
        } else {
            target = left.clone();
            value = left;
        }

        let mut rhs = factory.new_global_method_call("Math", "pow", &[value, right]);
        set_loc(&mut rhs, node.loc);
        let mut result = r37k18_new_assignment_expression(&factory, &target, &rhs);
        set_loc(&mut result, node.loc);
        result
    }

    fn visit_exponentiation_expression(tx: &mut Transformer, node: &Arc<Node>) -> Arc<Node> { ::tsox_core::fntrace::enter("visit_exponentiation_expression"); 
        let (left_node, right_node) = match &node.data {
            NodeData::BinaryExpression(d) => (d.left.clone(), d.right.clone()),
            _ => unreachable!(),
        };
        let left = tx.visitor().visit_node(&left_node);
        let right = tx.visitor().visit_node(&right_node);
        let mut result = tx
            .factory()
            .new_global_method_call("Math", "pow", &[left, right]);
        set_loc(&mut result, node.loc);
        result
    }
}

pub fn new_exponentiation_transformer(opts: &TransformOptions) -> Transformer { ::tsox_core::fntrace::enter("new_exponentiation_transformer"); 
    Transformer::new(exponentiation_transformer_visit, Some(opts.context.clone()))
}

fn exponentiation_transformer_visit(tx: &mut Transformer, node: Arc<Node>) -> Option<Arc<Node>> { ::tsox_core::fntrace::enter("exponentiation_transformer_visit"); 
    Some(ExponentiationTransformer::visit(tx, &node))
}

pub struct LogicalAssignmentTransformer;

impl LogicalAssignmentTransformer {
    pub fn visit(tx: &mut Transformer, node: &Arc<Node>) -> Arc<Node> { ::tsox_core::fntrace::enter("visit"); 
        if !node.subtree_facts().intersects(SubtreeFacts::CONTAINS_LOGICAL_ASSIGNMENTS) {
            return node.clone();
        }
        match node.kind {
            SyntaxKind::BinaryExpression => Self::visit_binary_expression(tx, node),
            _ => tx.visitor().visit_each_child(node),
        }
    }

    fn visit_binary_expression(tx: &mut Transformer, node: &Arc<Node>) -> Arc<Node> { ::tsox_core::fntrace::enter("visit_binary_expression"); 
        let (left_node, right_node, operator_kind) = match &node.data {
            NodeData::BinaryExpression(d) => {
                (d.left.clone(), d.right.clone(), d.operator_token.kind)
            }
            _ => unreachable!(),
        };
        let non_assignment_operator = match operator_kind {
            SyntaxKind::BarBarEqualsToken => SyntaxKind::BarBarToken,
            SyntaxKind::AmpersandAmpersandEqualsToken => SyntaxKind::AmpersandAmpersandToken,
            SyntaxKind::QuestionQuestionEqualsToken => SyntaxKind::QuestionQuestionToken,
            _ => return tx.visitor().visit_each_child(node),
        };

        let left = skip_parentheses(&tx.visitor().visit_node(&left_node));
        let mut assignment_target = left.clone();
        let right = skip_parentheses(&tx.visitor().visit_node(&right_node));

        let factory = tx.factory();
        let mut emit_context = tx.emit_context();
        let mut left = left;

        if is_access_expression(&left) {
            let left_expression = expression(&left).clone();
            let property_access_target_simple_copiable = is_simple_copiable_expression(&left_expression);
            let mut property_access_target = left_expression.clone();
            let mut property_access_target_assignment = left_expression.clone();
            if !property_access_target_simple_copiable {
                property_access_target = factory.generated_name_node(&factory.new_temp_variable());
                emit_context.add_variable_declaration(&property_access_target);
                property_access_target_assignment =
                    r37k18_new_assignment_expression(&factory, &property_access_target, &left_expression);
            }

            if is_property_access_expression(&left) {
                let name = left.name().unwrap().clone();
                assignment_target = factory.new_property_access_expression(
                    &property_access_target,
                    None,
                    &name,
                    NodeFlags::empty(),
                );
                left = factory.new_property_access_expression(
                    &property_access_target_assignment,
                    None,
                    &name,
                    NodeFlags::empty(),
                );
            } else {
                let left_argument = argument_expression(&left).clone();
                let element_access_argument_simple_copiable = is_simple_copiable_expression(&left_argument);
                let mut element_access_argument = left_argument.clone();
                let mut argument_expr = left_argument.clone();
                if !element_access_argument_simple_copiable {
                    element_access_argument =
                        factory.generated_name_node(&factory.new_temp_variable());
                    emit_context.add_variable_declaration(&element_access_argument);
                    argument_expr = r37k18_new_assignment_expression(
                        &factory,
                        &element_access_argument,
                        &left_argument,
                    );
                }

                assignment_target = factory.new_element_access_expression(
                    &property_access_target,
                    None,
                    &element_access_argument,
                    NodeFlags::empty(),
                );
                left = factory.new_element_access_expression(
                    &property_access_target_assignment,
                    None,
                    &argument_expr,
                    NodeFlags::empty(),
                );
            }
        }

        let operator_token = factory.new_token(non_assignment_operator);
        let assignment = r37k18_new_assignment_expression(&factory, &assignment_target, &right);
        let parenthesized = factory.new_parenthesized_expression(&assignment);
        factory.new_binary_expression(None, &left, None, &operator_token, &parenthesized)
    }
}

pub fn new_logical_assignment_transformer(opts: &TransformOptions) -> Transformer { ::tsox_core::fntrace::enter("new_logical_assignment_transformer"); 
    Transformer::new(logical_assignment_transformer_visit, Some(opts.context.clone()))
}

fn logical_assignment_transformer_visit(tx: &mut Transformer, node: Arc<Node>) -> Option<Arc<Node>> { ::tsox_core::fntrace::enter("logical_assignment_transformer_visit"); 
    Some(LogicalAssignmentTransformer::visit(tx, &node))
}

fn r37k18_new_assignment_expression(
    factory: &crate::printer::NodeFactory<'_>,
    left: &Arc<Node>,
    right: &Arc<Node>,
) -> Arc<Node> { ::tsox_core::fntrace::enter("r37k18_new_assignment_expression"); 
    factory.new_binary_expression(
        None,
        left,
        None,
        &factory.new_token(SyntaxKind::EqualsToken),
        right,
    )
}

fn expression(node: &Arc<Node>) -> &Arc<Node> { ::tsox_core::fntrace::enter("expression"); 
    match &node.data {
        NodeData::ElementAccessExpression(d) => &d.expression,
        NodeData::PropertyAccessExpression(d) => &d.expression,
        _ => panic!("expression: not an access expression"),
    }
}

fn argument_expression(node: &Arc<Node>) -> &Arc<Node> { ::tsox_core::fntrace::enter("argument_expression"); 
    match &node.data {
        NodeData::ElementAccessExpression(d) => &d.argument_expression,
        _ => panic!("argument_expression: not an element access expression"),
    }
}
