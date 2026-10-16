#![allow(unused_imports, dead_code)]

use std::sync::Arc;

use tsox_core::core::text::TextRange;
use tsox_frontend::ast::node::{Node, NodeList};
use tsox_frontend::ast::node_data_generated::{
    is_element_access_expression, is_identifier, is_property_access_expression,
};
use tsox_frontend::ast::syntax_kind_generated::SyntaxKind;
use tsox_frontend::ast::utilities::is_compound_assignment;

use crate::mig::m4g::r33k7_defs::{
    binary_left, binary_right, binary_operator_token, element_access_argument, property_access_name,
};
use crate::mig::m4g::{ClassFieldsTransformer, ClassFacts, ClassLexicalEnvironment};
use crate::mig::m4g_2::r37k13_defs::{
    ClassFieldsTransformerR37k13, K13Visitor, NodeFactoryR37k13,
};
use crate::mig::m4g::r39k15_defs::{NodeFactoryR39k15, K13VisitorR39k15};
use crate::mig::m4m_2::is_simple_inlineable_expression;
use crate::mig::m4m_4::get_non_assignment_operator_for_compound_assignment;

pub trait K13VisitorR40k14 {
    fn visit_nodes(&mut self, nodes: &Arc<NodeList>) -> Arc<NodeList>;
}

impl K13VisitorR40k14 for K13Visitor {
    fn visit_nodes(&mut self, nodes: &Arc<NodeList>) -> Arc<NodeList> { ::tsox_core::fntrace::enter("visit_nodes"); 
        let visited: Vec<Arc<Node>> = nodes.nodes.iter().map(|n| self.visit_each_child(n)).collect();
        Arc::new(NodeList::new(visited))
    }
}

fn set_loc(mut node: Arc<Node>, loc: TextRange) -> Arc<Node> { ::tsox_core::fntrace::enter("set_loc"); 
    if let Some(n) = Arc::get_mut(&mut node) {
        n.loc = loc;
    }
    node
}

pub trait ClassFieldsTransformerR40k14 {
    fn visit_binary_super_assignment(
        &mut self,
        node: &Arc<Node>,
        discarded: bool,
        data: &ClassLexicalEnvironment,
        left: &Arc<Node>,
    ) -> Option<Arc<Node>>;
}

impl ClassFieldsTransformerR40k14 for ClassFieldsTransformer {
    fn visit_binary_super_assignment(
        &mut self,
        node: &Arc<Node>,
        discarded: bool,
        data: &ClassLexicalEnvironment,
        left: &Arc<Node>,
    ) -> Option<Arc<Node>> { ::tsox_core::fntrace::enter("visit_binary_super_assignment"); 
        if data.facts.contains(ClassFacts::ClassWasDecorated) {
            return Some(self.factory().update_binary_expression(
                node,
                None,
                self.visit_invalid_super_property(binary_left(node)),
                None,
                binary_operator_token(node).clone(),
                self.visitor().visit_node(binary_right(node)),
            ));
        }
        let class_constructor = data.class_constructor.as_ref()?;
        let super_class_reference = data.super_class_reference.as_ref()?;

        let left_node = binary_left(node);
        let mut setter_name: Option<Arc<Node>> = None;
        if is_element_access_expression(left_node) {
            setter_name = Some(
                self.visitor()
                    .visit_node(element_access_argument(left_node).unwrap()),
            );
        } else if is_property_access_expression(left_node)
            && is_identifier(property_access_name(left_node))
        {
            setter_name = Some(
                self.factory()
                    .new_string_literal_from_node(property_access_name(left_node)),
            );
        }
        let mut setter_name = setter_name?;

        let mut expression = self.visitor().visit_node(binary_right(node));
        let operator_kind = binary_operator_token(node).kind;
        if is_compound_assignment(operator_kind) {
            let mut getter_name = setter_name.clone();
            if !is_simple_inlineable_expression(&setter_name) {
                let temp = self.factory().new_temp_variable_r37k13();
                self.emit_context().add_variable_declaration(&temp);
                getter_name = temp.clone();
                setter_name = self.factory().new_assignment_expression(&temp, &setter_name);
            }
            let super_property_get = self
                .factory()
                .new_reflect_get_call(super_class_reference, &getter_name, class_constructor);
            self.emit_context().set_original(&super_property_get, left_node);
            let super_property_get = set_loc(super_property_get, left_node.loc);
            expression = self.factory().new_binary_expression(
                None,
                &super_property_get,
                None,
                self.factory()
                    .new_token(get_non_assignment_operator_for_compound_assignment(operator_kind)),
                &expression,
            );
            expression = set_loc(expression, node.loc);
        }

        let mut temp: Option<Arc<Node>> = None;
        if !discarded {
            let temp_variable = self.factory().new_temp_variable_r37k13();
            self.emit_context().add_variable_declaration(&temp_variable);
            temp = Some(temp_variable);
        }
        if let Some(temp) = &temp {
            expression = set_loc(
                self.factory().new_assignment_expression(temp, &expression),
                node.loc,
            );
        }

        let mut expression = self.factory().new_reflect_set_call(
            super_class_reference,
            &setter_name,
            &expression,
            class_constructor,
        );
        self.emit_context().set_original(&expression, node);
        expression = set_loc(expression, node.loc);

        if let Some(temp) = &temp {
            expression = set_loc(
                self.factory().new_comma_expression(&expression, temp),
                node.loc,
            );
        }
        Some(expression)
    }
}
