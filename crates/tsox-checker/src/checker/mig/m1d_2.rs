#![allow(unused_imports)]
#![allow(dead_code)]

pub(crate) use super::m1d::r21k7_defs::*;

use tsox_frontend::ast::{get_class_extends_heritage_element, get_name_of_declaration, is_computed_property_name, is_string_literal};
use crate::checker::utilities_is_optional_symbol::is_late_bound_name;
use tsox_core::core::mig::m3j_3::node_core_modules;
use crate::checker::mapper::prepend_type_mapping;
use super::m2c::r21k2_defs::get_class_like_declaration_of_symbol;

use crate::checker::checker::*;
use crate::checker::types::*;
use std::sync::Arc;
use tsox_frontend::ast::{Node, NodeData, Symbol, SyntaxKind};

pub(crate) struct M1dKeyBuilder {
    hasher: std::collections::hash_map::DefaultHasher,
}

impl M1dKeyBuilder {
    pub(crate) fn new() -> Self {
        Self {
            hasher: std::collections::hash_map::DefaultHasher::new(),
        }
    }
    pub(crate) fn write_byte(&mut self, b: u8) {
        std::hash::Hasher::write_u8(&mut self.hasher, b);
    }
    pub(crate) fn write_type(&mut self, t: &Arc<Type>) {
        std::hash::Hasher::write_u32(&mut self.hasher, t.id);
    }
    pub(crate) fn write_types(&mut self, types: &[Arc<Type>]) {
        std::hash::Hasher::write_u32(&mut self.hasher, types.len() as u32);
        for t in types {
            self.write_type(t);
        }
    }
    pub(crate) fn write_alias(&mut self, alias: &Option<Box<TypeAlias>>) {
        if let Some(alias) = alias {
            if let Some(symbol) = &alias.symbol {
                std::hash::Hasher::write_u64(&mut self.hasher, symbol.id());
            } else {
                std::hash::Hasher::write_u8(&mut self.hasher, 0);
            }
            self.write_types(&alias.type_arguments);
        } else {
            std::hash::Hasher::write_u8(&mut self.hasher, 0);
        }
    }
    pub(crate) fn hash(self) -> u64 {
        std::hash::Hasher::finish(&self.hasher)
    }
}

pub(crate) fn get_alias_key(alias: &Option<Box<TypeAlias>>) -> u64 {
    let mut b = M1dKeyBuilder::new();
    b.write_alias(alias);
    b.hash()
}

pub(crate) fn get_conditional_type_key(
    type_arguments: &[Arc<Type>],
    alias: &Option<Box<TypeAlias>>,
    for_constraint: bool,
) -> u64 {
    let mut b = M1dKeyBuilder::new();
    b.write_types(type_arguments);
    b.write_alias(alias);
    if for_constraint {
        b.write_byte(b'!');
    }
    b.hash()
}

pub(crate) fn get_adjusted_node_for_error(node: &Arc<Node>) -> Arc<Node> {
    if let Some(name) = get_name_of_declaration(node) {
        return name;
    }
    Arc::clone(node)
}

pub(crate) fn get_base_type_node_of_class(t: &Arc<Type>) -> Option<Arc<Node>> {
    if let Some(symbol) = &t.symbol {
        let decl = get_class_like_declaration_of_symbol(symbol);
        if let Some(decl) = decl {
            return get_class_extends_heritage_element(&decl);
        }
    }
    None
}

impl Checker {
    pub(crate) fn get_constraint_or_unknown_from_type_parameter(
        &mut self,
        t: &Arc<Type>,
    ) -> Arc<Type> {
        self.get_constraint_from_type_parameter(t)
    }

    pub(crate) fn get_effective_type_argument_at_index(
        &mut self,
        node: &Arc<Node>,
        type_parameters: &[Arc<Type>],
        index: usize,
    ) -> Arc<Type> {
        if let Some(args) = node.type_arguments() {
            if index < args.nodes.len() {
                return self.get_type_from_type_node(&args.nodes[index]);
            }
        }
        let effective = self.get_effective_type_arguments(node, type_parameters);
        Arc::clone(&effective[index])
    }

    pub(crate) fn get_constraint_from_indexed_access(&mut self, t: &Arc<Type>) -> Option<Arc<Type>> {
        let d = t.indexed_access_data()?;
        let object_type = d.object_type.as_ref()?;
        let index_type = d.index_type.as_ref()?;
        if self.is_mapped_type_generic_indexed_access(t) {
            return Some(self.substitute_indexed_mapped_type(object_type, index_type));
        }
        let index_constraint = self.get_simplified_type_or_constraint(index_type);
        if let Some(constraint) = index_constraint {
            if constraint.id != index_type.id {
                let indexed_access = self.get_indexed_access_type_or_undefined(
                    object_type,
                    &constraint,
                    d.access_flags,
                    None,
                    None,
                );
                if let Some(indexed_access) = indexed_access {
                    return Some(indexed_access);
                }
            }
        }
        let object_constraint = self.get_simplified_type_or_constraint(object_type);
        if let Some(constraint) = object_constraint {
            if constraint.id != object_type.id {
                return self.get_indexed_access_type_or_undefined(
                    &constraint,
                    index_type,
                    d.access_flags,
                    None,
                    None,
                );
            }
        }
        None
    }

    pub(crate) fn get_constraint_from_conditional_type(
        &mut self,
        t: &Arc<Type>,
    ) -> Option<Arc<Type>> {
        if let Some(constraint) = self.get_constraint_of_distributive_conditional_type(t) {
            return Some(constraint);
        }
        self.get_default_constraint_of_conditional_type(t)
    }

    pub(crate) fn get_constraint_of_distributive_conditional_type(
        &mut self,
        t: &Arc<Type>,
    ) -> Option<Arc<Type>> {
        if let Some(cached) = t.resolved_constraint_of_distributive() {
            if cached.id != self.no_constraint_type().id {
                return Some(cached);
            }
            return None;
        }
        let d = t.conditional_data()?;
        let root = d.root.as_ref()?;
        let check_type = d.check_type.as_ref()?;
        let mut resolved: Option<Arc<Type>> = None;
        if root.is_distributive && !self.is_restrictive_instantiation(t) {
            let mut constraint = self.get_simplified_type(check_type, false);
            if constraint.id == check_type.id {
                if let Some(c) = self.get_constraint_of_type(&constraint) {
                    constraint = c;
                }
            }
            if constraint.id != check_type.id {
                let mapper = Arc::new(prepend_type_mapping(
                    Arc::clone(check_type),
                    Arc::clone(&constraint),
                    d.mapper.as_deref(),
                ));
                let instantiated =
                    self.get_conditional_type_instantiation(t, Some(mapper.as_ref()), true, None);
                if !instantiated.flags.contains(TypeFlags::Never) {
                    resolved = Some(Arc::clone(&instantiated));
                }
            }
        }
        t.set_resolved_constraint_of_distributive(resolved.clone());
        resolved
    }

    pub(crate) fn get_base_type_variable_of_class(&mut self, symbol: &Arc<Symbol>) -> Option<Arc<Type>> {
        let class_type = self.get_declared_type_of_class_or_interface(symbol);
        let base_constructor_type = self.get_base_constructor_type_of_class(&class_type)?;
        if base_constructor_type.flags.contains(TYPE_FLAGS_TYPE_VARIABLE) {
            return Some(base_constructor_type);
        }
        if base_constructor_type.flags.contains(TypeFlags::Intersection) {
            return base_constructor_type
                .types()
                .and_then(|ts| {
                    ts.iter()
                        .find(|t| t.flags.contains(TYPE_FLAGS_TYPE_VARIABLE))
                })
                .cloned();
        }
        None
    }

    pub(crate) fn get_declared_type_of_class_or_interface(
        &mut self,
        symbol: &Arc<Symbol>,
    ) -> Arc<Type> {
        let links = self.declared_type_links.get(symbol);
        if let Some(links) = links {
            if let Some(t) = &links.declared_type {
                return Arc::clone(t);
            }
        }
        self.create_declared_type_of_class_or_interface(symbol)
    }

    pub(crate) fn get_declaration_node_flags_from_symbol(
        &mut self,
        s: &Arc<Symbol>,
    ) -> tsox_frontend::ast::NodeFlags {
        if let Some(value_declaration) = &s.value_declaration {
            return self.get_combined_node_flags_cached(value_declaration);
        }
        tsox_frontend::ast::NodeFlags::empty()
    }

    pub(crate) fn get_combined_node_flags_cached(
        &mut self,
        node: &Arc<Node>,
    ) -> tsox_frontend::ast::NodeFlags {
        self.get_combined_node_flags(node)
    }

    pub(crate) fn get_combined_modifier_flags_cached(
        &mut self,
        node: &Arc<Node>,
    ) -> tsox_frontend::ast::ModifierFlags {
        if self
            .last_combined_modifier_flags_node
            .as_ref()
            .is_some_and(|n| n.id() == node.id())
        {
            return self.last_combined_modifier_flags_result;
        }
        let result = self.get_combined_modifier_flags(node);
        self.last_combined_modifier_flags_node = Some(Arc::clone(node));
        self.last_combined_modifier_flags_result = result;
        result
    }

    pub(crate) fn get_effective_property_name_for_property_name_node(
        &mut self,
        node: &Arc<Node>,
    ) -> (String, bool) {
        let name = get_property_name_for_property_name_node(node);
        if !name.is_empty() || !is_computed_property_name(node) {
            if !name.is_empty() {
                return (name, true);
            }
        }
        if is_computed_property_name(node) {
            let Some(expression) = node.expression().cloned() else {
                return (String::new(), false);
            };
            let t = self.get_type_of_expression(&expression);
            if let Some(name) = self.try_get_name_from_type(&t) {
                return (name, true);
            }
            return (String::new(), false);
        }
        (String::new(), false)
    }

    pub(crate) fn get_applicable_index_info_for_name(
        &mut self,
        t: &Arc<Type>,
        name: &str,
    ) -> Option<Arc<IndexInfo>> {
        if is_late_bound_name(name) {
            return self.get_applicable_index_info(t, &self.es_symbol_type());
        }
        let key_type = self.get_string_literal_type(name);
        self.get_applicable_index_info(t, &key_type)
    }

    pub(crate) fn get_constructors_for_type_arguments(
        &mut self,
        t: &Arc<Type>,
        type_argument_nodes: &[Arc<Node>],
        _location: &Arc<Node>,
    ) -> Vec<Arc<Signature>> {
        let type_arg_count = type_argument_nodes.len();
        self.get_signatures_of_type(t, SignatureKind::Construct)
            .into_iter()
            .filter(|sig| {
                let min = self.get_min_type_argument_count(&sig.type_parameters);
                type_arg_count >= min && type_arg_count <= sig.type_parameters.len()
            })
            .collect()
    }

    pub(crate) fn get_cannot_resolve_module_name_error_for_specific_module(
        &mut self,
        module_name: &Arc<Node>,
    ) -> Option<tsox_core::diagnostics::Message> {
        if is_string_literal(module_name) {
            if node_core_modules().get(module_name.text()).copied().unwrap_or(false) {
                if self.compiler_options.uses_wildcard_types() {
                    return Some(tsox_core::diagnostics::messages_generated::CANNOT_FIND_NAME_0_DO_YOU_NEED_TO_INSTALL_TYPE_DEFINITIONS_FOR_NODE_TRY_NPM_I_SAVE_DEV_TYPES_SLASHNODE);
                }
                return Some(tsox_core::diagnostics::messages_generated::CANNOT_FIND_NAME_0_DO_YOU_NEED_TO_INSTALL_TYPE_DEFINITIONS_FOR_NODE_TRY_NPM_I_SAVE_DEV_TYPES_SLASHNODE_AND_THEN_ADD_NODE_TO_THE_TYPES_FIELD_IN_YOUR_TSCONFIG);
            }
        }
        None
    }
}
