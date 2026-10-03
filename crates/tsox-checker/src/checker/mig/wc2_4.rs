#![allow(unused_imports)]

use crate::checker::checker::*;
use crate::checker::types::*;
use crate::checker::mig::m2c::r18k3_defs::MappedTypeNameTypeKind;
use crate::checker::mig::wc2::r22k3_defs::type_reference_node_type_name;
use crate::checker::mig::wc2::r24k8_defs::map_type_ex_self;
use crate::checker::mig::wc3::ThisAssignmentDeclarationKind;
use tsox_frontend::ast::mig::m3e_4::{get_assignment_declaration_kind, JsDeclarationKind};
use std::sync::Arc;
use tsox_frontend::ast::{self, Node, Symbol, SyntaxKind};

impl Checker {
    pub fn get_intended_type_from_jsdoc_type_reference(&mut self, node: &Arc<Node>) -> Option<Arc<Type>> { ::tsox_core::fntrace::enter("get_intended_type_from_jsdoc_type_reference"); 
        if !node.flags.intersects(ast::NodeFlags::JSDoc) || !ast::is_type_reference_node(node) {
            return None;
        }
        let type_name = type_reference_node_type_name(node);
        if !ast::is_identifier(&type_name) {
            return None;
        }
        let type_args = node.type_arguments();
        match type_name.text() {
            "String" => {
                self.check_no_type_arguments(node, None);
                Some(self.string_type())
            }
            "Number" => {
                self.check_no_type_arguments(node, None);
                Some(self.number_type())
            }
            "BigInt" => {
                self.check_no_type_arguments(node, None);
                Some(self.bigint_type())
            }
            "Boolean" => {
                self.check_no_type_arguments(node, None);
                Some(self.boolean_type())
            }
            "Void" => {
                self.check_no_type_arguments(node, None);
                Some(self.void_type())
            }
            "Undefined" => {
                self.check_no_type_arguments(node, None);
                Some(self.undefined_type())
            }
            "Null" => {
                self.check_no_type_arguments(node, None);
                Some(self.null_type())
            }
            "Function" | "function" => {
                self.check_no_type_arguments(node, None);
                Some(self.global_function_type())
            }
            "array" => {
                if type_args.as_ref().map_or(true, |a| a.nodes.is_empty()) && !self.no_implicit_any {
                    Some(self.any_array_type())
                } else {
                    None
                }
            }
            "promise" => {
                if type_args.as_ref().map_or(true, |a| a.nodes.is_empty()) && !self.no_implicit_any {
                    Some(self.create_promise_type(&self.any_type()))
                } else {
                    None
                }
            }
            "Object" => {
                if type_args.as_ref().is_some_and(|a| a.nodes.len() == 2) {
                    let type_args = type_args.as_ref().unwrap();
                    if let Some(record_symbol) = self.get_global_record_symbol() {
                        let index_type = self.get_type_from_type_node(&type_args.nodes[0]);
                        if self.is_valid_index_key_type(&index_type) {
                            let second = self.get_type_from_type_node(&type_args.nodes[1]);
                            return Some(self.get_type_alias_instantiation(&record_symbol, &[index_type, second], None));
                        }
                    }
                    return Some(self.any_type());
                }
                if !self.no_implicit_any {
                    self.check_no_type_arguments(node, None);
                    return Some(self.any_type());
                }
                None
            }
            _ => None,
        }
    }

    pub fn get_type_of_property_of_contextual_type_ex(
        &mut self,
        t: &Arc<Type>,
        name: &str,
        name_type: &Arc<Type>,
    ) -> Option<Arc<Type>> { ::tsox_core::fntrace::enter("get_type_of_property_of_contextual_type_ex"); 
        map_type_ex_self(self, t, &mut |c, t| {
            if t.flags.intersects(TypeFlags::Intersection) {
                let mut types: Vec<Arc<Type>> = vec![];
                let mut index_info_candidates: Vec<Arc<Type>> = vec![];
                let mut ignore_index_infos = false;
                for constituent_type in t.types().unwrap_or(&[]) {
                    if !constituent_type.flags.intersects(TypeFlags::Object) {
                        continue;
                    }
                    if c.is_generic_mapped_type(constituent_type)
                        && c.get_mapped_type_name_type_kind(constituent_type) != MappedTypeNameTypeKind::Remapping
                    {
                        let substituted_type = c.get_indexed_mapped_type_substituted_type_of_contextual_type(
                            constituent_type,
                            name,
                            Some(name_type),
                        );
                        types = c.append_contextual_property_type_constituent(types, substituted_type.as_ref());
                        continue;
                    }
                    match c.get_type_of_concrete_property_of_contextual_type(constituent_type, name) {
                        None => {
                            if !ignore_index_infos {
                                index_info_candidates.push(Arc::clone(constituent_type));
                            }
                        }
                        Some(property_type) => {
                            ignore_index_infos = true;
                            index_info_candidates.clear();
                            types = c.append_contextual_property_type_constituent(types, Some(&property_type));
                        }
                    }
                }
                for candidate in &index_info_candidates {
                    let index_info_type =
                        c.get_type_from_index_infos_of_contextual_type(candidate, name, Some(name_type));
                    types = c.append_contextual_property_type_constituent(types, index_info_type.as_ref());
                }
                if types.is_empty() {
                    return None;
                }
                if types.len() == 1 {
                    return Some(types.into_iter().next().unwrap());
                }
                return Some(c.get_intersection_type(types));
            }
            if !t.flags.intersects(TypeFlags::Object) {
                return None;
            }
            if c.is_generic_mapped_type(t)
                && c.get_mapped_type_name_type_kind(t) != MappedTypeNameTypeKind::Remapping
            {
                return c.get_indexed_mapped_type_substituted_type_of_contextual_type(
                    t,
                    name,
                    Some(name_type),
                );
            }
            if let Some(result) = c.get_type_of_concrete_property_of_contextual_type(t, name) {
                return Some(result);
            }
            c.get_type_from_index_infos_of_contextual_type(t, name, Some(name_type))
        }, true)
    }

    pub fn get_widened_type_for_assignment_declaration(&mut self, symbol: &Arc<Symbol>) -> Arc<Type> { ::tsox_core::fntrace::enter("get_widened_type_for_assignment_declaration"); 
        let mut t: Option<Arc<Type>> = None;
        let (kind, location) = self.is_constructor_declared_this_property(symbol);
        match kind {
            ThisAssignmentDeclarationKind::Typed => {
                let location = location.expect("location should not be nil when this assignment has a type.");
                t = Some(self.get_type_from_type_node(&location));
            }
            ThisAssignmentDeclarationKind::Constructor => {
                let location = location.expect("constructor should not be nil when this assignment is in a constructor.");
                t = self.get_flow_type_in_constructor(symbol, &location);
            }
            ThisAssignmentDeclarationKind::Method => {
                t = self.get_type_of_property_in_base_class(symbol);
            }
            _ => {}
        }
        if t.is_none() {
            let mut types: Vec<Arc<Type>> = vec![];
            for (i, declaration) in symbol.declarations.iter().enumerate() {
                if ast::is_binary_expression(declaration) && declaration.type_().is_some() {
                    t = Some(self.get_type_from_type_node(&declaration.type_().unwrap()));
                    break;
                }
                if let Some(assigned_type) = self.get_assignment_declaration_initializer_type(declaration) {
                    let is_exports_property_first = get_assignment_declaration_kind(declaration)
                        != JsDeclarationKind::ExportsProperty
                        || i != 0
                        || symbol.declarations.len() == 1
                        || !assigned_type.flags.intersects(TypeFlags::Undefined);
                    if is_exports_property_first {
                        if !types.iter().any(|x| Arc::ptr_eq(x, &assigned_type)) {
                            types.push(assigned_type);
                        }
                    }
                }
            }
            if kind == ThisAssignmentDeclarationKind::Method && !types.is_empty() && self.strict_null_checks {
                let undefined_or_missing = self.undefined_or_missing_type();
                if !types.iter().any(|x| Arc::ptr_eq(x, &undefined_or_missing)) {
                    types.push(undefined_or_missing);
                }
            }
            if t.is_none() {
                t = Some(if types.is_empty() {
                    self.any_type()
                } else {
                    self.get_union_type(types)
                });
            }
        }
        let t = self.get_widened_type(&t.unwrap_or_else(|| self.any_type()));
        if let Some(value_declaration) = &symbol.value_declaration {
            if ast::is_in_js_file(value_declaration)
                && self
                    .filter_type(&t, &mut |c| !c.flags.intersects(TYPE_FLAGS_NULLABLE))
                    .flags
                    .intersects(TypeFlags::Never)
            {
                self.report_implicit_any(value_declaration, &self.any_type(), WideningKind::Normal);
                return self.any_type();
            }
        }
        t
    }
}

impl Checker {
    pub fn get_object_literal_index_info(
        &mut self,
        is_readonly: bool,
        properties: &[Arc<Symbol>],
        key_type: &Arc<Type>,
    ) -> IndexInfo { ::tsox_core::fntrace::enter("get_object_literal_index_info"); 
        let mut prop_types: Vec<Arc<Type>> = vec![];
        let mut components: Vec<Arc<Node>> = vec![];
        for prop in properties {
            let string_key = Arc::ptr_eq(key_type, &self.string_type()) && !self.is_symbol_with_symbol_name(prop);
            let number_key = Arc::ptr_eq(key_type, &self.number_type()) && self.is_symbol_with_numeric_name(prop);
            let symbol_key = Arc::ptr_eq(key_type, &self.es_symbol_type()) && self.is_symbol_with_symbol_name(prop);
            if string_key || number_key || symbol_key {
                prop_types.push(self.get_type_of_symbol(prop));
                if self.is_symbol_with_computed_name(prop) {
                    if let Some(decl) = prop.declarations.first() {
                        components.push(Arc::clone(decl));
                    }
                }
            }
        }
        let union_type = if !prop_types.is_empty() {
            self.get_union_type_ex(prop_types, UnionReduction::Subtype)
        } else {
            self.undefined_type()
        };
        IndexInfo {
            key_type: Some(Arc::clone(key_type)),
            value_type: Some(Arc::clone(&union_type)),
            is_readonly,
            declaration: None,
            index_symbol: None,
            components: components.to_vec(),
        }
    }
}
