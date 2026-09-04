#![allow(dead_code)]
#![allow(unused_variables)]

use std::sync::Arc;

use crate::ast::{
    CheckFlags, Node, NodeData, SourceFile, Symbol, SymbolFlags, SymbolTable, SyntaxKind,
};
use crate::evaluator::EvalValue;

use super::checker::Checker;
use super::types::*;
use super::utilities::{
    get_property_name_from_type, is_literal_type, is_tuple_type, is_type_any,
    is_type_usable_as_property_name,
};

pub fn is_reserved_member_name(name: &str) -> bool {
    let mut chars = name.chars();
    match chars.next() {
        Some(c) if c == '\u{FE}' => match chars.next() {
            Some('@') | Some('#') => false,
            Some(_) => true,
            None => false,
        },
        _ => false,
    }
}

pub fn symbols_to_array(symbols: &SymbolTable) -> Vec<Arc<Symbol>> {
    symbols
        .entries
        .values()
        .filter(|s| !is_reserved_member_name(&s.name))
        .cloned()
        .collect()
}

pub fn introduces_arguments_exotic_object(node: &Arc<Node>) -> bool {
    matches!(
        node.kind,
        SyntaxKind::MethodDeclaration
            | SyntaxKind::MethodSignature
            | SyntaxKind::Constructor
            | SyntaxKind::GetAccessor
            | SyntaxKind::SetAccessor
            | SyntaxKind::FunctionDeclaration
            | SyntaxKind::FunctionExpression
    )
}

pub const KNOWN_GENERIC_TYPE_NAMES: &[&str] = &[
    "Array",
    "ArrayLike",
    "ReadonlyArray",
    "Promise",
    "PromiseLike",
    "Iterable",
    "IterableIterator",
    "AsyncIterable",
    "Set",
    "WeakSet",
    "ReadonlySet",
    "Map",
    "WeakMap",
    "ReadonlyMap",
    "Partial",
    "Required",
    "Readonly",
    "Pick",
    "Omit",
    "NonNullable",
];

fn is_known_generic_name(name: &str) -> bool {
    KNOWN_GENERIC_TYPE_NAMES.contains(&name)
}

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
        let prop = self.get_property_of_type(t, property_name);
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

    pub fn get_resolved_signature_for_signature_help(
        &mut self,
        node: &Arc<Node>,
        argument_count: i32,
    ) -> (Option<Arc<Signature>>, Vec<Arc<Signature>>) {

        self.get_resolved_signature_worker(node, CheckMode::IsForSignatureHelp, argument_count)
    }

    pub fn skip_alias(&self, symbol: &Arc<Symbol>) -> Arc<Symbol> {
        if symbol.flags.contains(SymbolFlags::Alias) {
            return self.get_aliased_symbol(symbol);
        }
        Arc::clone(symbol)
    }

    pub fn get_aliased_symbol(&self, symbol: &Arc<Symbol>) -> Arc<Symbol> {
        if let Some(links) = self.alias_symbol_links.get(symbol) {
            if let Some(ref target) = links.alias_target {
                return Arc::clone(target);
            }
            if let Some(ref target) = links.immediate_target {
                return Arc::clone(target);
            }
        }
        Arc::clone(symbol)
    }

    pub fn get_root_symbols(&self, symbol: &Arc<Symbol>) -> Vec<Arc<Symbol>> {
        let roots = self.get_immediate_root_symbols(symbol);
        if roots.is_empty() {
            return vec![Arc::clone(symbol)];
        }
        let mut result = Vec::new();
        for root in &roots {
            result.extend(self.get_root_symbols(root));
        }
        result
    }

    pub fn get_immediate_root_symbols(&self, symbol: &Arc<Symbol>) -> Vec<Arc<Symbol>> {
        if symbol.check_flags.intersects(CheckFlags::SYNTHETIC) {

            if let Some(links) = self.value_symbol_links.get(symbol) {
                if let Some(ref containing) = links.containing_type {
                    let types = containing.types().unwrap_or(&[]);
                    let mut result = Vec::new();
                    for t in types {
                        if let Some(prop) = self.get_property_of_type(t, &symbol.name) {
                            result.push(prop);
                        }
                    }
                    return result;
                }
            }
        }
        if symbol.flags.contains(SymbolFlags::Transient) {
            if let Some(links) = self.spread_links.get(symbol) {
                if links.left_spread.is_some() {
                    let mut result = Vec::new();
                    if let Some(ref left) = links.left_spread {
                        result.push(Arc::clone(left));
                    }
                    if let Some(ref right) = links.right_spread {
                        result.push(Arc::clone(right));
                    }
                    return result;
                }
            }
            if let Some(links) = self.mapped_symbol_links.get(symbol) {
                if let Some(ref origin) = links.synthetic_origin {
                    return vec![Arc::clone(origin)];
                }
            }
            let target = self.try_get_target(symbol);
            if let Some(target) = target {
                return vec![target];
            }
        }
        Vec::new()
    }

    pub fn try_get_target(&self, symbol: &Arc<Symbol>) -> Option<Arc<Symbol>> {
        let mut target = None;
        let mut next = Arc::clone(symbol);
        loop {
            let resolved = if let Some(links) = self.value_symbol_links.get(&next) {
                links.target.clone()
            } else if let Some(links) = self.export_type_links.get(&next) {
                links.target.clone()
            } else {
                None
            };
            match resolved {
                Some(n) => {
                    target = Some(Arc::clone(&n));
                    next = n;
                }
                None => break,
            }
        }
        target
    }

    pub fn get_mapped_type_symbol_of_property(&self, symbol: &Arc<Symbol>) -> Option<Arc<Symbol>> {
        if let Some(value_links) = self.value_symbol_links.get(symbol) {
            if let Some(ref containing) = value_links.containing_type {
                return containing.symbol.clone();
            }
        }
        None
    }

    pub fn get_export_symbol_of_symbol(&self, symbol: &Arc<Symbol>) -> Arc<Symbol> {

        let source = if let Some(ref export) = symbol.export_symbol {
            Arc::clone(export)
        } else {
            Arc::clone(symbol)
        };

        source
    }

    pub fn get_export_specifier_local_target_symbol(
        &mut self,
        node: &Arc<Node>,
    ) -> Option<Arc<Symbol>> {
        match node.kind {
            SyntaxKind::ExportSpecifier => {

                None
            }
            SyntaxKind::Identifier => {

                None
            }
            _ => {
                panic!(
                    "Unhandled case in getExportSpecifierLocalTargetSymbol, node should be ExportSpecifier | Identifier"
                )
            }
        }
    }

    pub fn get_shorthand_assignment_value_symbol(
        &mut self,
        location: Option<&Arc<Node>>,
    ) -> Option<Arc<Symbol>> {
        if let Some(loc) = location {
            if loc.kind == SyntaxKind::ShorthandPropertyAssignment {
                if let Some(name) = loc.name() {

                    return None;
                }
            }
        }
        None
    }

    pub fn get_symbols_of_parameter_property_declaration(
        &self,
        parameter: &Arc<Node>,
        parameter_name: &str,
    ) -> Option<(Arc<Symbol>, Arc<Symbol>)> {
        let constructor_declaration = parameter.parent.as_ref()?;
        let class_declaration = constructor_declaration.parent.as_ref()?;

        let _ = parameter_name;
        let _ = class_declaration;
        None
    }

    pub fn is_declaration_used(
        &mut self,
        source_file: &Arc<SourceFile>,
        identifier: &Arc<Node>,
        jsx_elements_present: bool,
        jsx_mode_needs_explicit_import: bool,
    ) -> bool {
        if jsx_elements_present && jsx_mode_needs_explicit_import {

            let identifier_text = identifier.text();

            if identifier_text == "React" || identifier_text == "h" {
                return true;
            }
        }

        let symbol = self.get_symbol_at_location(identifier);
        let symbol = match symbol {
            Some(s) => s,
            None => return true,
        };

        self.is_symbol_referenced_in_file(source_file, identifier, &symbol)
    }

    pub fn is_symbol_referenced_in_file(
        &mut self,
        source_file: &Arc<SourceFile>,
        definition: &Arc<Node>,
        symbol: &Arc<Symbol>,
    ) -> bool {
        let identifier_text = definition.text();
        for token in get_possible_symbol_reference_nodes(source_file, identifier_text, None) {
            if token.kind != SyntaxKind::Identifier {
                continue;
            }
            if token.text() != identifier_text {
                continue;
            }
            if Arc::ptr_eq(&token, definition) {
                continue;
            }
            let ref_symbol = self.get_symbol_at_location(&token);
            if let Some(ref ref_sym) = ref_symbol {
                if Arc::ptr_eq(ref_sym, symbol) {
                    return true;
                }
            }
            if let Some(ref parent) = token.parent {
                if parent.kind == SyntaxKind::ShorthandPropertyAssignment {
                    let shorthand_symbol = self.get_shorthand_assignment_value_symbol(Some(parent));
                    if let Some(ref shorthand) = shorthand_symbol {
                        if Arc::ptr_eq(shorthand, symbol) {
                            return true;
                        }
                    }
                }
            }
        }
        false
    }

    pub fn get_references_to_symbol_in_file(
        &mut self,
        source_file: &Arc<SourceFile>,
        symbol: &Arc<Symbol>,
    ) -> Vec<Arc<Node>> {
        let identifier_text = &symbol.name;
        let mut result = Vec::new();
        for token in get_possible_symbol_reference_nodes(source_file, identifier_text, None) {
            if token.kind != SyntaxKind::Identifier {
                continue;
            }
            if token.text() != identifier_text.as_str() {
                continue;
            }
            let ref_symbol = self.get_symbol_at_location(&token);
            if let Some(ref ref_sym) = ref_symbol {
                if Arc::ptr_eq(ref_sym, symbol) {
                    result.push(Arc::clone(&token));
                    continue;
                }
            }
            if let Some(ref parent) = token.parent {
                if parent.kind == SyntaxKind::ShorthandPropertyAssignment {
                    let shorthand_symbol = self.get_shorthand_assignment_value_symbol(Some(parent));
                    if let Some(ref shorthand) = shorthand_symbol {
                        if Arc::ptr_eq(shorthand, symbol) {
                            result.push(Arc::clone(&token));
                            continue;
                        }
                    }
                }
            }
        }
        result
    }

    pub fn get_type_argument_constraint(&mut self, node: &Arc<Node>) -> Option<Arc<Type>> {

        None
    }

    pub fn is_type_invalid_due_to_union_discriminant(
        &mut self,
        contextual_type: &Arc<Type>,
        obj: &Arc<Node>,
    ) -> bool {
        let properties: Vec<Arc<Node>> = match &obj.data {
            NodeData::ObjectLiteralExpression(data) => data.properties.nodes.clone(),
            _ => Vec::new(),
        };
        for property in &properties {
            let mut name_type = None;
            if let Some(property_name) = property.name() {
                if property_name.kind == SyntaxKind::JsxNamespacedName {
                    name_type = Some(self.get_string_literal_type(property_name.text()));
                } else {
                    name_type = self.get_literal_type_from_property_name(property_name);
                }
            }
            let mut name = String::new();
            if let Some(ref nt) = name_type {
                if is_type_usable_as_property_name(nt) {
                    name = get_property_name_from_type(nt);
                }
            }
            let mut expected = None;
            if !name.is_empty() {
                expected = self.get_type_of_property_of_type(contextual_type, &name);
            }
            if let Some(ref exp) = expected {
                if is_literal_type(exp) {
                    let prop_type = self.get_type_of_node(property);
                    if !self.is_type_assignable_to(&prop_type, exp) {
                        return true;
                    }
                }
            }
        }
        false
    }

    pub fn get_exports_and_properties_of_module(
        &mut self,
        module_symbol: &Arc<Symbol>,
    ) -> Vec<Arc<Symbol>> {
        let mut exports = self.get_exports_of_module_as_array(module_symbol);
        let export_equals = self.resolve_external_module_symbol(module_symbol, false);
        if !Arc::ptr_eq(&export_equals, module_symbol) {
            let t = self.get_type_of_symbol(&export_equals);
            if self.should_treat_properties_of_external_module_as_exports(&t) {
                exports.extend(self.get_properties_of_type(&t));
            }
        }
        exports
    }

    pub fn get_exports_of_module_as_array(&self, module_symbol: &Arc<Symbol>) -> Vec<Arc<Symbol>> {
        symbols_to_array(&self.get_exports_of_module_table(module_symbol))
    }

    pub fn get_jsx_intrinsic_tag_names_at(&mut self, location: &Arc<Node>) -> Vec<Arc<Symbol>> {

        let intrinsics = self.get_jsx_type_symbol("IntrinsicElements", location);
        if let Some(intrinsics) = intrinsics {
            return self.get_properties_of_type(&intrinsics);
        }
        Vec::new()
    }

    pub fn get_constant_value_for_services(&mut self, node: &Arc<Node>) -> Option<EvalValue> {
        if node.kind == SyntaxKind::EnumMember {
            return self.get_enum_member_value(node).value;
        }

        self.check_expression(node);

        let symbol = self
            .symbol_node_links
            .get(node)
            .and_then(|l| l.resolved_symbol.clone());

        if let Some(ref sym) = symbol {
            if sym.flags.contains(SymbolFlags::EnumMember) {

                if let Some(ref member) = sym.value_declaration {
                    if let Some(ref parent) = member.parent {

                        if parent.flags.contains(crate::ast::NodeFlags::Const) {
                            return self.get_enum_member_value(member).value;
                        }
                    }
                }
            }
        }

        None
    }

    pub fn get_resolved_signature_worker(
        &mut self,
        _node: &Arc<Node>,
        _check_mode: CheckMode,
        _argument_count: i32,
    ) -> (Option<Arc<Signature>>, Vec<Arc<Signature>>) {

        (None, Vec::new())
    }

    pub fn get_candidate_signatures_for_string_literal_completions(
        &mut self,
        _call: &Arc<Node>,
        _editing_argument: &Arc<Node>,
    ) -> Vec<Arc<Signature>> {

        Vec::new()
    }

    pub fn get_type_at_position_for_services(
        &mut self,
        sig: &Arc<Signature>,
        pos: usize,
    ) -> Arc<Type> {
        self.get_type_at_position(sig, pos)
    }

    pub fn get_type_parameter_at_position(
        &mut self,
        sig: &Arc<Signature>,
        pos: usize,
    ) -> Arc<Type> {
        let t = self.get_type_at_position(sig, pos);

        if t.flags.contains(TypeFlags::Index) {
            if let TypeData::Index(index_data) = &t.data {
                if let Some(ref target) = index_data.target {
                    if self.is_this_type_parameter(target) {
                        if let Some(constraint) = self.get_base_constraint_of_type(target) {
                            return self.get_index_type(&constraint);
                        }
                    }
                }
            }
        }
        t
    }

    pub fn get_contextual_type_for_array_literal_at_position(
        &mut self,
        contextual_array_type: Option<&Arc<Type>>,
        array_literal: &Arc<Node>,
        position: usize,
    ) -> Option<Arc<Type>> {
        let contextual_array_type = contextual_array_type?;
        let mut first_spread_index = -1i32;
        let mut last_spread_index = -1i32;
        let mut element_index = 0i32;

        let elements: Vec<Arc<Node>> = match &array_literal.data {
            NodeData::ArrayLiteralExpression(data) => data.elements.nodes.clone(),
            _ => return None,
        };

        for (i, elem) in elements.iter().enumerate() {
            if elem.pos() < position {
                element_index += 1;
            }
            if elem.kind == SyntaxKind::SpreadElement {
                if first_spread_index == -1 {
                    first_spread_index = i as i32;
                }
                last_spread_index = i as i32;
            }
        }

        self.get_contextual_type_for_element_expression(
            contextual_array_type,
            element_index as usize,
            None,
            first_spread_index,
            last_spread_index,
        )
    }

    pub fn get_first_type_argument_from_known_type(&mut self, t: &Arc<Type>) -> Option<Arc<Type>> {
        if t.object_flags.contains(ObjectFlags::Reference) {
            if let Some(ref symbol) = t.symbol {
                if is_known_generic_name(&symbol.name) {

                    let type_args = self.get_type_arguments(t);
                    return type_args.into_iter().next();
                }
            }
        }
        if let Some(ref alias) = t.alias {
            if let Some(ref alias_symbol) = alias.symbol {
                if is_known_generic_name(&alias_symbol.name) {

                    return alias.type_arguments.first().cloned();
                }
            }
        }
        None
    }

    pub fn get_property_symbols_from_contextual_type(
        &mut self,
        node: &Arc<Node>,
        contextual_type: &Arc<Type>,
        union_symbol_ok: bool,
    ) -> Vec<Arc<Symbol>> {
        let name = node.name().map(|n| n.text()).unwrap_or("");
        if name.is_empty() {
            return Vec::new();
        }

        if !contextual_type.flags.contains(TypeFlags::Union) {
            if let Some(symbol) = self.get_property_of_type(contextual_type, name) {
                return vec![symbol];
            }
            return Vec::new();
        }

        let mut filtered_types: Vec<Arc<Type>> = contextual_type
            .types()
            .unwrap_or(&[])
            .iter()
            .cloned()
            .collect();

        if let Some(ref parent) = node.parent {
            if parent.kind == SyntaxKind::ObjectLiteralExpression
                || parent.kind == SyntaxKind::JsxAttributes
            {
                filtered_types
                    .retain(|t| !self.is_type_invalid_due_to_union_discriminant(t, parent));
            }
        }

        let mut discriminated_property_symbols: Vec<Arc<Symbol>> = filtered_types
            .iter()
            .filter_map(|t| self.get_property_of_type(t, name))
            .collect();

        let constituent_count = contextual_type.types().map(|t| t.len()).unwrap_or(0);

        if union_symbol_ok
            && (discriminated_property_symbols.is_empty()
                || discriminated_property_symbols.len() == constituent_count)
        {
            if let Some(symbol) = self.get_property_of_type(contextual_type, name) {
                return vec![symbol];
            }
        }

        if filtered_types.is_empty() && discriminated_property_symbols.is_empty() {

            return contextual_type
                .types()
                .unwrap_or(&[])
                .iter()
                .filter_map(|t| self.get_property_of_type(t, name))
                .collect();
        }

        let mut seen = std::collections::HashSet::new();
        discriminated_property_symbols.retain(|s| seen.insert(s.id()));
        discriminated_property_symbols
    }

    pub fn get_property_symbol_of_destructuring_assignment(
        &mut self,
        location: &Arc<Node>,
    ) -> Option<Arc<Symbol>> {

        let parent = location.parent.as_ref()?;
        let grandparent = parent.parent.as_ref()?;

        if is_array_literal_or_object_literal_destructuring_pattern(grandparent) {
            if let Some(type_of_object_literal) = self.get_type_of_assignment_pattern(grandparent) {
                return self.get_property_of_type(&type_of_object_literal, location.text());
            }
        }
        None
    }

    pub fn get_type_of_assignment_pattern(&mut self, expr: &Arc<Node>) -> Option<Arc<Type>> {

        None
    }

    pub fn get_signature_from_declaration(&mut self, _node: &Arc<Node>) -> Option<Arc<Signature>> {

        None
    }

    pub fn is_lib_symbol_for_hover_verbosity(&self, symbol: &Arc<Symbol>) -> bool {
        for decl in &symbol.declarations {
            if let Some(sf) = self.get_source_file_of_node(decl) {
                if self.program.is_source_file_default_library(&sf.file_name) {
                    return true;
                }
            }
        }
        false
    }

    pub fn is_lib_type_for_hover_verbosity(&self, t: &Arc<Type>) -> bool {
        let symbol = if t.object_flags.contains(ObjectFlags::Reference) {
            t.target().and_then(|target| target.symbol.clone())
        } else {
            t.symbol.clone()
        };
        if let Some(ref sym) = symbol {
            if self.is_lib_symbol_for_hover_verbosity(sym) {
                return true;
            }
        }
        is_tuple_type(t)
    }

    pub fn resolve_external_module_symbol(
        &self,
        module_symbol: &Arc<Symbol>,
        _dont_resolve_alias: bool,
    ) -> Arc<Symbol> {

        if let Some(export_equals) = module_symbol.exports.get("export=") {
            return Arc::clone(export_equals);
        }
        Arc::clone(module_symbol)
    }

    pub fn get_members_of_symbol(&self, symbol: &Arc<Symbol>) -> SymbolTable {
        symbol.members.clone()
    }

    pub fn remove_optional_type_marker(&self, t: &Arc<Type>) -> Arc<Type> {

        Arc::clone(t)
    }

    pub fn get_index_type_of_type(
        &self,
        t: &Arc<Type>,
        _index_kind: IndexKind,
    ) -> Option<Arc<Type>> {
        if let Some(structured) = t.as_structured() {
            for info in &structured.index_infos {
                if let Some(ref key_type) = info.key_type {
                    let matches = match _index_kind {
                        IndexKind::String => key_type.flags.contains(TypeFlags::String),
                        IndexKind::Number => key_type.flags.contains(TypeFlags::Number),
                    };
                    if matches {
                        return info.value_type.clone();
                    }
                }
            }
        }
        None
    }

    pub fn get_apparent_type(&self, t: &Arc<Type>) -> Arc<Type> {

        Arc::clone(t)
    }

    pub fn get_reduced_apparent_type(&self, t: &Arc<Type>) -> Arc<Type> {
        self.get_apparent_type(t)
    }

    pub fn resolve_structured_type_members(&self, t: &Arc<Type>) -> Arc<Type> {
        Arc::clone(t)
    }

    pub fn is_named_member(&self, _symbol: &Arc<Symbol>, _name: &str) -> bool {
        !is_reserved_member_name(_name)
    }

    pub fn get_named_members(
        &self,
        props_by_name: &std::collections::HashMap<String, Arc<Symbol>>,
    ) -> Vec<Arc<Symbol>> {
        props_by_name
            .values()
            .filter(|s| !is_reserved_member_name(&s.name))
            .cloned()
            .collect()
    }

    pub fn is_property_accessible(
        &self,
        _node: &Arc<Node>,
        _is_super: bool,
        _is_write: bool,
        _t: &Arc<Type>,
        _property: &Arc<Symbol>,
    ) -> bool {
        true
    }

    fn get_widened_type_of_expression(&mut self, expr: &Arc<Node>) -> Arc<Type> {
        let t = self.get_type_of_node(expr);
        self.get_widened_type(&t)
    }

    pub fn get_type_of_property_of_type(&mut self, t: &Arc<Type>, name: &str) -> Option<Arc<Type>> {
        if let Some(prop) = self.get_property_of_type(t, name) {
            return Some(self.get_type_of_symbol(&prop));
        }
        None
    }

    pub fn get_literal_type_from_property_name(
        &mut self,
        property_name: &Arc<Node>,
    ) -> Option<Arc<Type>> {
        match property_name.kind {
            SyntaxKind::StringLiteral => Some(self.get_string_literal_type(property_name.text())),
            SyntaxKind::NumericLiteral => {

                None
            }
            SyntaxKind::PrivateIdentifier => {

                None
            }
            SyntaxKind::ComputedPropertyName => {

                None
            }
            _ => None,
        }
    }

    pub fn is_this_type_parameter(&self, t: &Arc<Type>) -> bool {
        if !t.flags.contains(TypeFlags::TypeParameter) {
            return false;
        }
        if let TypeData::TypeParameter(tp) = &t.data {
            tp.is_this_type
        } else {
            false
        }
    }

    pub fn get_contextual_type_for_element_expression(
        &mut self,
        _contextual_type: &Arc<Type>,
        _element_index: usize,
        _length: Option<usize>,
        _first_spread_index: i32,
        _last_spread_index: i32,
    ) -> Option<Arc<Type>> {
        None
    }

    fn global_callable_function_type(&self) -> Option<Arc<Type>> {

        None
    }

    fn global_newable_function_type(&self) -> Option<Arc<Type>> {

        None
    }

    fn get_jsx_type_symbol(&self, _name: &str, _location: &Arc<Node>) -> Option<Arc<Type>> {
        None
    }

    pub fn get_source_file_of_node(&self, node: &Arc<Node>) -> Option<Arc<SourceFile>> {
        let mut current = Arc::clone(node);
        loop {
            if current.kind == SyntaxKind::SourceFile {

                let node_id = current.id();
                for file in &self.files {
                    if file.node.id() == node_id {
                        return Some(Arc::clone(file));
                    }
                }
                return None;
            }
            current = current.parent.clone()?;
        }
    }
}

fn get_possible_symbol_reference_nodes(
    source_file: &Arc<SourceFile>,
    symbol_name: &str,
    container: Option<&Arc<Node>>,
) -> Vec<Arc<Node>> {
    let positions = get_possible_symbol_reference_positions(source_file, symbol_name, container);
    let mut result = Vec::new();
    for pos in positions {

        if let Some(node) = find_identifier_at_pos(source_file, pos) {
            result.push(node);
        }
    }
    result
}

fn get_possible_symbol_reference_positions(
    source_file: &Arc<SourceFile>,
    symbol_name: &str,
    container: Option<&Arc<Node>>,
) -> Vec<usize> {
    let mut positions = Vec::new();

    if symbol_name.is_empty() {
        return positions;
    }

    let text = source_file.text.as_str();
    let symbol_name_len = symbol_name.len();

    let search_start = container.and_then(|c| Some(c.pos())).unwrap_or(0);
    let end_pos = container.and_then(|c| Some(c.end())).unwrap_or(text.len());

    let mut search_from = search_start;
    while search_from < end_pos {
        let remainder = &text[search_from..end_pos];
        let relative_pos = match remainder.find(symbol_name) {
            Some(p) => p,
            None => break,
        };
        let position = search_from + relative_pos;
        let end_position = position + symbol_name_len;

        let prev_ok = position == 0 || !is_identifier_part_byte(text.as_bytes()[position - 1]);
        let next_ok =
            end_position >= text.len() || !is_identifier_part_byte(text.as_bytes()[end_position]);

        if prev_ok && next_ok {
            positions.push(position);
        }

        search_from = position + symbol_name_len + 1;
        if search_from > text.len() {
            break;
        }
    }

    positions
}

fn is_identifier_part_byte(b: u8) -> bool {
    b.is_ascii_alphanumeric() || b == b'$' || b == b'_'
}

fn find_identifier_at_pos(source_file: &Arc<SourceFile>, pos: usize) -> Option<Arc<Node>> {

    let file_node = &source_file.node;
    find_node_at_pos(file_node, pos)
}

fn find_node_at_pos(node: &Arc<Node>, pos: usize) -> Option<Arc<Node>> {
    if node.pos() <= pos && pos < node.end() {
        if node.kind == SyntaxKind::Identifier && node.pos() == pos {
            return Some(Arc::clone(node));
        }

        let mut found = None;
        crate::ast::for_each_child(node, |child| {
            if found.is_none() {
                if let Some(f) = find_node_at_pos(child, pos) {
                    found = Some(Arc::clone(&f));
                    return true;
                }
            }
            false
        });
        return found;
    }
    None
}

fn is_array_literal_or_object_literal_destructuring_pattern(node: &Arc<Node>) -> bool {
    matches!(
        node.kind,
        SyntaxKind::ArrayLiteralExpression | SyntaxKind::ObjectLiteralExpression
    ) && node
        .parent
        .as_ref()
        .map(|p| {
            p.kind == SyntaxKind::BinaryExpression
                || p.kind == SyntaxKind::ForOfStatement
                || p.kind == SyntaxKind::VariableDeclaration
        })
        .unwrap_or(false)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IndexKind {
    String,
    Number,
}
