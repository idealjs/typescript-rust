#![allow(unused_imports)]

use std::sync::Arc;

use tsox_core::core::compiler_options_kinds::ScriptTarget;
use tsox_frontend::ast::node::{ModifierList, Node, NodeList};
use tsox_frontend::ast::syntax_kind_generated::SyntaxKind;
use tsox_frontend::ast::{ModifierFlags, NodeFlags};

use super::m4l::{
    has_class_element_with_decorator_containing_private_identifier_in_expression,
    is_class_static_block_declaration_or_static_property, is_not_export_default_or_decorator,
    LegacyDecoratorsTransformer,
};
use super::m4l_3::r39k10_defs::{
    R39K10LegacyDecoratorsExt, R39K10NodeExt, R39K10NodeFactoryExt,
};
use crate::mig::m4f_2::r38k5_defs::R38K5NodeVisitorExt;
use crate::mig::m4l::r39k20_defs::R39K20NodeVisitorExt;
use crate::mig::m4l_7::r38k3_defs::K3NodeAccessExt;
use crate::mig::m4m_2::is_generated_identifier;
use crate::mig::m4m_4::move_range_past_modifiers;
use crate::mig::m4n_4::AssignedNameOptions;

impl LegacyDecoratorsTransformer {
    pub fn transform_class_declaration_with_class_decorators(
        &mut self,
        node: &Arc<Node>,
        name: Option<Arc<Node>>,
    ) -> Option<Arc<Node>> {
        let is_export = node.has_syntactic_modifier(ModifierFlags::Export);
        let is_default = node.has_syntactic_modifier(ModifierFlags::Default);
        let mut modifiers: Option<Arc<ModifierList>> = None;
        let modifier_list = node.modifiers();
        let modifier_total = modifier_list.map(|m| m.list.nodes.len()).unwrap_or(0);
        let modifier_nodes: Vec<Arc<Node>> = modifier_list
            .map(|m| m.list.nodes.as_slice())
            .unwrap_or(&[])
            .iter()
            .filter(|m| is_not_export_default_or_decorator(m))
            .cloned()
            .collect();
        if !modifier_nodes.is_empty() {
            if modifier_nodes.len() != modifier_total {
                let mut replacement = self.factory().new_modifier_list(modifier_nodes);
                if let Some(replacement_list) = Arc::get_mut(&mut replacement) {
                    replacement_list.list.loc = modifier_list.map(|m| m.list.loc).unwrap_or(node.loc);
                }
                modifiers = Some(replacement);
            } else {
                modifiers = modifier_list.map(|m| {
                    Arc::new(ModifierList {
                        list: NodeList {
                            nodes: m.list.nodes.clone(),
                            loc: m.list.loc,
                        },
                        modifier_flags: m.modifier_flags,
                    })
                });
            }
        }

        let location = move_range_past_modifiers(node);
        let class_alias = self.get_class_alias_if_needed(node);
        if class_alias.is_some() {
            self.push_enclosing_class(node);
        }

        let decl_name = self.factory().get_local_name_ex(
            node,
            AssignedNameOptions {
                allow_comments: false,
                allow_source_maps: true,
                ignore_assigned_name: false,
            },
        );

        let heritage_clauses = self.visitor().visit_nodes(node.heritage_clauses());
        let members = self
            .visitor()
            .visit_nodes(node.members())
            .unwrap_or_default();

        let (members, decoration_statements) =
            self.transform_decorators_of_class_elements(node, members);

        let assign_class_alias_in_static_block = self.language_version >= ScriptTarget::ES2022
            && class_alias.is_some()
            && !members.nodes.is_empty()
            && members
                .nodes
                .iter()
                .any(is_class_static_block_declaration_or_static_property);
        let members = if assign_class_alias_in_static_block {
            let static_block = self.factory().new_class_static_block_declaration(
                None,
                self.factory().new_block(
                    &self.factory().new_node_list(vec![self.factory().new_expression_statement(
                        &self.factory().new_assignment_expression(
                            &class_alias.clone().unwrap(),
                            &self.factory().new_keyword_expression(SyntaxKind::ThisKeyword),
                        ),
                    )]),
                    false,
                ),
            );
            let mut member_list: Vec<Arc<Node>> = vec![static_block];
            member_list.extend(members.nodes.iter().cloned());
            let mut new_list = NodeList::new(member_list);
            new_list.loc = members.loc;
            new_list
        } else {
            members
        };

        let mut expr_name = name.clone();
        if let Some(name) = &name {
            if is_generated_identifier(&*self.emit_context(), name) {
                expr_name = None;
            }
        }
        let mut class_expression = self.factory().new_class_expression(
            modifiers,
            expr_name,
            None,
            heritage_clauses.map(Arc::new),
            Arc::new(members),
        );

        self.emit_context().set_original(&class_expression, node);
        if let Some(class_expr) = Arc::get_mut(&mut class_expression) {
            class_expr.loc = location;
        }

        let var_initializer = if class_alias.is_some() && !assign_class_alias_in_static_block {
            self.factory()
                .new_assignment_expression(&class_alias.clone().unwrap(), &class_expression)
        } else {
            class_expression
        };
        let var_decl = self
            .factory()
            .new_variable_declaration(&decl_name, None, None, Some(&var_initializer));
        self.emit_context().set_original(&var_decl, node);

        let var_decl_list = self.factory().new_variable_declaration_list(
            &self.factory().new_node_list(vec![var_decl]),
            NodeFlags::Let,
        );
        let mut var_statement = self.factory().new_variable_statement(None, &var_decl_list);
        if let Some(statement) = Arc::get_mut(&mut var_statement) {
            statement.loc = location;
        }
        self.emit_context().set_original(&var_statement, node);
        self.emit_context_mut().set_comment_range(&var_statement, node.loc);

        let mut statements: Vec<Arc<Node>> = vec![var_statement];
        statements.extend(decoration_statements);
        if let Some(statement) = self.get_constructor_decoration_statement(node) {
            statements.push(statement);
        }

        if is_export {
            let export_statement = if is_default {
                self.factory().new_export_default(&decl_name)
            } else {
                self.factory()
                    .new_external_module_export(&self.factory().get_declaration_name(node))
            };
            statements.push(export_statement);
        }

        if class_alias.is_some() {
            self.pop_enclosing_class();
        }

        if statements.len() == 1 {
            return Some(statements.into_iter().next().unwrap());
        }
        Some(self.factory().new_syntax_list(statements))
    }

    pub fn transform_class_declaration_without_class_decorators(
        &mut self,
        node: &Arc<Node>,
        mut name: Option<Arc<Node>>,
    ) -> Option<Arc<Node>> {
        let modifiers = self.visitor().visit_modifiers(&node.modifiers().cloned());
        let heritage_clauses = self.visitor().visit_nodes(node.heritage_clauses());
        let initial_members = self
            .visitor()
            .visit_nodes(node.members())
            .unwrap_or_default();
        let (members, decoration_statements) =
            self.transform_decorators_of_class_elements(node, initial_members);

        if name.is_none() && !decoration_statements.is_empty() {
            name = Some(self.factory().generated_name_node(
                &self.factory().new_generated_name_for_node(node),
            ));
        }

        let updated = self.factory().update_class_declaration(
            node,
            modifiers,
            name.as_ref(),
            None,
            heritage_clauses.as_ref(),
            &members,
        );

        if decoration_statements.is_empty() {
            return Some(updated);
        }
        let mut statements = vec![updated];
        statements.extend(decoration_statements);
        Some(self.factory().new_syntax_list(statements))
    }

    pub fn transform_decorators(&mut self, decorators: &[Arc<Node>]) -> Vec<Arc<Node>> {
        let mut results = Vec::new();
        for d in decorators {
            results.push(self.visitor().visit_node(d.expression().unwrap()));
        }
        results
    }
}
