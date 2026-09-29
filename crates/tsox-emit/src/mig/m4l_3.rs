#![allow(unused_imports)]

use std::collections::HashMap;
use std::sync::Arc;

use tsox_frontend::ast::mig::m3c::visit_each_child;
use tsox_frontend::ast::node::{ModifierList, Node, NodeList};
use tsox_frontend::ast::node_data_generated::NodeData;
use tsox_frontend::ast::node_flags::{ModifierFlags, NodeFlags};
use tsox_frontend::ast::subtree_facts::SubtreeContainsDecorators;
use tsox_frontend::ast::syntax_kind_generated::SyntaxKind;
use tsox_frontend::ast::visitor::NodeVisitor;

use tsox_checker::checker::mig::wc1b::class_or_constructor_parameter_is_decorated;
use tsox_frontend::ast::node_data_generated::is_computed_property_name;
use tsox_frontend::ast::utilities::skip_partially_emitted_expressions_arc;
use tsox_frontend::format::mig::m4o::EmitFlags;

use super::m4l::r33k9_defs::child_is_decorated;
use super::m4l::r39k20_defs::R39K20NodeFactoryExt;
use super::m4l::{is_not_export_default_or_decorator, node_key, LegacyDecoratorsTransformer};
use crate::mig::m4g::r33k7_defs::has_decorators;
use crate::mig::m4g::r39k15_defs::NodeFactoryR39k15;
use crate::mig::m4l_7::r38k3_defs::K3NodeAccessExt;
use crate::mig::m4m_2::is_simple_inlineable_expression;
use crate::mig::m4m_4::move_range_past_modifiers;
use self::r39k10_defs::{
    r39k10_elide_modifiers, R39K10LegacyDecoratorsExt, R39K10NodeExt, R39K10NodeFactoryExt,
};

#[path = "r39k10_defs.rs"]
pub mod r39k10_defs;

trait LegacyDecoratorsVisitorExt {
    fn visit_each_child(&mut self, node: &Arc<Node>) -> Arc<Node>;
    fn visit_nodes(&mut self, nodes: Option<&NodeList>) -> NodeList;
    fn visit_modifiers(
        &mut self,
        modifiers: Option<&Arc<ModifierList>>,
    ) -> Option<Arc<ModifierList>>;
}

fn m3c_visitor() -> tsox_frontend::ast::mig::m3c::NodeVisitor {
    tsox_frontend::ast::mig::m3c::NodeVisitor {
        factory: tsox_frontend::ast::mig::m3c::NodeFactory {
            hooks: tsox_frontend::ast::mig::m3c::NodeFactoryHooks::default(),
            text_count: 0,
            node_count: 0,
        },
    }
}

impl LegacyDecoratorsVisitorExt for NodeVisitor {
    fn visit_each_child(&mut self, node: &Arc<Node>) -> Arc<Node> {
        visit_each_child(node, &mut m3c_visitor())
    }

    fn visit_nodes(&mut self, nodes: Option<&NodeList>) -> NodeList {
        match nodes {
            Some(list) => {
                let visited = list.nodes.iter().map(|n| self.visit_node(n)).collect();
                let mut new_list = NodeList::new(visited);
                new_list.loc = list.loc;
                new_list
            }
            None => NodeList::new(Vec::new()),
        }
    }

    fn visit_modifiers(
        &mut self,
        modifiers: Option<&Arc<ModifierList>>,
    ) -> Option<Arc<ModifierList>> {
        modifiers.map(|m| {
            let visited = m.list.nodes.iter().map(|n| self.visit_node(n)).collect();
            let mut new_list = NodeList::new(visited);
            new_list.loc = m.list.loc;
            Arc::new(ModifierList {
                list: new_list,
                modifier_flags: m.modifier_flags,
            })
        })
    }
}

impl LegacyDecoratorsTransformer {

    pub fn visit(&mut self, node: &Arc<Node>) -> Option<Arc<Node>> {
        if !node
            .subtree_facts()
            .intersects(SubtreeContainsDecorators)
            && self.enclosing_classes.is_empty()
        {
            return Some(Arc::clone(node));
        }

        match node.kind {
            SyntaxKind::Identifier => self.visit_identifier(node),
            SyntaxKind::PropertyAccessExpression => self.visit_property_access_expression(node),
            SyntaxKind::Decorator => None,
            SyntaxKind::ClassDeclaration => self.visit_class_declaration(node),
            SyntaxKind::ClassExpression => self.visit_class_expression(node),
            SyntaxKind::Constructor => self.visit_constructor_declaration(node),
            SyntaxKind::MethodDeclaration => self.visit_method_declaration(node),
            SyntaxKind::SetAccessor => self.visit_set_accessor_declaration(node),
            SyntaxKind::GetAccessor => self.visit_get_accessor_declaration(node),
            SyntaxKind::PropertyDeclaration => self.visit_property_declaration(node),
            SyntaxKind::Parameter => self.visit_paramer_declaration(node),
            SyntaxKind::SourceFile => {
                self.class_aliases = HashMap::new();
                self.enclosing_classes.clear();
                let result = self.visitor().visit_each_child(node);
                let helpers = self.emit_context_mut().read_emit_helpers();
                self.emit_context_mut().add_emit_helper(&result, &helpers);
                self.class_aliases.clear();
                self.enclosing_classes.clear();
                Some(result)
            }
            _ => Some(self.visitor().visit_each_child(node)),
        }
    }

    pub fn visit_class_declaration(&mut self, node: &Arc<Node>) -> Option<Arc<Node>> {
        let decorated = class_or_constructor_parameter_is_decorated(true, node);
        if !(decorated || child_is_decorated(true, node, None)) {
            return Some(self.visitor().visit_each_child(node));
        }

        if decorated {
            return self.transform_class_declaration_with_class_decorators(node, node.name().cloned());
        }
        self.transform_class_declaration_without_class_decorators(node, node.name().cloned())
    }

    pub fn visit_class_expression(&mut self, node: &Arc<Node>) -> Option<Arc<Node>> {
        let heritage_clauses = self
            .visitor()
            .visit_nodes(node.heritage_clauses().map(|l| l.as_ref()));
        let members = self.visitor().visit_nodes(node.members().map(|l| l.as_ref()));
        let modifiers = self.visitor().visit_modifiers(node.modifiers());
        Some(self.factory().update_class_expression(
            node,
            modifiers,
            node.name(),
            None,
            Some(&heritage_clauses),
            &members,
        ))
    }

    pub fn visit_constructor_declaration(&mut self, node: &Arc<Node>) -> Option<Arc<Node>> {
        let parameters = self
            .visitor()
            .visit_nodes(node.parameters().map(|l| l.as_ref()));
        let body = self.visitor().visit_node_opt(node.body());
        let modifiers = self.visitor().visit_modifiers(node.modifiers());
        Some(self.factory().update_constructor_declaration(
            node,
            modifiers,
            None,
            &parameters,
            None,
            None,
            body.as_ref(),
        ))
    }

    pub fn visit_get_accessor_declaration(&mut self, node: &Arc<Node>) -> Option<Arc<Node>> {
        let name = self.visit_property_name_of_class_element(node);
        let parameters = self
            .visitor()
            .visit_nodes(node.parameters().map(|l| l.as_ref()));
        let body = self.visitor().visit_node_opt(node.body());
        let modifiers = self.visitor().visit_modifiers(node.modifiers());
        let updated = self.factory().update_get_accessor_declaration(
            node,
            modifiers,
            &name,
            None,
            &parameters,
            None,
            None,
            body.as_ref(),
        );
        Some(self.finish_class_element(&updated, node))
    }

    pub fn visit_identifier(&mut self, node: &Arc<Node>) -> Option<Arc<Node>> {
        for d in &self.enclosing_classes {
            if let Some(alias) = self.class_aliases.get(&node_key(d)) {
                let matches = self
                    .reference_resolver
                    .get_referenced_value_declaration(&self.emit_context().most_original(node))
                    .is_some_and(|decl| {
                        Arc::ptr_eq(
                            &self.emit_context().most_original(&decl),
                            &self.emit_context().most_original(d),
                        )
                    });
                if matches {
                    return Some(Arc::clone(alias));
                }
            }
        }
        Some(Arc::clone(node))
    }

    pub fn visit_method_declaration(&mut self, node: &Arc<Node>) -> Option<Arc<Node>> {
        let name = self.visit_property_name_of_class_element(node);
        let parameters = self
            .visitor()
            .visit_nodes(node.parameters().map(|l| l.as_ref()));
        let body = self.visitor().visit_node_opt(node.body());
        let modifiers = self.visitor().visit_modifiers(node.modifiers());
        let updated = self.factory().update_method_declaration(
            node,
            modifiers,
            node.asterisk_token(),
            &name,
            None,
            None,
            &parameters,
            None,
            None,
            body.as_ref(),
        );
        Some(self.finish_class_element(&updated, node))
    }

    pub fn visit_paramer_declaration(&mut self, node: &Arc<Node>) -> Option<Arc<Node>> {
        let name = self.visitor().visit_node(node.name().unwrap());
        let initializer = self.visitor().visit_node_opt(node.initializer());
        let mut updated = self.factory().r39k20_update_parameter_declaration(
            node,
            r39k10_elide_modifiers(&self.factory(), node.modifiers()),
            node.dot_dot_dot_token(),
            &name,
            None,
            None,
            initializer.as_ref(),
        );
        if !Arc::ptr_eq(&updated, node) {
            self.emit_context_mut().set_comment_range(&updated, node.loc);
            let new_loc = move_range_past_modifiers(node);
            if let Some(updated_node) = Arc::get_mut(&mut updated) {
                updated_node.loc = new_loc;
            }
            self.emit_context_mut()
                .set_source_map_range(&updated, new_loc);
            self.emit_context_mut().set_emit_flags(
                &updated.name().unwrap(),
                EmitFlags::NO_TRAILING_SOURCE_MAP,
            );
        }
        Some(updated)
    }

    pub fn visit_property_access_expression(&mut self, node: &Arc<Node>) -> Option<Arc<Node>> {
        let flags = node.flags;
        let (expression, question_dot_token, name) = match &node.data {
            tsox_frontend::ast::node_data_generated::NodeData::PropertyAccessExpression(d) => (
                Arc::clone(&d.expression),
                d.question_dot_token.clone(),
                Arc::clone(&d.name),
            ),
            _ => unreachable!(),
        };
        let visited = self.visitor().visit_node(&expression);
        if !Arc::ptr_eq(&visited, &expression) {
            return Some(self.factory().update_property_access_expression(
                node,
                &visited,
                question_dot_token.as_ref(),
                &name,
                flags,
            ));
        }
        Some(Arc::clone(node))
    }

    pub fn visit_property_declaration(&mut self, node: &Arc<Node>) -> Option<Arc<Node>> {
        if node.flags.contains(NodeFlags::Ambient) {
            return None;
        }
        if node.has_syntactic_modifier(ModifierFlags::Ambient.union(ModifierFlags::Abstract)) {
            return None;
        }

        let name = self.visit_property_name_of_class_element(node);
        let initializer = self.visitor().visit_node_opt(node.initializer());
        let modifiers = self.visitor().visit_modifiers(node.modifiers());
        let updated = self.factory().update_property_declaration(
            node,
            modifiers,
            &name,
            None,
            None,
            initializer,
        );
        Some(self.finish_class_element(&updated, node))
    }

    pub fn visit_property_name_of_class_element(&mut self, member: &Arc<Node>) -> Arc<Node> {
        let name = member.name().unwrap();
        if is_computed_property_name(&name) && has_decorators(member) {
            if let NodeData::ComputedPropertyName(d) = &name.data {
                let expression = self.visitor().visit_node(&d.expression);
                let inner_expression = skip_partially_emitted_expressions_arc(&expression);
                if !is_simple_inlineable_expression(&inner_expression) {
                    let generated_name = self.factory().generated_name_node(
                        &self.factory().new_generated_name_for_node(&name),
                    );
                    self.emit_context_mut().add_variable_declaration(&generated_name);
                    return self.factory().update_computed_property_name(
                        &name,
                        &self
                            .factory()
                            .new_assignment_expression(&generated_name, &expression),
                    );
                }
            }
        }
        self.visitor().visit_node(&name)
    }

    pub fn visit_set_accessor_declaration(&mut self, node: &Arc<Node>) -> Option<Arc<Node>> {
        let name = self.visit_property_name_of_class_element(node);
        let parameters = self
            .visitor()
            .visit_nodes(node.parameters().map(|l| l.as_ref()));
        let body = self.visitor().visit_node_opt(node.body());
        let modifiers = self.visitor().visit_modifiers(node.modifiers());
        let updated = self.factory().update_set_accessor_declaration(
            node,
            modifiers,
            &name,
            None,
            &parameters,
            None,
            None,
            body.as_ref(),
        );
        Some(self.finish_class_element(&updated, node))
    }
}
