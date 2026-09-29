#![allow(unused_imports)]
use std::sync::Arc;
use tsox_frontend::ast::{Node, NodeFlags, NodeList, SyntaxKind};
use tsox_frontend::ast::subtree_facts::SubtreeFacts;
use tsox_frontend::ast::mig::m3c_2::subtree_facts;
use tsox_frontend::ast::{is_parenthesized_expression, is_synthetic_reference_expression, skip_parentheses};
use crate::printer::{EmitContext, NodeFactory};
use crate::mig::m3m::TransformOptions;
use crate::mig::m4m_5::is_simple_copiable_expression;
use super::m4i_8::r39k14_defs::{R39K14NodeExt, R39K14NodeFactoryExt};
use crate::mig::m4i_12::r36k17_defs::NodeAsR36k17Ext;
use crate::mig::m4e::r39k01_defs::R39K01DataExt;
pub struct OptionalChainTransformer {
    pub(crate) emit_context: EmitContext,
}

pub fn new_optional_chain_transformer(opts: &TransformOptions) -> OptionalChainTransformer {
    OptionalChainTransformer {
        emit_context: opts.context.clone(),
    }
}

impl OptionalChainTransformer {
    pub(crate) fn factory(&self) -> NodeFactory<'_> {
        NodeFactory::new(&self.emit_context)
    }

}
impl OptionalChainTransformer {
    pub(crate) fn visit_node_opt(&mut self, node: Option<&Arc<Node>>) -> Option<Arc<Node>> {
        node.map(|node| self.visit(node.clone()))
    }

    pub(crate) fn visit_nodes_slice(&mut self, nodes: &[Arc<Node>]) -> Arc<NodeList> {
        Arc::new(NodeList::new(
            nodes.iter().map(|node| self.visit(node.clone())).collect(),
        ))
    }

    pub(crate) fn visit_each_child(&mut self, node: Arc<Node>) -> Arc<Node> {
        let mut changed = false;
        tsox_frontend::ast::node_data_generated::for_each_child(&node, |child| {
            let visited = self.visit(child.clone());
            changed |= !Arc::ptr_eq(&visited, child);
            true
        });
        if !changed {
            return node;
        }
        panic!(
            "OptionalChainTransformer visit_each_child rebuild for kind {:?} pending factory update-function port (progress_notes_r39k14.md)",
            node.kind
        )
    }

}
impl OptionalChainTransformer {
    pub fn visit(&mut self, node: Arc<Node>) -> Arc<Node> {
        if !subtree_facts(&node).intersects(SubtreeFacts::CONTAINS_OPTIONAL_CHAINING) {
            return node;
        }
        match node.kind {
            SyntaxKind::CallExpression => self.visit_call_expression(node, false),
            SyntaxKind::PropertyAccessExpression | SyntaxKind::ElementAccessExpression => {
                if node.flags.intersects(NodeFlags::OptionalChain) {
                    return self.visit_optional_expression(node, false, false);
                }
                self.visit_each_child(node)
            }
            SyntaxKind::DeleteExpression => self.visit_delete_expression(node),
            _ => self.visit_each_child(node),
        }
    }

}
impl OptionalChainTransformer {
    fn visit_call_expression(&mut self, node: Arc<Node>, capture_this_arg: bool) -> Arc<Node> {
        let data = node.as_call_expression();
        if node.flags.intersects(NodeFlags::OptionalChain) {
            return self.visit_optional_expression(node, capture_this_arg, false);
        }
        if is_parenthesized_expression(&data.expression) {
            let unwrapped = skip_parentheses(&data.expression);
            if unwrapped.flags.intersects(NodeFlags::OptionalChain) {
                let expression = self.visit_parenthesized_expression(&data.expression, true, false);
                let args = self.visit_nodes_slice(&data.arguments.nodes);
                if is_synthetic_reference_expression(&expression) {
                    let synth = expression.as_synthetic_reference_expression();
                    let mut res = self.factory().new_function_call_call(
                        &synth.expression,
                        &synth.this_arg,
                        &args.nodes,
                    );
                    if let Some(res_data) = Arc::get_mut(&mut res) {
                        res_data.loc = node.loc;
                    }
                    self.emit_context.set_original(&res, &node);
                    return res;
                }
                return self.factory().update_call_expression(
                    &node,
                    Some(expression),
                    None,
                    None,
                    args,
                    node.flags,
                );
            }
        }
        self.visit_each_child(node)
    }

}
impl OptionalChainTransformer {
    fn visit_parenthesized_expression(
        &mut self,
        node: &Arc<Node>,
        capture_this_arg: bool,
        is_delete: bool,
    ) -> Arc<Node> {
        let data = node.as_parenthesized_expression();
        let expr = self.visit_non_optional_expression(&data.expression, capture_this_arg, is_delete);
        if is_synthetic_reference_expression(&expr) {
            let synth = expr.as_synthetic_reference_expression();
            let res = self.factory().new_synthetic_reference_expression(
                &self.factory().update_parenthesized_expression(node, &synth.expression),
                &synth.this_arg,
            );
            self.emit_context.set_original(&res, node);
            return res;
        }
        self.factory().update_parenthesized_expression(node, &expr)
    }

}
impl OptionalChainTransformer {
    fn visit_property_or_element_access_expression(
        &mut self,
        node: &Arc<Node>,
        capture_this_arg: bool,
        is_delete: bool,
    ) -> Arc<Node> {
        if node.flags.intersects(NodeFlags::OptionalChain) {
            return self.visit_optional_expression(node.clone(), capture_this_arg, is_delete);
        }
        let mut expression = self.visit_node_opt(node.expression());
        debug_assert!(
            expression.is_none()
                || !expression
                    .as_deref()
                    .map_or(false, is_synthetic_reference_expression)
        );

        let mut this_arg: Option<Arc<Node>> = None;
        if capture_this_arg {
            let expr = expression.clone().unwrap();
            if !is_simple_copiable_expression(&expr) {
                this_arg = Some(
                    self.factory()
                        .generated_name_node(&self.factory().new_temp_variable()),
                );
                self.emit_context
                    .add_variable_declaration(this_arg.as_ref().unwrap());
                expression = Some(
                    self.factory()
                        .new_assignment_expression(&this_arg.clone().unwrap(), &expr),
                );
            } else {
                this_arg = Some(expr);
            }
        }

        let expression = if node.kind == SyntaxKind::PropertyAccessExpression {
            let p = node.as_property_access_expression();
            let visited_name = self.visit_node_opt(Some(&p.name));
            self.factory().update_property_access_expression(
                node,
                expression,
                p.question_dot_token.clone(),
                visited_name,
                node.flags,
            )
        } else {
            let p = node.as_element_access_expression();
            let visited_argument = self.visit_node_opt(Some(&p.argument_expression));
            self.factory().update_element_access_expression(
                node,
                expression,
                p.question_dot_token.clone(),
                visited_argument,
                node.flags,
            )
        };

        if let Some(this_arg) = this_arg {
            let res = self
                .factory()
                .new_synthetic_reference_expression(&expression, &this_arg);
            self.emit_context.set_original(&res, node);
            return res;
        }
        expression
    }

}
impl OptionalChainTransformer {
    fn visit_delete_expression(&mut self, node: Arc<Node>) -> Arc<Node> {
        let data = node.as_delete_expression();
        let unwrapped = skip_parentheses(&data.expression);
        if unwrapped.flags.intersects(NodeFlags::OptionalChain) {
            return self.visit_non_optional_expression(&data.expression, false, true);
        }
        self.visit_each_child(node)
    }

}
impl OptionalChainTransformer {
    pub(crate) fn visit_non_optional_expression(
        &mut self,
        node: &Arc<Node>,
        capture_this_arg: bool,
        is_delete: bool,
    ) -> Arc<Node> {
        match node.kind {
            SyntaxKind::ParenthesizedExpression => {
                self.visit_parenthesized_expression(node, capture_this_arg, is_delete)
            }
            SyntaxKind::ElementAccessExpression | SyntaxKind::PropertyAccessExpression => {
                self.visit_property_or_element_access_expression(node, capture_this_arg, is_delete)
            }
            SyntaxKind::CallExpression => self.visit_call_expression(node.clone(), capture_this_arg),
            _ => self.visit_node_opt(Some(node)).unwrap(),
        }
    }

}
