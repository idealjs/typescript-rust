#![allow(unused_imports)]
use std::sync::Arc;
use tsox_frontend::ast::{Node, NodeFlags, SyntaxKind};
use tsox_frontend::ast::{
    is_call_expression, is_non_null_expression, is_synthetic_reference_expression,
    is_tagged_template_expression,
};
use tsox_frontend::ast::skip_partially_emitted_expressions_arc as skip_partially_emitted_expressions;
use tsox_frontend::ast::mig::m3b::question_dot_token;
use tsox_frontend::ast::mig::m3g_3::OuterExpressionKinds;
use tsox_frontend::format::mig::m4o::EmitFlags;
use super::m4i_6::OptionalChainTransformer;
use super::m4i_11::create_not_null_condition;
use super::m4i_8::r39k14_defs::{R39K14NodeExt, R39K14NodeFactoryExt};
use crate::mig::m4e::r37k1_defs::R37K1DataExt;
use crate::mig::m4m_5::is_simple_copiable_expression;
use crate::mig::m4p_4::r37k15_defs::{R37k15NodeVisitorExt, R37k15PropertyAccessExt};
use tsox_frontend::ast::mig::x1a::argument_list;
impl OptionalChainTransformer {
    pub(crate) fn visit_optional_expression(
        &mut self,
        node: Arc<Node>,
        capture_this_arg: bool,
        is_delete: bool,
    ) -> Arc<Node> {
        let r = flatten_chain(&node);
        let expression = r.expression;
        let chain = r.chain;
        let mut left = self.visit_non_optional_expression(
            &skip_partially_emitted_expressions(&expression),
            is_call_chain(&chain[0]),
            false,
        );
        let mut left_this_arg: Option<Arc<Node>> = None;
        let mut captured_left = left.clone();
        if is_synthetic_reference_expression(&left) {
            let synth = left.as_synthetic_reference_expression();
            left_this_arg = Some(synth.this_arg.clone());
            captured_left = synth.expression.clone();
        }
        let mut left_expression = self.factory().restore_outer_expressions(
            &expression,
            &captured_left,
            OuterExpressionKinds::PARTIALLY_EMITTED_EXPRESSIONS,
        );
        if !is_simple_copiable_expression(&captured_left) {
            captured_left = self
                .factory()
                .generated_name_node(&self.factory().new_temp_variable());
            self.emit_context
                .add_variable_declaration(&captured_left);
            left_expression = self
                .factory()
                .new_assignment_expression(&captured_left, &left_expression);
        }
        let mut right_expression = captured_left.clone();
        let mut this_arg: Option<Arc<Node>> = None;

        for (i, segment) in chain.iter().enumerate() {
            match segment.kind {
                SyntaxKind::ElementAccessExpression | SyntaxKind::PropertyAccessExpression => {
                    if i == chain.len() - 1 && capture_this_arg {
                        if !is_simple_copiable_expression(&right_expression) {
                            this_arg = Some(
                                self.factory()
                                    .generated_name_node(&self.factory().new_temp_variable()),
                            );
                            self.emit_context
                                .add_variable_declaration(this_arg.as_ref().unwrap());
                            right_expression = self.factory().new_assignment_expression(
                                this_arg.as_ref().unwrap(),
                                &right_expression,
                            );
                        } else {
                            this_arg = Some(right_expression.clone());
                        }
                    }
                    if segment.kind == SyntaxKind::ElementAccessExpression {
                        let argument = self
                            .visit_node_opt(Some(&segment.as_element_access_expression().argument_expression))
                            .unwrap();
                        right_expression = self.factory().new_element_access_expression(
                            &right_expression,
                            None,
                            &argument,
                            NodeFlags::empty(),
                        );
                    } else {
                        let name = self
                            .visit_node_opt(Some(&segment.as_property_access_expression().name))
                            .unwrap();
                        right_expression = self.factory().new_property_access_expression(
                            &right_expression,
                            None,
                            &name,
                            NodeFlags::empty(),
                        );
                    }
                }
                SyntaxKind::CallExpression => {
                    if i == 0 && left_this_arg.is_some() {
                        let mut left_this = left_this_arg.clone().unwrap();
                        if !self.emit_context.has_auto_generate_info(&left_this) {
                            left_this = Arc::clone(&left_this);
                            self.emit_context
                                .add_emit_flags(&left_this, EmitFlags::NO_COMMENTS);
                        }
                        let mut call_this_arg = left_this.clone();
                        if left_this.kind == SyntaxKind::SuperKeyword {
                            call_this_arg = self.factory().new_this_expression();
                        }
                        let segment_args = self.visit_nodes_slice(&argument_list(segment).unwrap().nodes);
                        right_expression = self.factory().new_function_call_call(
                            &right_expression,
                            &call_this_arg,
                            &segment_args.nodes,
                        );
                    } else {
                        let segment_args = self.visit_nodes_slice(&argument_list(segment).unwrap().nodes);
                        right_expression = self.factory().new_call_expression(
                            &right_expression,
                            None,
                            None,
                            segment_args,
                            NodeFlags::empty(),
                        );
                    }
                }
                _ => {}
            }
            self.emit_context.set_original(&right_expression, segment);
        }

        let mut target = if is_delete {
            self.factory().new_conditional_expression(
                &create_not_null_condition(&self.emit_context, left_expression.clone(), captured_left.clone(), true),
                &self.factory().new_token(SyntaxKind::QuestionToken),
                &self.factory().new_true_expression(),
                &self.factory().new_token(SyntaxKind::ColonToken),
                &self.factory().new_delete_expression(&right_expression),
            )
        } else {
            self.factory().new_conditional_expression(
                &create_not_null_condition(&self.emit_context, left_expression, captured_left, true),
                &self.factory().new_token(SyntaxKind::QuestionToken),
                &self.factory().new_void_zero_expression(),
                &self.factory().new_token(SyntaxKind::ColonToken),
                &right_expression,
            )
        };
        if let Some(target_data) = Arc::get_mut(&mut target) {
            target_data.loc = node.loc;
        }
        if let Some(this_arg) = this_arg {
            target = self
                .factory()
                .new_synthetic_reference_expression(&target, &this_arg);
        }
        self.emit_context.set_original(&target, &node);
        target
    }
}

pub struct FlattenResult {
    pub expression: Arc<Node>,
    pub chain: Vec<Arc<Node>>,
}

pub fn is_non_null_chain(node: &Arc<Node>) -> bool {
    is_non_null_expression(node) && node.flags.intersects(NodeFlags::OptionalChain)
}

pub fn flatten_chain(chain: &Arc<Node>) -> FlattenResult {
    debug_assert!(!is_non_null_chain(chain));
    let mut links = vec![chain.clone()];
    let mut current = chain.clone();
    while !is_tagged_template_expression(&current) && question_dot_token(&current).is_none() {
        current = skip_partially_emitted_expressions(&current.expression().unwrap());
        debug_assert!(!is_non_null_chain(&current));
        links.insert(0, current.clone());
    }
    FlattenResult {
        expression: current.expression().unwrap().clone(),
        chain: links,
    }
}

pub fn is_call_chain(node: &Arc<Node>) -> bool {
    is_call_expression(node) && node.flags.intersects(NodeFlags::OptionalChain)
}
