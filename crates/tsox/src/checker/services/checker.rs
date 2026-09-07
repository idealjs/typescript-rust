#![allow(unused_imports)]

use super::*;

impl Checker {
    pub fn get_symbols_in_scope(
        &self,
        _location: &Arc<Node>,
        _meaning: SymbolFlags,
    ) -> Vec<Arc<Symbol>> {
        Vec::new()
    }

    pub fn get_exports_of_module(&self, symbol: &Arc<Symbol>) -> Vec<Arc<Symbol>> {
        symbols_to_array(&self.get_exports_of_module_table(symbol))
    }

    pub fn get_exports_of_module_table(&self, module_symbol: &Arc<Symbol>) -> SymbolTable {
        if let Some(links) = self.module_symbol_links.get(module_symbol) {
            if !links.resolved_exports.is_empty() {
                return links.resolved_exports.clone();
            }
        }

        module_symbol.exports.clone()
    }

    pub fn for_each_export_and_property_of_module(
        &mut self,
        module_symbol: &Arc<Symbol>,
        cb: &mut dyn FnMut(&Arc<Symbol>, &str),
    ) {
        for (key, exported_symbol) in self
            .get_exports_of_module_table(module_symbol)
            .entries
            .iter()
        {
            if !is_reserved_member_name(key) {
                cb(exported_symbol, key);
            }
        }

        let export_equals = self.resolve_external_module_symbol(module_symbol, false);
        if Arc::ptr_eq(&export_equals, module_symbol) {
            return;
        }

        let type_of_symbol = self.get_type_of_symbol(&export_equals);
        if !self.should_treat_properties_of_external_module_as_exports(&type_of_symbol) {
            return;
        }

        let reduced_type = self.get_reduced_apparent_type(&type_of_symbol);
        if !reduced_type.flags.intersects(TYPE_FLAGS_STRUCTURED_TYPE) {
            return;
        }
        if let Some(structured) = reduced_type.as_structured() {
            for (name, symbol) in structured.members.entries.iter() {
                if self.is_named_member(symbol, name) {
                    cb(symbol, name);
                }
            }
        }
    }

    pub fn is_valid_property_access(&mut self, node: &Arc<Node>, property_name: &str) -> bool {
        match node.kind {
            SyntaxKind::PropertyAccessExpression => {
                let is_super = if let Some(expr) = node.expression() {
                    expr.kind == SyntaxKind::SuperKeyword
                } else {
                    false
                };
                let t = if let Some(expr) = node.expression() {
                    self.check_expression(expr);
                    let widened = self.get_widened_type_of_expression(expr);
                    widened
                } else {
                    self.any_type()
                };
                self.is_valid_property_access_with_type(node, is_super, property_name, &t)
            }
            SyntaxKind::QualifiedName => {
                let t = if let NodeData::QualifiedName(data) = &node.data {
                    self.check_expression(&data.left);
                    self.get_widened_type_of_expression(&data.left)
                } else {
                    self.any_type()
                };
                self.is_valid_property_access_with_type(node, false, property_name, &t)
            }
            SyntaxKind::ImportType => {
                let t = self.get_type_from_type_node(node);
                self.is_valid_property_access_with_type(node, false, property_name, &t)
            }
            _ => {
                panic!(
                    "Unexpected node kind in isValidPropertyAccess: {:?}",
                    node.kind
                )
            }
        }
    }

    pub fn is_valid_property_access_with_type(
        &self,
        node: &Arc<Node>,
        is_super: bool,
        property_name: &str,
        t: &Arc<Type>,
    ) -> bool {
        if is_type_any(t) {
            return true;
        }
        let prop = self.get_property_of_type_cached(t, property_name);
        prop.is_some() && self.is_property_accessible(node, is_super, false, t, &prop.unwrap())
    }

    pub fn is_valid_property_access_for_completions(
        &self,
        node: &Arc<Node>,
        t: &Arc<Type>,
        property: &Arc<Symbol>,
    ) -> bool {
        let is_super = node.kind == SyntaxKind::PropertyAccessExpression
            && node
                .expression()
                .map(|e| e.kind == SyntaxKind::SuperKeyword)
                .unwrap_or(false);
        self.is_property_accessible(node, is_super, false, t, property)
    }

    pub fn get_all_possible_properties_of_types(
        &mut self,
        types: &[Arc<Type>],
    ) -> Vec<Arc<Symbol>> {
        let union_type = self.get_union_type(types.to_vec());
        if !union_type.flags.contains(TypeFlags::Union) {
            return self.get_augmented_properties_of_type(&union_type);
        }

        let mut props: std::collections::HashMap<String, Arc<Symbol>> =
            std::collections::HashMap::new();
        for member_type in types {
            let augmented_props = self.get_augmented_properties_of_type(member_type);
            for p in augmented_props {
                if !props.contains_key(&p.name) {
                    props.insert(p.name.clone(), Arc::clone(&p));
                }
            }
        }
        props.into_values().collect()
    }

    pub fn is_unknown_symbol(&self, symbol: &Arc<Symbol>) -> bool {
        self.unknown_symbol
            .as_ref()
            .map(|s| Arc::ptr_eq(s, symbol))
            .unwrap_or(false)
    }

    pub fn is_undefined_symbol(&self, symbol: &Arc<Symbol>) -> bool {
        self.undefined_symbol
            .as_ref()
            .map(|s| Arc::ptr_eq(s, symbol))
            .unwrap_or(false)
    }

    pub fn is_arguments_symbol(&self, symbol: &Arc<Symbol>) -> bool {
        self.arguments_symbol
            .as_ref()
            .map(|s| Arc::ptr_eq(s, symbol))
            .unwrap_or(false)
    }

    pub fn get_non_optional_type(&self, t: &Arc<Type>) -> Arc<Type> {
        self.remove_optional_type_marker(t)
    }

    pub fn get_string_index_type(&mut self, t: &Arc<Type>) -> Option<Arc<Type>> {
        self.get_index_type_of_type(t, IndexKind::String)
    }

    pub fn get_number_index_type(&mut self, t: &Arc<Type>) -> Option<Arc<Type>> {
        self.get_index_type_of_type(t, IndexKind::Number)
    }

    pub fn get_element_type_of_array_type(&self, t: &Arc<Type>) -> Option<Arc<Type>> {
        let type_args = self.get_type_arguments(t);
        if let Some(elem) = type_args.first() {
            return Some(Arc::clone(elem));
        }
        None
    }

    pub fn get_call_signatures(&self, t: &Arc<Type>) -> Vec<Arc<Signature>> {
        self.get_signatures_of_type(t, SignatureKind::Call)
    }

    pub fn get_construct_signatures(&self, t: &Arc<Type>) -> Vec<Arc<Signature>> {
        self.get_signatures_of_type(t, SignatureKind::Construct)
    }

    pub fn get_apparent_properties(&mut self, t: &Arc<Type>) -> Vec<Arc<Symbol>> {
        self.get_augmented_properties_of_type(t)
    }

    pub fn get_augmented_properties_of_type(&mut self, t: &Arc<Type>) -> Vec<Arc<Symbol>> {
        let apparent = self.get_apparent_type(t);
        let props_list = self.get_properties_of_type(&apparent);
        let mut props_by_name: std::collections::HashMap<String, Arc<Symbol>> =
            std::collections::HashMap::new();
        for p in &props_list {
            props_by_name.insert(p.name.clone(), Arc::clone(p));
        }

        let call_sigs = self.get_signatures_of_type(&apparent, SignatureKind::Call);
        let construct_sigs = self.get_signatures_of_type(&apparent, SignatureKind::Construct);
        let function_type = if !call_sigs.is_empty() {
            self.global_callable_function_type()
        } else if !construct_sigs.is_empty() {
            self.global_newable_function_type()
        } else {
            None
        };

        if let Some(ref ft) = function_type {
            for p in self.get_properties_of_type(ft) {
                if !props_by_name.contains_key(&p.name) {
                    props_by_name.insert(p.name.clone(), Arc::clone(&p));
                }
            }
        }

        self.get_named_members(&props_by_name)
    }

    pub fn try_get_member_in_module_exports_and_properties(
        &mut self,
        member_name: &str,
        module_symbol: &Arc<Symbol>,
    ) -> Option<Arc<Symbol>> {
        if let Some(symbol) = self.try_get_member_in_module_exports(member_name, module_symbol) {
            return Some(symbol);
        }

        let export_equals = self.resolve_external_module_symbol(module_symbol, false);
        if Arc::ptr_eq(&export_equals, module_symbol) {
            return None;
        }

        let t = self.get_type_of_symbol(&export_equals);
        if self.should_treat_properties_of_external_module_as_exports(&t) {
            return self.get_property_of_type(&t, member_name);
        }
        None
    }

    pub fn try_get_member_in_module_exports(
        &self,
        member_name: &str,
        module_symbol: &Arc<Symbol>,
    ) -> Option<Arc<Symbol>> {
        let symbol_table = self.get_exports_of_module_table(module_symbol);
        symbol_table.get(member_name).cloned()
    }

    pub fn should_treat_properties_of_external_module_as_exports(
        &self,
        resolved_external_module_type: &Arc<Type>,
    ) -> bool {
        !resolved_external_module_type
            .flags
            .intersects(TYPE_FLAGS_PRIMITIVE)
            || resolved_external_module_type
                .object_flags
                .contains(ObjectFlags::Class)
            || self.is_array_type(resolved_external_module_type)
            || is_tuple_type(resolved_external_module_type)
    }

    pub fn get_contextual_type_for_services(
        &mut self,
        node: &Arc<Node>,
        context_flags: ContextFlags,
    ) -> Option<Arc<Type>> {
        self.get_contextual_type(node, context_flags)
    }

}
