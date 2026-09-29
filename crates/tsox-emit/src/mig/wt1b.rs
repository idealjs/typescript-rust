#![allow(unused_imports)]
#![allow(dead_code)]

use std::collections::HashMap;
use std::sync::Arc;

use tsox_frontend::ast::mig::w3::get_all_accessor_declarations;
use tsox_frontend::ast::mig::x4ast::get_first_constructor_with_body;
use tsox_frontend::ast::node::Node;
use tsox_frontend::ast::node_data_generated::is_property_declaration;
use tsox_frontend::ast::node_flags::{ModifierFlags, NodeFlags};
use tsox_frontend::ast::subtree_facts::SubtreeContainsPrivateIdentifierInExpression;
use tsox_frontend::ast::syntax_kind_generated::SyntaxKind;
use tsox_frontend::ast::utilities::{has_accessor_modifier, has_syntactic_modifier, is_static};
use tsox_frontend::ast::{ModifierList, NodeList};
use tsox_frontend::format::mig::m4o::EmitFlags;

use crate::printer::NodeFactory;

#[path = "r39k05_defs.rs"]
pub mod r39k05_defs;

use super::m4g::r33k7_defs::has_decorators;
use super::m4l::{
    LegacyDecoratorsTransformer, get_decorators_of_parameters, is_decorated_class_element,
};
use super::m4m_2::{is_generated_identifier, is_simple_inlineable_expression};
use super::m4m_4::move_range_past_modifiers;
use tsox_frontend::ast::mig::m3b::members as node_members;

fn node_key(node: &Arc<Node>) -> usize {
    Arc::as_ptr(node) as usize
}

fn decorators_of(node: &Arc<Node>) -> Vec<Arc<Node>> {
    node.modifiers()
        .map(|m| {
            m.list
                .nodes
                .iter()
                .filter(|n| n.kind == SyntaxKind::Decorator)
                .cloned()
                .collect()
        })
        .unwrap_or_default()
}

pub struct AllDecorators {
    pub decorators: Vec<Arc<Node>>,
    pub parameters: Vec<Vec<Arc<Node>>>,
}

struct LegacyDecoratorsReferenceResolver {
    inner: Arc<dyn tsox_checker::binder::referenceresolver::ReferenceResolver>,
}

impl super::m4g::r33k7_defs::ReferenceResolver for LegacyDecoratorsReferenceResolver {
    fn get_referenced_export_container(
        &self,
        node: &Arc<Node>,
        prefix_locals: bool,
    ) -> Option<Arc<Node>> {
        self.inner.get_referenced_export_container(node, prefix_locals)
    }

    fn get_referenced_import_declaration(&self, node: &Arc<Node>) -> Option<Arc<Node>> {
        self.inner.get_referenced_import_declaration(node)
    }

    fn get_referenced_value_declaration(&self, node: &Arc<Node>) -> Option<Arc<Node>> {
        self.inner.get_referenced_value_declaration(node)
    }

    fn get_referenced_value_declarations(&self, node: &Arc<Node>) -> Vec<Arc<Node>> {
        self.inner.get_referenced_value_declarations(node)
    }

    fn get_element_access_expression_name(&self, expression: &Arc<Node>) -> String {
        self.inner.get_element_access_expression_name(expression)
    }

    fn get_referenced_member_value_declaration(&self, node: &Arc<Node>) -> Option<Arc<Node>> {
        self.inner.get_referenced_member_value_declaration(node)
    }
}

pub fn new_legacy_decorators_transformer(
    opts: &super::m3m::TransformOptions,
) -> Option<Box<super::m3m::Transformer>> {
    let _tx = LegacyDecoratorsTransformer {
        language_version: opts.compiler_options.get_emit_script_target(),
        reference_resolver: Arc::new(LegacyDecoratorsReferenceResolver {
            inner: opts.resolver.clone(),
        }),
        class_aliases: HashMap::new(),
        enclosing_classes: Vec::new(),
        emit_context: opts.context.clone(),
        substitution_visitor: Some(tsox_frontend::ast::visitor::NodeVisitor::default()),
    };
    fn legacy_decorators_visit(_tx: &mut super::m3m::Transformer, node: Arc<Node>) -> Option<Arc<Node>> {
        Some(node)
    }
    Some(Box::new(super::m3m::Transformer::new(
        legacy_decorators_visit,
        Some(opts.context.clone()),
    )))
}

pub fn decorator_contains_private_identifier_in_expression(decorator: &Arc<Node>) -> bool {
    decorator
        .subtree_facts()
        .contains(SubtreeContainsPrivateIdentifierInExpression)
}

pub fn elide_nodes(f: &NodeFactory, nodes: Option<&NodeList>) -> Option<Arc<NodeList>> {
    let nodes = nodes?;
    let mut replacement = f.new_node_list(Vec::new());
    if let Some(r) = Arc::get_mut(&mut replacement) {
        r.loc = nodes.loc;
    }
    Some(replacement)
}

pub fn elide_modifiers(f: &NodeFactory, nodes: Option<&NodeList>) -> Option<Arc<ModifierList>> {
    let nodes = nodes?;
    let mut replacement = f.new_modifier_list(Vec::<Arc<Node>>::new());
    if let Some(r) = Arc::get_mut(&mut replacement) {
        r.list.loc = nodes.loc;
    }
    Some(replacement)
}

impl LegacyDecoratorsTransformer {
    pub fn get_class_alias_if_needed(&mut self, node: &Arc<Node>) -> Option<Arc<Node>> {
        if !self.has_internal_static_reference(node) {
            return None;
        }
        let name_node = node.name();
        let mut name_text = "default".to_string();
        if let Some(name) = name_node {
            if !is_generated_identifier(&self.emit_context(), &name) {
                name_text = name.text().to_string();
            }
        }
        let class_alias =
            self.factory()
                .generated_name_node(&self.factory().new_unique_name(&name_text));
        self.emit_context_mut().add_variable_declaration(&class_alias);
        self.class_aliases.insert(node_key(node), class_alias.clone());
        Some(class_alias)
    }

    pub fn get_constructor_decoration_statement(&mut self, node: &Arc<Node>) -> Option<Arc<Node>> {
        let expression = self.generate_constructor_decoration_expression(node)?;
        let result = self.factory().new_expression_statement(&expression);
        self.emit_context().set_original(&result, node);
        Some(result)
    }

    pub fn generate_constructor_decoration_expression(&mut self, node: &Arc<Node>) -> Option<Arc<Node>> {
        let all_decorators = get_all_decorators_of_class(node, true);
        let has_alias = self
            .enclosing_classes
            .last()
            .is_some_and(|c| Arc::ptr_eq(c, node));
        if has_alias {
            self.pop_enclosing_class();
        }
        let decorator_expressions = self.transform_all_decorators_of_declaration(all_decorators.as_ref());
        if has_alias {
            self.push_enclosing_class(node);
        }
        if decorator_expressions.is_empty() {
            return None;
        }

        let class_alias = self.class_aliases.get(&node_key(node)).cloned();
        let local_name = self.factory().get_declaration_name_ex(
            node,
            super::m4n_4::NameOptions {
                allow_comments: false,
                allow_source_maps: true,
            },
        );
        let decorate = self
            .factory()
            .new_decorate_helper(decorator_expressions, &local_name, None, None);
        let assignment_target = match class_alias {
            Some(alias) => self.factory().new_assignment_expression(&alias, &decorate),
            None => decorate,
        };
        let expression = self.factory().new_assignment_expression(&local_name, &assignment_target);
        self.emit_context_mut().set_emit_flags(&expression, EmitFlags::NO_COMMENTS);
        self.emit_context_mut()
            .set_source_map_range(&expression, move_range_past_modifiers(node));
        Some(expression)
    }

    pub fn get_class_element_decoration_statements(
        &mut self,
        node: &Arc<Node>,
        is_static: bool,
    ) -> Vec<Arc<Node>> {
        let exprs = self.generate_class_element_decoration_expressions(node, is_static);
        exprs
            .into_iter()
            .map(|e| self.factory().new_expression_statement(&e))
            .collect()
    }

    pub fn generate_class_element_decoration_expressions(
        &mut self,
        node: &Arc<Node>,
        is_static: bool,
    ) -> Vec<Arc<Node>> {
        let members = get_decorated_class_elements(node, is_static);
        let mut expressions = Vec::new();
        for member in members {
            if let Some(expr) = self.generate_class_element_decoration_expression(node, &member) {
                expressions.push(expr);
            }
        }
        expressions
    }

    pub fn generate_class_element_decoration_expression(
        &mut self,
        node: &Arc<Node>,
        member: &Arc<Node>,
    ) -> Option<Arc<Node>> {
        let all_decorators = get_all_decorators_of_class_element(member, node, true);
        let decorator_expressions = self.transform_all_decorators_of_declaration(all_decorators.as_ref());
        if decorator_expressions.is_empty() {
            return None;
        }

        let prefix = self.get_class_member_prefix(node, member);
        let member_name = self.get_expression_for_property_name(member, !member.flags.contains(NodeFlags::Ambient));
        let descriptor = if is_property_declaration(member) && !has_accessor_modifier(member) {
            self.factory().new_void_zero_expression()
        } else {
            self.factory().new_keyword_expression(SyntaxKind::NullKeyword)
        };
        let helper = self
            .factory()
            .new_decorate_helper(decorator_expressions, &prefix, Some(&member_name), Some(&descriptor));
        self.emit_context_mut().set_emit_flags(&helper, EmitFlags::NO_COMMENTS);
        self.emit_context_mut()
            .set_source_map_range(&helper, move_range_past_modifiers(member));
        Some(helper)
    }

    pub fn get_class_member_prefix(&mut self, node: &Arc<Node>, member: &Arc<Node>) -> Arc<Node> {
        if is_static(member) {
            return self.factory().get_declaration_name(node);
        }
        self.get_class_prototype(node)
    }

    pub fn get_class_prototype(&self, node: &Arc<Node>) -> Arc<Node> {
        self.factory().new_property_access_expression(
            &self.factory().get_declaration_name(node),
            None,
            &self.factory().new_identifier("prototype"),
            NodeFlags::empty(),
        )
    }
}

pub fn get_all_decorators_of_class(node: &Arc<Node>, use_legacy_decorators: bool) -> Option<AllDecorators> {
    let decorators = decorators_of(node);
    let parameters = if use_legacy_decorators {
        get_first_constructor_with_body(node)
            .map(|ctor| get_decorators_of_parameters(Some(&ctor)))
            .unwrap_or_default()
    } else {
        Vec::new()
    };
    if decorators.is_empty() && parameters.is_empty() {
        return None;
    }
    Some(AllDecorators {
        decorators,
        parameters,
    })
}

pub fn get_all_decorators_of_class_element(
    member: &Arc<Node>,
    parent: &Arc<Node>,
    use_legacy_decorators: bool,
) -> Option<AllDecorators> {
    match member.kind {
        SyntaxKind::GetAccessor | SyntaxKind::SetAccessor => {
            if !use_legacy_decorators {
                return get_all_decorators_of_method(member, false);
            }
            get_all_decorators_of_accessors(member, parent, true)
        }
        SyntaxKind::MethodDeclaration => get_all_decorators_of_method(member, use_legacy_decorators),
        SyntaxKind::PropertyDeclaration => get_all_decorators_of_property(member),
        _ => None,
    }
}

pub fn get_all_decorators_of_accessors(
    accessor: &Arc<Node>,
    parent: &Arc<Node>,
    use_legacy_decorators: bool,
) -> Option<AllDecorators> {
    if accessor.body().is_none() {
        return None;
    }
    let decls = get_all_accessor_declarations(node_members(parent), accessor);
    let first_accessor_with_decorators = if decls
        .first_accessor
        .as_ref()
        .is_some_and(|first| has_decorators(first))
    {
        decls.first_accessor.clone()
    } else if decls
        .second_accessor
        .as_ref()
        .is_some_and(|second| has_decorators(second))
    {
        decls.second_accessor.clone()
    } else {
        None
    };

    let first_accessor_with_decorators = first_accessor_with_decorators?;
    if !Arc::ptr_eq(accessor, &first_accessor_with_decorators) {
        return None;
    }

    let decorators = decorators_of(&first_accessor_with_decorators);
    let parameters = if use_legacy_decorators {
        decls
            .set_accessor
            .as_ref()
            .map(|set| get_decorators_of_parameters(Some(set)))
            .unwrap_or_default()
    } else {
        Vec::new()
    };

    if decorators.is_empty() && parameters.is_empty() {
        return None;
    }

    Some(AllDecorators {
        decorators,
        parameters,
    })
}

pub fn get_all_decorators_of_property(property: &Arc<Node>) -> Option<AllDecorators> {
    let decorators = decorators_of(property);
    if decorators.is_empty() {
        return None;
    }
    Some(AllDecorators {
        decorators,
        parameters: Vec::new(),
    })
}

pub fn get_all_decorators_of_method(method: &Arc<Node>, use_legacy_decorators: bool) -> Option<AllDecorators> {
    if method.body().is_none() {
        return None;
    }
    let decorators = decorators_of(method);
    let parameters = if use_legacy_decorators {
        get_decorators_of_parameters(Some(method))
    } else {
        Vec::new()
    };
    if decorators.is_empty() && parameters.is_empty() {
        return None;
    }
    Some(AllDecorators {
        decorators,
        parameters,
    })
}

pub fn get_decorated_class_elements(node: &Arc<Node>, is_static: bool) -> Vec<Arc<Node>> {
    let members = node_members(node);
    if members.is_empty() {
        return Vec::new();
    }
    members
        .iter()
        .filter(|member| is_decorated_class_element(member, is_static, node))
        .cloned()
        .collect()
}
