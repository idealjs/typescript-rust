#![allow(unused_imports)]
use std::collections::HashSet;
use std::sync::Arc;
use tsox_core::core::compiler_options::CompilerOptions;
use tsox_core::core::text::TextRange;
use tsox_frontend::ast::{Node, NodeFlags, NodeList, SyntaxKind};
use tsox_frontend::ast::subtree_facts::SubtreeFacts;
use tsox_frontend::ast::mig::m3c_2::subtree_facts;
use tsox_frontend::ast::{is_block, skip_parentheses};
use tsox_frontend::ast::mig::m3f_3::is_assignment_pattern;
use tsox_frontend::ast::mig::m3f_4::is_destructuring_assignment;
use tsox_frontend::ast::mig::w2::contains_object_rest_or_spread;
use super::m4i_3::ObjectRestSpreadTransformer;
use crate::mig::m4i_9::r38k6_defs::R38K6DataExt;
use crate::mig::m4m_5::r38k9_defs::R38K9NodeCastExt;
use crate::mig::m4n_5::r36k29_defs::R36K29NodeExt;
use crate::mig::m4j::r36k3_defs::R36K3NodeAccessExt;
use crate::mig::m4e_2::{FlattenLevel, flatten_destructuring_assignment};
use crate::mig::m4k_2::Transformer;
use crate::mig::m4g::r39k15_defs::NodeFactoryR39k15;
use crate::mig::m4l_3::r39k10_defs::R39K10NodeExt;
use tsox_frontend::ast::mig::m3c::statement_list;
impl ObjectRestSpreadTransformer {
    pub(crate) fn visit_for_of_statement(&mut self, node: Arc<Node>) -> Arc<Node> { ::tsox_core::fntrace::enter("visit_for_of_statement"); 
        let data = node.as_for_in_or_of_statement();
        let initializer = data.initializer.clone();
        if subtree_facts(&initializer).intersects(SubtreeFacts::CONTAINS_OBJECT_REST_OR_SPREAD)
            || (is_assignment_pattern(&initializer) && contains_object_rest_or_spread(&initializer))
        {
            let initializer_without_parens = skip_parentheses(&initializer);
            if initializer_without_parens.kind == SyntaxKind::VariableDeclarationList
                || is_assignment_pattern(&initializer_without_parens)
            {
                let mut body_location = TextRange::default();
                let mut statements_location = TextRange::default();
                let temp = self
                    .factory()
                    .generated_name_node(&self.factory().new_temp_variable());
                let res = self.visit_node(Some(
                    &self
                        .factory()
                        .create_for_of_binding_statement(&initializer_without_parens, &temp),
                ));
                let mut statements: Vec<Arc<Node>> = Vec::new();
                if let Some(res) = res {
                    statements.push(res);
                }
                let statement = data.statement.clone();
                if is_block(&statement) {
                    for s in statement.as_block().statements.nodes.iter() {
                        let visited = self.visit_each_child(s);
                        statements.push(visited);
                    }
                    body_location = statement.loc;
                    statements_location = statement_list(&statement).unwrap().loc;
                } else if statement.kind != SyntaxKind::Unknown {
                    statements.push(self.visit_each_child(&statement));
                    body_location = statement.loc;
                    statements_location = statement.loc;
                }

                let list = self.factory().new_variable_declaration_list(
                    &self.factory().new_node_list(vec![
                        self.factory().new_variable_declaration(&temp, None, None, None),
                    ]),
                    NodeFlags::Let,
                );
                let mut list = list;
                if let Some(list_data) = Arc::get_mut(&mut list) {
                    list_data.loc = initializer.loc;
                }

                let expr = self.visit_each_child(&data.expression);

                let statements_list = self.factory().new_node_list(statements);
                let mut statements_list = statements_list;
                if let Some(statements_data) = Arc::get_mut(&mut statements_list) {
                    statements_data.loc = statements_location;
                }

                let block = self.factory().new_block(&statements_list, true);
                let mut block = block;
                if let Some(block_data) = Arc::get_mut(&mut block) {
                    block_data.loc = body_location;
                }

                return self.factory().update_for_in_or_of_statement(
                    &node,
                    data.await_modifier.clone(),
                    Some(list),
                    Some(expr),
                    Some(block),
                );
            }
        }
        self.visit_each_child(&node)
    }

}
impl ObjectRestSpreadTransformer {
    pub(crate) fn visit_binary_expression(&mut self, node: Arc<Node>, expression_result_is_unused: bool) -> Arc<Node> { ::tsox_core::fntrace::enter("visit_binary_expression"); 
        let data = node.as_binary_expression();
        if is_destructuring_assignment(&node) && contains_object_rest_or_spread(&data.left) {
            let mut tx = Transformer::new(|_, node| Some(node), Some(self.emit_context.clone()));
            let flattened = flatten_destructuring_assignment(
                &mut tx,
                node.clone(),
                !expression_result_is_unused,
                FlattenLevel::ObjectRest,
                None,
            );
            return flattened.unwrap_or_else(|| node.clone());
        }
        if data.operator_token.kind == SyntaxKind::CommaToken {
            self.expression_result_is_unused = true;
            let left = self.visit_node(Some(&data.left));
            self.expression_result_is_unused = expression_result_is_unused;
            let right = self.visit_node(Some(&data.right));
            return self.factory().update_binary_expression_r39k13(
                &node,
                &left.unwrap(),
                &data.operator_token,
                &right.unwrap(),
            );
        }
        self.visit_each_child(&node)
    }

}
impl ObjectRestSpreadTransformer {
    pub(crate) fn visit_object_literal_expression(&mut self, node: Arc<Node>) -> Arc<Node> { ::tsox_core::fntrace::enter("visit_object_literal_expression"); 
        let data = node.as_object_literal_expression();
        if !subtree_facts(&node).intersects(SubtreeFacts::CONTAINS_OBJECT_REST_OR_SPREAD) {
            return self.visit_each_child(&node);
        }

        let mut objects = self.chunk_object_literal_elements(Some(data.properties.as_ref()));
        if !objects.is_empty() && objects[0].kind != SyntaxKind::ObjectLiteralExpression {
            objects.insert(
                0,
                self.factory()
                    .new_object_literal_expression(&self.factory().new_node_list(Vec::new()), false),
            );
        }
        if objects.len() > 1 {
            let mut expression = objects[0].clone();
            for (i, obj) in objects.iter().enumerate() {
                if i == 0 {
                    continue;
                }
                expression = self.factory().new_assign_helper(
                    vec![expression, obj.clone()],
                    self.compiler_options.get_emit_script_target(),
                );
            }
            return expression;
        }
        self.factory().new_assign_helper(
            objects,
            self.compiler_options.get_emit_script_target(),
        )
    }

}
impl ObjectRestSpreadTransformer {
    fn chunk_object_literal_elements(&mut self, list: Option<&NodeList>) -> Vec<Arc<Node>> { ::tsox_core::fntrace::enter("chunk_object_literal_elements"); 
        let list = match list {
            Some(l) if !l.nodes.is_empty() => l,
            _ => return Vec::new(),
        };
        let mut chunk_object: Vec<Arc<Node>> = Vec::new();
        let mut objects: Vec<Arc<Node>> = Vec::new();
        for e in &list.nodes {
            if e.kind == SyntaxKind::SpreadAssignment {
                if !chunk_object.is_empty() {
                    objects.push(
                        self.factory()
                            .new_object_literal_expression(&self.factory().new_node_list(chunk_object.clone()), false),
                    );
                    chunk_object.clear();
                }
                let target = e.expression();
                objects.push(self.visit_node(target).unwrap());
            } else {
                let elem = if e.kind == SyntaxKind::PropertyAssignment {
                    let visited_initializer = self.visit_node(e.initializer()).unwrap();
                    self.factory().new_property_assignment(
                        None,
                        e.name().unwrap(),
                        None,
                        None,
                        &visited_initializer,
                    )
                } else {
                    self.visit_node(Some(e)).unwrap()
                };
                chunk_object.push(elem);
            }
        }
        if !chunk_object.is_empty() {
            objects.push(
                self.factory()
                    .new_object_literal_expression(&self.factory().new_node_list(chunk_object), false),
            );
        }
        objects
    }
}
