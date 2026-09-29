#![allow(unused_imports)]
#![allow(dead_code)]

use std::sync::Arc;

use super::m4q::r33k12_defs::Set;
use super::m4q::r33k12_defs::{concatenate, every, splice};
use tsox_frontend::ast::deep_clone_node;
use tsox_frontend::ast::node::Node;
use tsox_frontend::ast::node_data_generated::{
    is_binding_pattern, is_function_declaration, is_identifier, is_private_identifier,
    is_variable_statement,
};
use tsox_frontend::ast::is_member_name;
use tsox_frontend::ast::node_flags::NodeFlags;
use tsox_frontend::ast::NodeList;
use tsox_frontend::ast::syntax_kind_generated::SyntaxKind;
use tsox_frontend::ast::is_prologue_directive;

use crate::mig::m4n_7::span_end;
use tsox_frontend::ast::mig::m3c::question_token;
use crate::mig::m4n::r37k19_defs::R37K19ArcNodeExt;
use crate::printer::{AutoGenerateId, EmitContext};
use super::m4q::r33k12_defs::EmitFlags;
use super::m4i_12::r36k17_defs::NodeAsR36k17Ext;

impl EmitContext {
    pub fn add_default_value_assignment_for_binding_pattern(
        &mut self,
        parameter: &Arc<Node>,
    ) -> Arc<Node> {
        let generated_name = self
            .factory()
            .generated_name_node(&self.factory().new_generated_name_for_node(parameter));
        let init_node = match parameter.initializer() {
            Some(initializer) => self.factory().new_conditional_expression(
                &self.factory().new_strict_equality_expression(
                    &generated_name,
                    &self.factory().new_void_zero_expression(),
                ),
                &self.factory().new_token(SyntaxKind::QuestionToken),
                initializer,
                &self.factory().new_token(SyntaxKind::ColonToken),
                &generated_name,
            ),
            None => Arc::clone(&generated_name),
        };
        let declaration = self.factory().new_variable_declaration(
            &parameter.name().expect("parameter requires a name").clone(),
            None,
            parameter.type_node(),
            Some(&init_node),
        );
        let declaration_list =
            self.factory()
                .new_variable_declaration_list(
                    &self.factory().new_node_list(vec![declaration]),
                    NodeFlags::empty(),
                );
        let statement =
            self.factory()
                .new_variable_statement(None, &declaration_list);
        self.add_initialization_statement(&statement);
        self.factory().update_parameter_declaration(
            parameter,
            parameter.modifiers().cloned(),
            parameter.dot_dot_dot_token(),
            &generated_name,
            question_token(parameter),
            parameter.type_node(),
            None,
        )
    }

    pub fn add_default_value_assignment_for_initializer(
        &mut self,
        parameter: &Arc<Node>,
        name: &Arc<Node>,
        initializer: &Arc<Node>,
    ) -> Arc<Node> {
        self.add_emit_flags(
            initializer,
            EmitFlags::NO_SOURCE_MAP | EmitFlags::NO_COMMENTS,
        );
        let name_clone = deep_clone_node(name);
        self.add_emit_flags(&name_clone, EmitFlags::NO_SOURCE_MAP);
        let mut init_assignment = self
            .factory()
            .new_assignment_expression(&name_clone, initializer);
        init_assignment.set_loc(parameter.loc);
        self.add_emit_flags(&init_assignment, EmitFlags::NO_COMMENTS);
        let mut init_block = self.factory().new_block(
            &self.factory()
                .new_node_list(vec![self.factory().new_expression_statement(&init_assignment)]),
            false,
        );
        init_block.set_loc(parameter.loc);
        self.add_emit_flags(
            &init_block,
            EmitFlags::SINGLE_LINE
                | EmitFlags::NO_TRAILING_SOURCE_MAP
                | EmitFlags::NO_TOKEN_SOURCE_MAPS
                | EmitFlags::NO_COMMENTS,
        );
        let type_check = self
            .factory()
            .new_type_check(&deep_clone_node(name), "undefined");
        self.add_initialization_statement(&self.factory().new_if_statement(
            &type_check,
            &init_block,
            None,
        ));
        self.factory().update_parameter_declaration(
            parameter,
            parameter.modifiers().cloned(),
            parameter.dot_dot_dot_token(),
            parameter.name().unwrap(),
            question_token(parameter),
            parameter.type_node(),
            None,
        )
    }

    pub fn add_default_value_assignment_if_needed(&mut self, parameter: &Arc<Node>) -> Arc<Node> {
        if parameter.dot_dot_dot_token().is_some() {
            return Arc::clone(parameter);
        }
        if parameter.name().is_some_and(|n| is_binding_pattern(n)) {
            return self.add_default_value_assignment_for_binding_pattern(parameter);
        }
        if let Some(initializer) = parameter.initializer() {
            return self
                .add_default_value_assignment_for_initializer(parameter, &parameter.name().unwrap(), &initializer);
        }
        Arc::clone(parameter)
    }

    pub fn add_default_value_assignments_if_needed(
        &mut self,
        node_list: Option<&NodeList>,
    ) -> NodeList {
        let Some(node_list) = node_list else {
            return NodeList::new(Vec::new());
        };
        let nodes = &node_list.nodes;
        let mut result: Option<Vec<Arc<Node>>> = None;
        for (i, parameter) in nodes.iter().enumerate() {
            let updated = self.add_default_value_assignment_if_needed(parameter);
            if !Arc::ptr_eq(&updated, parameter) {
                let result = result.get_or_insert_with(|| nodes.to_vec());
                result[i] = updated;
            }
        }
        if let Some(result) = result {
            let mut list = NodeList::new(result);
            list.loc = node_list.loc;
            return list;
        }
        let mut list = NodeList::new(nodes.to_vec());
        list.loc = node_list.loc;
        list
    }

    pub fn end_and_merge_lexical_environment(
        &mut self,
        statements: &[Arc<Node>],
    ) -> (Vec<Arc<Node>>, bool) {
        let declarations = self.end_lexical_environment();
        self.merge_environment_inner(statements, &declarations)
    }

    pub fn end_and_merge_variable_environment(
        &mut self,
        statements: &[Arc<Node>],
    ) -> (Vec<Arc<Node>>, bool) {
        let declarations = self.end_variable_environment();
        self.merge_environment_inner(statements, &declarations)
    }

    pub fn get_node_for_generated_name_worker(
        &self,
        node: &Arc<Node>,
        auto_generate_id: AutoGenerateId,
    ) -> Arc<Node> {
        let mut node = Arc::clone(node);
        let mut original = self.original(&node);
        while let Some(current_original) = original {
            node = current_original;
            if is_member_name(&node) {
                let Some(auto_generate) = self.get_auto_generate_info(&node) else {
                    break;
                };
                if auto_generate.flags.is_node() && auto_generate.id != auto_generate_id {
                    break;
                }
                if auto_generate.flags.is_node() {
                    original = auto_generate.node.clone();
                    continue;
                }
            }
            original = self.original(&node);
        }
        node
    }

    pub fn is_custom_prologue(&self, node: &Arc<Node>) -> bool {
        self.emit_flags(node).contains(EmitFlags::CUSTOM_PROLOGUE)
    }

    pub fn is_hoisted_function(&self, node: &Arc<Node>) -> bool {
        self.is_custom_prologue(node) && is_function_declaration(node)
    }

    pub fn is_hoisted_variable_statement(&self, node: &Arc<Node>) -> bool {
        self.is_custom_prologue(node)
            && is_variable_statement(node)
            && every(
                &node.as_variable_statement()
                    .declaration_list
                    .as_variable_declaration_list()
                    .declarations
                    .nodes,
                is_hoisted_variable,
            )
    }

    pub fn on_clone(&mut self, updated: &Arc<Node>, original: &Arc<Node>) {
        self.set_original(updated, original);
        if is_identifier(updated) || is_private_identifier(updated) {
            if let Some(auto_generate) = self.get_auto_generate_info(original) {
                let auto_generate_copy = auto_generate.clone();
                self.auto_generate
                    .insert(Arc::as_ptr(updated) as *const Node, auto_generate_copy);
            }
        }
    }

    pub fn on_create(&self, node: &mut Node) {
        node.flags.insert(NodeFlags::Synthesized);
    }

    pub fn on_update(&mut self, updated: &Arc<Node>, original: &Arc<Node>) {
        self.set_original(updated, original);
    }
}

pub fn is_hoisted_variable(node: &Arc<Node>) -> bool {
    node.name().is_some_and(|n| is_identifier(n)) && node.initializer().is_none()
}
