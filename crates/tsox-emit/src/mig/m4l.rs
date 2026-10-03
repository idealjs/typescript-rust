#![allow(unused_imports)]

use std::collections::HashMap;
use std::sync::Arc;

use tsox_core::collections::multimap::group_by;
use tsox_core::core::compiler_options_kinds::ScriptTarget;
use tsox_frontend::ast::node::Node;
use tsox_frontend::ast::node::{ModifierList, NodeList};
use tsox_frontend::ast::node_data_generated::{
    is_class_static_block_declaration, is_computed_property_name, is_decorator, is_identifier,
    is_private_identifier, is_property_access_expression, is_property_declaration,
};
use tsox_frontend::ast::node_flags::{ModifierFlags, NodeFlags};
use tsox_frontend::ast::mig::m3g_2::is_this_parameter;
use tsox_frontend::ast::mig::m3g_3::node_or_child_is_decorated;
use tsox_frontend::ast::subtree_facts::{
    SubtreeContainsDecorators, SubtreeContainsPrivateIdentifierInExpression,
};
use tsox_frontend::ast::syntax_kind_generated::SyntaxKind;
use tsox_frontend::ast::utilities::{has_static_modifier, is_static};
use tsox_frontend::ast::{can_have_decorators, deep_clone_node, for_each_child};
use tsox_frontend::ast::mig::m3b::members as node_members;
use tsox_frontend::format::mig::m4o::EmitFlags;
use tsox_frontend::format::mig::m4o::EF_NO_COMMENTS;
use tsox_frontend::scanner::{TokenFlags, TOKEN_FLAGS_NONE};

#[path = "r33k9_defs.rs"]
pub mod r33k9_defs;

use crate::mig::m4l_3::r39k10_defs::R39K10NodeFactoryExt;
use crate::mig::m4l::r39k20_defs::R39K20NodeFactoryExt;
use crate::mig::m4g::r33k7_defs::{has_decorators, ReferenceResolver};
use crate::mig::m4l_7::r38k3_defs::K3NodeAccessExt;
use crate::mig::m4m_2::is_simple_inlineable_expression;
use crate::mig::wt1b::{
    decorator_contains_private_identifier_in_expression, get_all_decorators_of_class_element,
    AllDecorators,
};
use crate::printer::{EmitContext, NodeFactory};
use tsox_frontend::ast::visitor::NodeVisitor;

#[path = "r39k20_defs.rs"]
pub mod r39k20_defs;

pub fn node_key(node: &Arc<Node>) -> usize { ::tsox_core::fntrace::enter("node_key"); 
    Arc::as_ptr(node) as usize
}

pub fn get_decorators_of_parameters(node: Option<&Arc<Node>>) -> Vec<Vec<Arc<Node>>> { ::tsox_core::fntrace::enter("get_decorators_of_parameters"); 
    let mut decorators: Vec<Vec<Arc<Node>>> = Vec::new();
    if let Some(node) = node {
        let parameters: &[Arc<Node>] = match node.parameters() {
            Some(list) => &list.nodes,
            None => &[],
        };
        let first_parameter_is_this =
            !parameters.is_empty() && is_this_parameter(&parameters[0]);
        let mut first_parameter_offset = 0;
        let mut num_parameters = parameters.len();
        if first_parameter_is_this {
            first_parameter_offset = 1;
            num_parameters -= 1;
        }
        for i in 0..num_parameters {
            let p = &parameters[i + first_parameter_offset];
            if !decorators.is_empty() || has_decorators(p) {
                if decorators.is_empty() {
                    decorators = vec![Vec::new(); num_parameters];
                }
                decorators[i] = p.decorators();
            }
        }
    }
    decorators
}

pub fn has_class_element_with_decorator_containing_private_identifier_in_expression(
    node: &Arc<Node>,
) -> bool { ::tsox_core::fntrace::enter("has_class_element_with_decorator_containing_private_identifier_in_expression"); 
    if node_members(node).is_empty() {
        return false;
    }
    for member in node_members(node).iter() {
        if !can_have_decorators(member) {
            continue;
        }
        let all_decorators = get_all_decorators_of_class_element(member, node, true);
        let Some(all_decorators) = all_decorators else {
            continue;
        };
        if all_decorators
            .decorators
            .iter()
            .any(decorator_contains_private_identifier_in_expression)
        {
            return true;
        }
        if all_decorators
            .parameters
            .iter()
            .any(|p| parameter_decorators_contain_private_identifier_in_expression(p))
        {
            return true;
        }
    }
    false
}

pub fn is_class_static_block_declaration_or_static_property(node: &Arc<Node>) -> bool { ::tsox_core::fntrace::enter("is_class_static_block_declaration_or_static_property"); 
    is_class_static_block_declaration(node) || (is_property_declaration(node) && has_static_modifier(node))
}

pub fn is_decorated_class_element(member: &Arc<Node>, is_static_element: bool, parent: &Arc<Node>) -> bool { ::tsox_core::fntrace::enter("is_decorated_class_element"); 
    is_static_element == is_static(member)
        && node_or_child_is_decorated(true, member, Some(parent), None)
}

pub fn is_not_export_default_or_decorator(node: &Arc<Node>) -> bool { ::tsox_core::fntrace::enter("is_not_export_default_or_decorator"); 
    !(is_decorator(node)
        || node.kind == SyntaxKind::ExportKeyword
        || node.kind == SyntaxKind::DefaultKeyword)
}

pub fn parameter_decorators_contain_private_identifier_in_expression(
    parameter_decorators: &[Arc<Node>],
) -> bool { ::tsox_core::fntrace::enter("parameter_decorators_contain_private_identifier_in_expression"); 
    parameter_decorators
        .iter()
        .any(decorator_contains_private_identifier_in_expression)
}

pub struct LegacyDecoratorsTransformer {
    pub language_version: ScriptTarget,
    pub reference_resolver: Arc<dyn ReferenceResolver>,
    pub class_aliases: HashMap<usize, Arc<Node>>,
    pub enclosing_classes: Vec<Arc<Node>>,
    pub emit_context: EmitContext,
    pub substitution_visitor: Option<NodeVisitor>,
}

impl LegacyDecoratorsTransformer {
    pub fn emit_context(&self) -> &EmitContext { ::tsox_core::fntrace::enter("emit_context"); 
        &self.emit_context
    }

    pub fn emit_context_mut(&mut self) -> &mut EmitContext { ::tsox_core::fntrace::enter("emit_context_mut"); 
        &mut self.emit_context
    }

    pub fn factory(&self) -> NodeFactory<'_> { ::tsox_core::fntrace::enter("factory"); 
        NodeFactory::new(&self.emit_context)
    }

    pub fn visitor(&mut self) -> &mut NodeVisitor { ::tsox_core::fntrace::enter("visitor"); 
        self.substitution_visitor.as_mut().unwrap()
    }

    pub fn get_expression_for_property_name(
        &mut self,
        member: &Arc<Node>,
        generate_name_for_computed_property_name: bool,
    ) -> Arc<Node> { ::tsox_core::fntrace::enter("get_expression_for_property_name"); 
        let name = member.name().unwrap();
        if is_private_identifier(&name) {
            self.factory().new_identifier("")
        } else if is_computed_property_name(&name) {
            let expression = match &name.data {
                tsox_frontend::ast::node_data_generated::NodeData::ComputedPropertyName(d) => {
                    Arc::clone(&d.expression)
                }
                _ => unreachable!(),
            };
            if generate_name_for_computed_property_name
                && !is_simple_inlineable_expression(&expression)
            {
                return self
                    .factory()
                    .generated_name_node(&self.factory().new_generated_name_for_node(&name));
            }
            expression
        } else if is_identifier(&name) {
            self.factory()
                .new_string_literal(name.text(), TOKEN_FLAGS_NONE)
        } else {
            deep_clone_node(&name)
        }
    }

    pub fn has_internal_static_reference(&self, node: &Arc<Node>) -> bool { ::tsox_core::fntrace::enter("has_internal_static_reference"); 
        let class_node = self.emit_context().most_original(node);

        fn is_or_contains_static_self_reference(
            tx: &LegacyDecoratorsTransformer,
            class_node: &Arc<Node>,
            n: &Arc<Node>,
        ) -> bool { ::tsox_core::fntrace::enter("is_or_contains_static_self_reference"); 
            if is_identifier(n)
                && tx
                    .reference_resolver
                    .get_referenced_value_declaration(&tx.emit_context().most_original(n))
                    .is_some_and(|d| {
                        Arc::ptr_eq(&tx.emit_context().most_original(&d), class_node)
                    })
            {
                return true;
            }
            if is_property_access_expression(n) {
                return n
                    .expression()
                    .is_some_and(|e| is_or_contains_static_self_reference(tx, class_node, e));
            }
            let mut result = false;
            for_each_child(n, |child| {
                if is_or_contains_static_self_reference(tx, class_node, child) {
                    result = true;
                    true
                } else {
                    false
                }
            });
            result
        }

        for member in node_members(node).iter() {
            let mut found = false;
            for_each_child(member, |child| {
                if is_or_contains_static_self_reference(self, &class_node, child) {
                    found = true;
                    true
                } else {
                    false
                }
            });
            if found {
                return true;
            }
        }
        false
    }

    pub fn is_synthetic_metadata_decorator(&self, node: &Arc<Node>) -> bool { ::tsox_core::fntrace::enter("is_synthetic_metadata_decorator"); 
        self.emit_context()
            .is_call_to_helper(node.expression().expect("decorator expression"), "__metadata")
    }

    pub fn pop_enclosing_class(&mut self) { ::tsox_core::fntrace::enter("pop_enclosing_class"); 
        self.enclosing_classes.pop();
    }

    pub fn push_enclosing_class(&mut self, cls: &Arc<Node>) { ::tsox_core::fntrace::enter("push_enclosing_class"); 
        self.enclosing_classes.push(Arc::clone(cls));
    }

    pub fn transform_all_decorators_of_declaration(
        &mut self,
        all_decorators: Option<&AllDecorators>,
    ) -> Vec<Arc<Node>> { ::tsox_core::fntrace::enter("transform_all_decorators_of_declaration"); 
        let Some(all_decorators) = all_decorators else {
            return Vec::new();
        };

        let mut metadata: Vec<Arc<Node>> = Vec::new();
        let mut decorators: Vec<Arc<Node>> = Vec::new();
        for d in &all_decorators.decorators {
            if self.is_synthetic_metadata_decorator(d) {
                metadata.push(Arc::clone(d));
            } else {
                decorators.push(Arc::clone(d));
            }
        }

        let mut decorator_expressions = Vec::new();
        decorator_expressions.extend(self.transform_decorators(&decorators));
        decorator_expressions.extend(
            self.transform_decorators_of_parameters(&all_decorators.parameters),
        );
        decorator_expressions.extend(self.transform_decorators(&metadata));
        decorator_expressions
    }

    pub fn transform_decorators_of_class_elements(
        &mut self,
        node: &Arc<Node>,
        members: NodeList,
    ) -> (NodeList, Vec<Arc<Node>>) { ::tsox_core::fntrace::enter("transform_decorators_of_class_elements"); 
        let mut decoration_statements = Vec::new();
        decoration_statements.extend(self.get_class_element_decoration_statements(node, false));
        decoration_statements.extend(self.get_class_element_decoration_statements(node, true));
        if has_class_element_with_decorator_containing_private_identifier_in_expression(node) {
            let mut member_nodes: Vec<Arc<Node>> = Vec::new();
            member_nodes.extend(members.nodes.iter().cloned());
            member_nodes.push(self.factory().new_class_static_block_declaration(
                None,
                self.factory().new_block(
                    &self.factory().new_node_list(decoration_statements),
                    true,
                ),
            ));
            let new_members = self.factory().new_node_list(member_nodes);
            let mut members = NodeList::new(new_members.nodes.clone());
            members.loc = new_members.loc;
            return (members, Vec::new());
        }

        (members, decoration_statements)
    }

    pub fn transform_decorators_of_parameters(
        &mut self,
        parameters: &[Vec<Arc<Node>>],
    ) -> Vec<Arc<Node>> { ::tsox_core::fntrace::enter("transform_decorators_of_parameters"); 
        let mut results = Vec::new();
        for (i, decorators) in parameters.iter().enumerate() {
            if !decorators.is_empty() {
                for decorator in decorators {
                    let Some(decorator_expression) = decorator.expression() else {
                        continue;
                    };
                    let visited_expression = self.visitor().visit_node(decorator_expression);
                    let mut helper = self
                        .factory()
                        .new_param_helper(&visited_expression, i, decorator_expression.loc);
                    self.emit_context_mut()
                        .set_emit_flags(&helper, EF_NO_COMMENTS);
                    results.push(helper);
                }
            }
        }
        results
    }
}