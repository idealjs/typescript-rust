#![allow(unused_imports)]
use crate::checker::types_type_flags_instantiable_non_primitive::SIGNATURE_FLAGS_PROPAGATING_FLAGS;
use crate::checker::mig::wc3::NodeAccessExt;
#[path = "r21k5_defs.rs"]
pub mod r21k5_defs;

#[allow(unused_imports, ambiguous_glob_reexports)]
use crate::checker::*;
#[allow(unused_imports)]
use tsox_frontend::ast::*;
#[allow(unused_imports)]
use tsox_core::diagnostics::messages_generated::*;

pub(crate) use crate::checker::checker::*;
pub(crate) use crate::checker::mig::m1c::*;
pub(crate) use crate::checker::mig::m1c_2::*;
pub(crate) use crate::checker::mig::m1c_3::*;
#[allow(unused_imports)]
use crate::checker::mig::wc3::is_conflicting_private_property;
#[allow(unused_imports)]
use crate::checker::mig::m3a_2::new_diagnostic_chain_for_node;
#[allow(unused_imports)]
use crate::checker::mig::w9a::new_type_mapper;
use std::sync::Arc;

use crate::checker::types::*;
use tsox_frontend::ast::{Diagnostic, Node, NodeData};

impl Checker {
    pub fn create_default_property_wrapper_for_module(
        &mut self,
        symbol: &Arc<Symbol>,
        original_symbol: &Arc<Symbol>,
        anonymous_symbol: Option<&Arc<Symbol>>,
    ) -> Arc<Type> {
        let mut member_table = SymbolTable::new();
        let new_symbol = self.new_symbol(SymbolFlags::Alias, INTERNAL_SYMBOL_NAME_DEFAULT);
        new_symbol.set_parent(&original_symbol);
        self.value_symbol_links.get_or_default(&new_symbol).name_type =
            Some(self.get_string_literal_type("default"));
        self.alias_symbol_links.get_or_default(&new_symbol).alias_target =
            Some(self.resolve_symbol(symbol));
        member_table.insert(INTERNAL_SYMBOL_NAME_DEFAULT, new_symbol);
        let anonymous_symbol = match anonymous_symbol {
            Some(s) => s.clone(),
            None => {
                let mut s = self.new_symbol(SymbolFlags::ObjectLiteral, INTERNAL_SYMBOL_NAME_OBJECT);
                if let Some(s_mut) = Arc::get_mut(&mut s) {
                    s_mut.declarations = original_symbol.declarations.clone();
                }
                s
            }
        };
        self.new_anonymous_type(
            &anonymous_symbol,
            member_table,
            vec![],
            vec![],
            vec![],
        )
    }

    pub fn clone_type_as_module_type(
        &mut self,
        symbol: &Arc<Symbol>,
        module_type: &Arc<Type>,
        reference_parent: &Arc<Node>,
    ) -> Arc<Symbol> {
        let mut result = self.new_symbol(symbol.flags, &symbol.name);
        if let Some(result_mut) = Arc::get_mut(&mut result) {
            result_mut.declarations = symbol.declarations.clone();
            result_mut.value_declaration = symbol.value_declaration.clone();
            result_mut.members = symbol.members.clone();
            result_mut.exports = symbol.exports.clone();
        }
        if let Some(parent) = symbol.parent() {
            result.set_parent(&parent);
        }
        {
            let links = self.export_type_links.get_or_default(&result);
            links.target = Some(symbol.clone());
            links.originating_import = Some(reference_parent.clone());
        }
        let resolved_module_type = self.resolve_structured_type_members(module_type);
        let resolved_type = self.new_anonymous_type(
            &result,
            resolved_module_type
                .as_structured()
                .map(|d| d.members.clone())
                .unwrap_or_default(),
            vec![],
            vec![],
            resolved_module_type
                .as_structured()
                .map(|d| d.index_infos.clone())
                .unwrap_or_default(),
        );
        self.value_symbol_links.get_or_default(&result).resolved_type = Some(resolved_type);
        result
    }

    pub fn contains_same_named_this_property(
        &self,
        this_property: &Arc<Node>,
        expression: &Arc<Node>,
    ) -> bool {
        fn visit(c: &Checker, this_property: &Arc<Node>, node: &Arc<Node>) -> bool {
            if c.is_matching_reference(this_property, node) {
                return true;
            }
            if is_function_like(node) {
                return false;
            }
            node.for_each_child(|child| {
                if visit(c, this_property, child) {
                    panic!("RETURN_TRUE");
                }
                false
            });
            false
        }
        visit(self, this_property, expression)
    }

    pub fn find_applicable_index_info(
        &mut self,
        index_infos: &[Arc<IndexInfo>],
        key_type: &Arc<Type>,
    ) -> Option<Arc<IndexInfo>> {
        let mut string_index_info: Option<Arc<IndexInfo>> = None;
        let mut applicable_infos: Vec<Arc<IndexInfo>> = Vec::with_capacity(8);
        for info in index_infos {
            if info.key_type.as_ref().map(|k| k.id) == self.string_type.get().map(|t| t.id) {
                string_index_info = Some(info.clone());
            } else if self.is_applicable_index_type(key_type, info.key_type.as_ref().unwrap()) {
                applicable_infos.push(info.clone());
            }
        }
        match applicable_infos.len() {
            0 => {
                let string_key = self.string_type.get().cloned();
                if string_index_info.is_some()
                    && let Some(string_key) = string_key
                    && self.is_applicable_index_type(key_type, &string_key)
                {
                    string_index_info
                } else {
                    None
                }
            }
            1 => applicable_infos.into_iter().next(),
            _ => Some(Arc::new(IndexInfo {
                key_type: None,
                value_type: Some(self.unknown_type()),
                is_readonly: false,
                declaration: None,
                index_symbol: None,
                components: vec![],
            })),
        }
    }

    pub fn find_index_info(
        &self,
        index_infos: &[Arc<IndexInfo>],
        key_type: &Arc<Type>,
    ) -> Option<Arc<IndexInfo>> {
        index_infos
            .iter()
            .find(|info| info.key_type.as_ref().map(|k| k.id) == Some(key_type.id))
            .cloned()
    }

    pub fn clone_signature(&mut self, sig: &Arc<Signature>) -> Arc<Signature> {
        let mut result = self.new_signature(
            sig.flags & SIGNATURE_FLAGS_PROPAGATING_FLAGS,
            sig.declaration.as_ref(),
            &sig.type_parameters,
            sig.this_parameter.as_ref(),
            &sig.parameters,
            &self.unknown_type(),
            None,
            sig.min_argument_count as usize,
        );
        if let Some(result_mut) = Arc::get_mut(&mut result) {
            result_mut.target = sig.target.clone();
            result_mut.mapper = sig.mapper.clone();
        }
        result
    }

    pub fn create_signature_instantiation(
        &mut self,
        sig: &Arc<Signature>,
        type_arguments: &[Arc<Type>],
    ) -> Arc<Signature> {
        let mapper = self.create_signature_type_mapper(sig, type_arguments);
        self.instantiate_signature_ex(sig, Some(&mapper), true)
    }

    pub fn create_signature_type_mapper(
        &mut self,
        sig: &Arc<Signature>,
        type_arguments: &[Arc<Type>],
    ) -> Arc<TypeMapper> {
        let type_parameters = self.get_type_parameters_for_mapper(sig);
        Arc::new(new_type_mapper(type_parameters.clone(), type_arguments.to_vec()))
    }

    pub fn get_type_parameters_for_mapper(&mut self, sig: &Arc<Signature>) -> Vec<Arc<Type>> {
        if let Some(target) = &sig.target {
            return self.get_type_parameters_for_mapper(target);
        }
        sig.type_parameters.clone()
    }

    pub fn create_instantiated_symbol_table(
        &mut self,
        symbols: &[Arc<Symbol>],
        m: &Arc<TypeMapper>,
    ) -> SymbolTable {
        if symbols.is_empty() {
            return SymbolTable::new();
        }
        let mut result = SymbolTable::new();
        for symbol in symbols {
            let Some(instantiated) = self.instantiate_symbol(symbol, Some(m)) else {
                continue;
            };
            result.insert(&symbol.name, instantiated);
        }
        result
    }

    pub fn create_symbol_with_type(
        &mut self,
        source: &Arc<Symbol>,
        t: &Arc<Type>,
    ) -> Arc<Symbol> {
        let mut symbol = self.new_symbol_ex(
            source.flags,
            &source.name,
            source.check_flags & CheckFlags::Readonly,
        );
        if let Some(symbol_mut) = Arc::get_mut(&mut symbol) {
            symbol_mut.declarations = source.declarations.clone();
            symbol_mut.value_declaration = source.value_declaration.clone();
        }
        if let Some(parent) = source.parent() {
            symbol.set_parent(&parent);
        }
        self.value_symbol_links.get_or_default(&symbol).resolved_type = Some(t.clone());
        symbol
    }

    pub fn elaborate_never_intersection(
        &mut self,
        chain: Arc<Diagnostic>,
        node: &Arc<Node>,
        t: &Arc<Type>,
    ) -> Arc<Diagnostic> {
        if t.flags.contains(TypeFlags::Intersection)
            && t.object_flags.contains(ObjectFlags::IsNeverIntersection)
        {
            let props = self.get_properties_of_union_or_intersection_type(t);
            if let Some(never_prop) = props.iter().find(|p| self.is_discriminant_with_never_type(p)) {
                let type_str = self.type_to_string_ex(t, crate::checker::mig::m1c::r25k9_defs::no_type_reduction_flags());
                let prop_str = self.symbol_to_string(never_prop);
                return new_diagnostic_chain_for_node(
                    Some(&chain),
                    Some(node),
                    THE_INTERSECTION_0_WAS_REDUCED_TO_NEVER_BECAUSE_PROPERTY_1_HAS_CONFLICTING_TYPES_IN_SOME_CONSTITUENTS,
                    vec![type_str, prop_str],
                ).into();
            }
            if let Some(private_prop) = props.iter().find(|p| is_conflicting_private_property(p)) {
                let type_str = self.type_to_string_ex(t, crate::checker::mig::m1c::r25k9_defs::no_type_reduction_flags());
                let prop_str = self.symbol_to_string(private_prop);
                return new_diagnostic_chain_for_node(
                    Some(&chain),
                    Some(node),
                    THE_INTERSECTION_0_WAS_REDUCED_TO_NEVER_BECAUSE_PROPERTY_1_EXISTS_IN_MULTIPLE_CONSTITUENTS_AND_IS_PRIVATE_IN_SOME,
                    vec![type_str, prop_str],
                ).into();
            }
        }
        chain
    }

    pub fn find_active_mapper(&self, mapper: &Arc<TypeMapper>) -> Option<usize> {
        self.active_mappers
            .iter()
            .rposition(|m| Arc::ptr_eq(m, mapper))
    }

    pub fn clear_active_mapper_caches(&mut self) {
        for cache in &mut self.active_type_mappers_caches {
            cache.clear();
        }
    }

    pub fn clone_type_parameter(&mut self, tp: &Arc<Type>) -> Arc<Type> {
        let mut result = self.new_type_parameter(tp.symbol.clone());
        if let TypeData::TypeParameter(data) = &result.data {
            let _ = data.resolved_default_type.set(self.no_constraint_type());
        }
        if let Some(result_mut) = Arc::get_mut(&mut result) {
            if let TypeData::TypeParameter(data) = &mut result_mut.data {
                data.target = Some(tp.clone());
            } else {
                result_mut.symbol = tp.symbol.clone();
            }
        }
        result
    }

    pub fn create_normalized_type_reference(
        &mut self,
        target: &Arc<Type>,
        type_arguments: &[Arc<Type>],
    ) -> Arc<Type> {
        if target.object_flags.contains(ObjectFlags::Tuple) {
            return self.create_normalized_tuple_type(target, type_arguments);
        }
        self.create_type_reference(target, type_arguments)
    }

    pub fn create_normalized_tuple_type(
        &mut self,
        target: &Arc<Type>,
        element_types: &[Arc<Type>],
    ) -> Arc<Type> {
        self.create_normalized_tuple_type_ex(target, element_types.to_vec(), ObjectFlags::empty())
    }

    pub fn create_computed_enum_type(&mut self, symbol: &Arc<Symbol>) -> Arc<Type> {
        let mut regular_type = self.new_literal_type(TypeFlags::Enum, LiteralValue::None, None);
        if let Some(rt) = Arc::get_mut(&mut regular_type) {
            rt.symbol = Some(symbol.clone());
        }
        let mut fresh_type = self.new_literal_type(TypeFlags::Enum, LiteralValue::None, Some(&regular_type));
        if let Some(ft) = Arc::get_mut(&mut fresh_type) {
            ft.symbol = Some(symbol.clone());
        }
        if let TypeData::Literal(data) = &regular_type.data {
            let _ = data.fresh_type.set(fresh_type.clone());
        }
        if let TypeData::Literal(data) = &fresh_type.data {
            let _ = data.fresh_type.set(fresh_type.clone());
        }
        regular_type
    }

}

