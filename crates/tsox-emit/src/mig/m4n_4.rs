#![allow(unused_imports)]
#![allow(dead_code)]

use std::sync::Arc;

use tsox_core::core::compiler_options_kinds::ScriptTarget;
use tsox_frontend::ast::deep_clone_node;
use tsox_frontend::ast::node::Node;
use tsox_frontend::ast::node_data_generated::{
    is_identifier, is_qualified_name, is_variable_declaration_list, NodeData,
    VariableDeclarationData,
};
use tsox_frontend::ast::node_flags::NodeFlags;
use tsox_frontend::ast::syntax_kind_generated::SyntaxKind;
use tsox_frontend::ast::{has_syntactic_modifier, ModifierFlags};
use tsox_frontend::ast::is_prologue_directive;
use tsox_frontend::scanner::TokenFlags;

use crate::printer::EmitContext;
use crate::mig::m4j::r36k3_defs::R36K3NodeFactoryExt;
use crate::mig::m4n::r37k19_defs::R37K19ArcNodeExt;
use crate::mig::m4m::r36k5_defs::NodeDataExt;
use crate::mig::m4i_12::r36k17_defs::NodeAsR36k17Ext;
use super::m4q::r33k12_defs::{EmitFlags, NodeFactory, flatten_comma_elements};

#[derive(Debug, Clone, Copy, Default)]
pub struct NameOptions {
    pub allow_comments: bool,
    pub allow_source_maps: bool,
}

#[derive(Debug, Clone, Copy, Default)]
pub struct AssignedNameOptions {
    pub allow_comments: bool,
    pub allow_source_maps: bool,
    pub ignore_assigned_name: bool,
}

impl<'a> NodeFactory<'a> {
    pub fn create_expression_from_entity_name(&self, node: &Arc<Node>) -> Arc<Node> {
        if is_qualified_name(node) {
            let left = self.create_expression_from_entity_name(&node.as_qualified_name().left);
            let mut right = deep_clone_node(&node.as_qualified_name().right);
            right.set_loc(node.as_qualified_name().right.loc);
            if let Some(parent) = node.as_qualified_name().right.parent() {
                right.set_parent(&parent);
            }
            let mut prop_access = self.new_property_access_expression(
                &left,
                None,
                &right,
                NodeFlags::empty(),
            );
            prop_access.set_loc(node.loc);
            return prop_access;
        }
        let mut res = deep_clone_node(node);
        res.set_loc(node.loc);
        if let Some(parent) = node.parent() {
            res.set_parent(&parent);
        }
        res
    }

    pub fn create_for_of_binding_statement(
        &self,
        node: &Arc<Node>,
        bound_value: &Arc<Node>,
    ) -> Arc<Node> {
        if is_variable_declaration_list(node) {
            let first_declaration =
                Arc::clone(&node.as_variable_declaration_list().declarations.nodes[0]);
            let mut updated_declaration = Node::new(
                SyntaxKind::VariableDeclaration,
                NodeData::VariableDeclaration(VariableDeclarationData {
                    name: Arc::clone(
                        &first_declaration
                            .name()
                            .expect("variable declaration requires a name"),
                    ),
                    exclamation_token: None,
                    type_node: None,
                    initializer: Some(Arc::clone(bound_value)),
                }),
            );
            updated_declaration.loc = first_declaration.loc;
            updated_declaration.flags = first_declaration.flags;
            let updated_declaration = Arc::new(updated_declaration);
            let mut statement = self.new_variable_statement(
                None,
                &self.update_variable_declaration_list(
                    node,
                    &self.new_node_list(vec![updated_declaration]),
                    node.flags,
                ),
            );
            statement.set_loc(node.loc);
            return statement;
        }
        let mut updated_expression = self.new_assignment_expression(node, bound_value);
        updated_expression.set_loc(node.loc);
        let mut statement = self.new_expression_statement(&updated_expression);
        statement.set_loc(node.loc);
        statement
    }

    pub fn ensure_use_strict(&self, statements: Vec<Arc<Node>>) -> Vec<Arc<Node>> {
        if let Some(first) = statements.first() {
            if is_prologue_directive(first)
                && first.expression().map(|e| e.text()).as_deref() == Some("use strict")
            {
                return statements;
            }
        }
        let use_strict_prologue =
            self.new_expression_statement(&self.new_string_literal("use strict", 0));
        let mut result = vec![use_strict_prologue];
        result.extend(statements);
        result
    }

    pub fn get_declaration_name(&self, node: &Arc<Node>) -> Arc<Node> {
        self.get_declaration_name_ex(node, NameOptions::default())
    }

    pub fn get_declaration_name_ex(&self, node: &Arc<Node>, opts: NameOptions) -> Arc<Node> {
        self.get_name(
            node,
            EmitFlags::NONE,
            AssignedNameOptions {
                allow_comments: opts.allow_comments,
                allow_source_maps: opts.allow_source_maps,
                ignore_assigned_name: false,
            },
        )
    }

    pub fn get_export_name(&self, node: &Arc<Node>) -> Arc<Node> {
        self.get_export_name_ex(node, AssignedNameOptions::default())
    }

    pub fn get_export_name_ex(&self, node: &Arc<Node>, opts: AssignedNameOptions) -> Arc<Node> {
        self.get_name(node, EmitFlags::EXPORT_NAME, opts)
    }

    pub fn get_external_module_or_namespace_export_name(
        &self,
        ns: Option<&Arc<Node>>,
        node: &Arc<Node>,
        allow_comments: bool,
        allow_source_maps: bool,
    ) -> Arc<Node> {
        if let Some(ns) = ns {
            if has_syntactic_modifier(node, ModifierFlags::Export) {
                let name_opts = NameOptions {
                    allow_comments,
                    allow_source_maps,
                };
                return self.get_namespace_member_name(
                    ns,
                    &self.get_declaration_name_ex(node, name_opts),
                    name_opts,
                );
            }
        }
        self.get_export_name_ex(
            node,
            AssignedNameOptions {
                allow_comments,
                allow_source_maps,
                ignore_assigned_name: false,
            },
        )
    }

    pub fn get_local_name(&self, node: &Arc<Node>) -> Arc<Node> {
        self.get_local_name_ex(node, AssignedNameOptions::default())
    }

    pub fn get_local_name_ex(&self, node: &Arc<Node>, opts: AssignedNameOptions) -> Arc<Node> {
        self.get_name(node, EmitFlags::LOCAL_NAME, opts)
    }

    pub fn get_namespace_member_name(
        &self,
        ns: &Arc<Node>,
        name: &Arc<Node>,
        opts: NameOptions,
    ) -> Arc<Node> {
        let name = if !(self.emit_context).has_auto_generate_info(name) {
            deep_clone_node(name)
        } else {
            Arc::clone(name)
        };
        let qualified_name =
            self.new_property_access_expression(ns, None, &name, NodeFlags::empty());
        self.emit_context_mut()
            .assign_comment_and_source_map_ranges(&qualified_name, &name);
        if !opts.allow_comments {
            self.emit_context_mut()
                .add_emit_flags(&qualified_name, EmitFlags::NO_COMMENTS);
        }
        if !opts.allow_source_maps {
            self.emit_context_mut()
                .add_emit_flags(&qualified_name, EmitFlags::NO_SOURCE_MAP);
        }
        qualified_name
    }

    pub fn inline_expressions(&self, expressions: Vec<Arc<Node>>) -> Option<Arc<Node>> {
        if expressions.is_empty() {
            return None;
        }
        if expressions.len() == 1 {
            return expressions.into_iter().next();
        }
        let mut expressions = flatten_comma_elements(&expressions);
        let mut iter = expressions.iter_mut();
        let mut expression = Arc::clone(iter.next().unwrap());
        for next in iter {
            let next = Arc::clone(next);
            expression = self.new_comma_expression(&expression, &next);
        }
        Some(expression)
    }

    pub fn new_array_slice_call(&self, array: &Arc<Node>, start: i32) -> Arc<Node> {
        let mut args = Vec::new();
        if start != 0 {
            args.push(self.new_numeric_literal(&start.to_string(), 0));
        }
        self.new_method_call(array, &self.new_identifier("slice"), args)
    }

    pub fn new_assign_helper(
        &self,
        attributes_segments: Vec<Arc<Node>>,
        script_target: ScriptTarget,
    ) -> Arc<Node> {
        self.new_call_expression(
            &self.new_property_access_expression(
                &self.new_identifier("Object"),
                None,
                &self.new_identifier("assign"),
                NodeFlags::empty(),
            ),
            None,
            None,
            self.new_node_list(attributes_segments),
            NodeFlags::empty(),
        )
    }

    pub fn new_assignment_expression(
        &self,
        left: &Arc<Node>,
        right: &Arc<Node>,
    ) -> Arc<Node> {
        self.new_binary_expression(
            None,
            left,
            None,
            &self.new_token(SyntaxKind::EqualsToken),
            right,
        )
    }

    pub fn new_comma_expression(&self, left: &Arc<Node>, right: &Arc<Node>) -> Arc<Node> {
        self.new_binary_expression(
            None,
            left,
            None,
            &self.new_token(SyntaxKind::CommaToken),
            right,
        )
    }

    pub fn new_false_expression(&self) -> Arc<Node> {
        self.new_keyword_expression(SyntaxKind::FalseKeyword)
    }
}
