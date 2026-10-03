#![allow(unused_imports)]
use crate::checker::mig::m1f::r19k9_defs::R19K9NodeExt;
use crate::checker::mig::wc3::NodeAccessExt;

#[allow(unused_imports, ambiguous_glob_reexports)]
use crate::checker::*;
#[allow(unused_imports)]
use tsox_frontend::ast::*;
#[allow(unused_imports)]
use tsox_core::diagnostics::messages_generated::*;

pub(crate) use crate::checker::checker::*;
pub(crate) use super::m1f::*;
#[allow(unused_imports)]
use super::m1b::PredicateSemantics;
#[allow(unused_imports)]
use tsox_core::core::mig::m3j_2::{get_spelling_suggestion, get_spelling_suggestion_with_max_candidate_count};
#[allow(unused_imports)]
use tsox_frontend::ast::mig::m3g_2::is_right_side_of_qualified_name_or_property_access;
#[allow(unused_imports)]
use tsox_frontend::ast::mig::m3e_3::symbol_name;
#[allow(unused_imports)]
use tsox_core::core::compiler_options_kinds::JsxEmit;
#[allow(unused_imports)]
use tsox_frontend::ast::mig::m3g_3::{skip_outer_expressions, OuterExpressionKinds};
use std::sync::Arc;
use std::cmp::Ordering;
use crate::checker::mig::m2c::r18k3_defs::{get_string_literal_value, MappedTypeNameTypeKind};
use crate::checker::mig::m1f::r25k6_defs::map_type_with_checker;
use crate::checker::mig::wc3_3::is_generic_tuple_type;
use crate::checker::mig::m2a::r20k6_defs::R20K6CheckerExt;
use crate::checker::mig::m2a::r19k11_defs::R19K11NodeExt;
use crate::checker::checker_lib_feature_map::{
    suggested_lib_for_name, suggested_lib_for_property,
};

impl Checker {
    pub fn get_simplified_indexed_access_type(&mut self, t: &Arc<Type>, writing: bool) -> Arc<Type> { ::tsox_core::fntrace::enter("get_simplified_indexed_access_type"); 
        let key = CachedTypeKey {
            kind: if writing {
                CachedTypeKind::IndexedAccessForWriting
            } else {
                CachedTypeKind::IndexedAccessForReading
            },
            type_id: t.id,
        };
        if let Some(cached) = self.cached_types.get(&key) {
            return if cached.id == self.circular_constraint_type().id {
                t.clone()
            } else {
                cached.clone()
            };
        }
        self.cached_types.insert(key, t.clone());
        let mut result = self.get_simplified_indexed_access_type_worker(t, writing);
        if result.id != t.id {
            result = self.remove_type(&result, t);
            self.cached_types.insert(key, result.clone());
        }
        result
    }

    pub fn get_simplified_indexed_access_type_worker(
        &mut self,
        t: &Arc<Type>,
        writing: bool,
    ) -> Arc<Type> { ::tsox_core::fntrace::enter("get_simplified_indexed_access_type_worker"); 
        let (object_type, index_type) = match &t.data {
            TypeData::IndexedAccess(d) => (d.object_type.clone(), d.index_type.clone()),
            _ => return t.clone(),
        };
        let object_type = object_type.unwrap_or_else(|| t.clone());
        let index_type = index_type.unwrap_or_else(|| t.clone());
        let object_type = self.get_simplified_type(&object_type, writing);
        let index_type = self.get_simplified_type(&index_type, writing);
        if let Some(distributed) =
            self.distribute_object_over_index_type(&object_type, &index_type, writing)
        {
            return distributed;
        }
        if !index_type.flags.intersects(TypeFlags::STRUCTURED_OR_INSTANTIABLE) {
            if let Some(distributed) =
                self.distribute_index_over_object_type(&object_type, &index_type, writing)
            {
                return distributed;
            }
        }
        if is_generic_tuple_type(&object_type) && index_type.flags.intersects(TypeFlags::NUMBER_LIKE)
        {
            let fixed_length = object_type.target_tuple_type().map(|d| d.fixed_length).unwrap_or(0);
            let start = if index_type.flags.intersects(TypeFlags::Number) {
                0
            } else {
                fixed_length
            };
            if let Some(element_type) = self.get_element_type_of_slice_of_tuple_type(
                &object_type,
                start as i32,
                0,
                writing,
                false,
            ) {
                return element_type;
            }
        }
        if self.is_generic_mapped_type(&object_type)
            && self.get_mapped_type_name_type_kind(&object_type) != MappedTypeNameTypeKind::Remapping
        {
            let substituted = self.substitute_indexed_mapped_type(&object_type, &index_type);
            return map_type_with_checker(self, &substituted, &mut |c, st| {
                Some(c.get_simplified_type(st, writing))
            })
            .unwrap_or_else(|| substituted.clone());
        }
        t.clone()
    }

    pub fn get_simplified_type(&mut self, t: &Arc<Type>, writing: bool) -> Arc<Type> { ::tsox_core::fntrace::enter("get_simplified_type"); 
        if t.flags.intersects(TypeFlags::IndexedAccess) {
            return self.get_simplified_indexed_access_type(t, writing);
        }
        if t.flags.intersects(TypeFlags::Conditional) {
            return self.get_simplified_conditional_type(t, writing);
        }
        t.clone()
    }

    pub fn get_spelling_suggestion_for_name(
        &mut self,
        name: &str,
        symbols: Vec<Arc<Symbol>>,
        meaning: SymbolFlags,
    ) -> Option<Arc<Symbol>> { ::tsox_core::fntrace::enter("get_spelling_suggestion_for_name"); 
        let mut candidates: Vec<(Arc<Symbol>, String)> = Vec::new();
        for candidate in &symbols {
            let candidate_name = symbol_name(candidate);
            if candidate_name.is_empty()
                || candidate_name.starts_with('"')
                || candidate_name.starts_with('\u{FE}')
            {
                continue;
            }
            let mut matched = candidate.flags.intersects(meaning);
            if !matched && candidate.flags.intersects(SymbolFlags::Alias) {
                if let Some(alias) = self.try_resolve_alias(candidate) {
                    matched = alias.flags.intersects(meaning);
                }
            }
            if matched {
                candidates.push((candidate.clone(), candidate_name));
            }
        }
        get_spelling_suggestion(
            name,
            candidates.into_iter(),
            |(_, candidate_name)| candidate_name.clone(),
            |a, b| {
                let ord = self.compare_symbols(&a.0, &b.0);
                if ord < 0 {
                    Ordering::Less
                } else if ord > 0 {
                    Ordering::Greater
                } else {
                    Ordering::Equal
                }
            },
        )
        .map(|(symbol, _)| symbol)
    }

    pub fn get_suggested_import_extension(&self, extensionless_import_path: &str) -> String { ::tsox_core::fntrace::enter("get_suggested_import_extension"); 
        if self.program.file_exists(&format!("{extensionless_import_path}.mts")) {
            return ".mjs".to_string();
        }
        if self.program.file_exists(&format!("{extensionless_import_path}.ts")) {
            return ".js".to_string();
        }
        if self.program.file_exists(&format!("{extensionless_import_path}.cts")) {
            return ".cjs".to_string();
        }
        if self.program.file_exists(&format!("{extensionless_import_path}.mjs")) {
            return ".mjs".to_string();
        }
        if self.program.file_exists(&format!("{extensionless_import_path}.js")) {
            return ".js".to_string();
        }
        if self.program.file_exists(&format!("{extensionless_import_path}.cjs")) {
            return ".cjs".to_string();
        }
        if self.program.file_exists(&format!("{extensionless_import_path}.tsx")) {
            return if self.compiler_options.jsx == JsxEmit::Preserve {
                ".jsx".to_string()
            } else {
                ".js".to_string()
            };
        }
        if self.program.file_exists(&format!("{extensionless_import_path}.jsx")) {
            return ".jsx".to_string();
        }
        if self.program.file_exists(&format!("{extensionless_import_path}.json")) {
            return ".json".to_string();
        }
        "".to_string()
    }

    pub fn get_suggested_lib_for_non_existent_name(&self, name: &str) -> String { ::tsox_core::fntrace::enter("get_suggested_lib_for_non_existent_name"); 
        suggested_lib_for_name(name).unwrap_or_default().to_string()
    }

    pub fn get_suggested_lib_for_non_existent_property(
        &self,
        missing_property: &str,
        containing_type: &Arc<Type>,
    ) -> String { ::tsox_core::fntrace::enter("get_suggested_lib_for_non_existent_property"); 
        if let Some(container) = self.lib_suggestion_container_name(containing_type) {
            if let Some(lib) = suggested_lib_for_property(&container, missing_property) {
                return lib.to_string();
            }
        }
        "".to_string()
    }

    pub fn get_suggested_symbol_for_nonexistent_module(
        &mut self,
        name: &Node,
        target_module: &Arc<Symbol>,
    ) -> Option<Arc<Symbol>> { ::tsox_core::fntrace::enter("get_suggested_symbol_for_nonexistent_module"); 
        let exports = self.get_exports_of_module(target_module);
        self.get_spelling_suggestion_for_name(name.text(), exports, SymbolFlags::MODULE_MEMBER)
    }

    pub fn get_suggested_symbol_for_nonexistent_property(
        &mut self,
        name: &Node,
        containing_type: &Arc<Type>,
    ) -> Option<Arc<Symbol>> { ::tsox_core::fntrace::enter("get_suggested_symbol_for_nonexistent_property"); 
        let mut props = self.get_properties_of_type(containing_type);
        if let Some(parent) = name.parent() {
            if is_property_access_expression(&parent) {
                props.retain(|prop| {
                    self.is_valid_property_access_for_completions(&parent, containing_type, prop)
                });
            }
        }
        self.get_spelling_suggestion_for_name(name.text(), props, SymbolFlags::VALUE)
    }

    pub fn get_suggested_symbol_for_nonexistent_symbol(
        &mut self,
        location: &Arc<Node>,
        outer_name: &str,
        meaning: SymbolFlags,
    ) -> Option<Arc<Symbol>> { ::tsox_core::fntrace::enter("get_suggested_symbol_for_nonexistent_symbol"); 
        let mut resolver = self.create_name_resolver_for_suggestion();
        resolver.resolve(location, outer_name, meaning, None, false, false)
    }

    pub fn get_suggested_type_for_nonexistent_string_literal_type(
        &self,
        source: &Arc<Type>,
        target: &Arc<Type>,
    ) -> Option<Arc<Type>> { ::tsox_core::fntrace::enter("get_suggested_type_for_nonexistent_string_literal_type"); 
        let candidates: Vec<Arc<Type>> = target
            .types()
            .unwrap_or(&[])
            .iter()
            .filter(|t| t.flags.intersects(TypeFlags::StringLiteral))
            .cloned()
            .collect();
        let source_value = get_string_literal_value(source);
        get_spelling_suggestion_with_max_candidate_count(
            &source_value,
            candidates.into_iter(),
            |t| get_string_literal_value(t),
            |a, b| compare_types(a, b),
            1000,
        )
    }

    pub fn get_suggestion_for_nonexistent_index_signature(
        &mut self,
        object_type: &Arc<Type>,
        expr: &Arc<Node>,
        keyed_type: &Arc<Type>,
    ) -> String { ::tsox_core::fntrace::enter("get_suggestion_for_nonexistent_index_signature"); 
        let has_prop = |c: &mut Checker, name: &str| -> bool {
            if let Some(prop) = c.get_property_of_object_type(object_type, name) {
                let t = c.get_type_of_symbol(&prop);
                if let Some(s) = c.get_single_call_signature(&t) {
                    if c.get_min_argument_count(&s) < 1 {
                        return false;
                    }
                    let first_param = c.get_type_at_position(&s, 0);
                    return c.is_type_assignable_to(keyed_type, &first_param);
                }
            }
            false
        };
        let suggested_method = if is_assignment_target(expr) { "set" } else { "get" };
        if !has_prop(self, suggested_method) {
            return "".to_string();
        }
        let suggestion = expr
            .expression()
            .map(|e| try_get_property_access_or_identifier_to_string(e))
            .unwrap_or_default();
        if suggestion.is_empty() {
            return suggested_method.to_string();
        }
        format!("{suggestion}.{suggested_method}")
    }

    pub fn get_suggestion_for_symbol_name_lookup(
        &mut self,
        symbols: &SymbolTable,
        name: &str,
        meaning: SymbolFlags,
    ) -> Option<Arc<Symbol>> { ::tsox_core::fntrace::enter("get_suggestion_for_symbol_name_lookup"); 
        if let Some(symbol) = self.get_symbol(symbols, name, meaning) {
            return Some(symbol);
        }
        let mut candidates: Vec<Arc<Symbol>> = symbols
            .iter()
            .map(|(_, symbol)| symbol.clone())
            .collect();
        if meaning.intersects(SymbolFlags::GlobalLookup) {
            candidates.extend(get_primitive_type_alias_suggestions(symbols));
        }
        self.get_spelling_suggestion_for_name(name, candidates, meaning)
    }

    pub fn get_symbol(
        &mut self,
        symbols: &SymbolTable,
        name: &str,
        meaning: SymbolFlags,
    ) -> Option<Arc<Symbol>> { ::tsox_core::fntrace::enter("get_symbol"); 
        if meaning.intersects(SymbolFlags::all()) {
            if let Some(symbol) = symbols.get(name) {
                let symbol = self.get_merged_symbol(symbol);
                if symbol.flags.intersects(meaning) {
                    return Some(symbol);
                }
                if symbol.flags.intersects(SymbolFlags::Alias) {
                    let target_flags = self.get_symbol_flags(&symbol);
                    if target_flags.intersects(meaning) {
                        return Some(symbol);
                    }
                }
            }
        }
        None
    }

    pub fn get_symbol_flags_ex(
        &mut self,
        symbol: &Arc<Symbol>,
        exclude_type_only_meanings: bool,
        exclude_local_meanings: bool,
    ) -> SymbolFlags { ::tsox_core::fntrace::enter("get_symbol_flags_ex"); 
        let mut seen_symbols: HashSet<*const Symbol> = HashSet::new();
        let mut flags = if !exclude_local_meanings {
            symbol.flags
        } else {
            SymbolFlags::None
        };
        let mut symbol = symbol.clone();
        while symbol.flags.intersects(SymbolFlags::Alias) {
            if exclude_type_only_meanings && self.get_type_only_alias_declaration(&symbol).is_some()
            {
                break;
            }
            let resolved = self.resolve_alias(&symbol);
            let target = self.get_export_symbol_of_value_symbol_if_exported(&resolved);
            if target.id() == self.unknown_symbol().id() {
                return SymbolFlags::all();
            }
            if target.flags.intersects(SymbolFlags::Alias) {
                if Arc::ptr_eq(&target, &symbol) || seen_symbols.contains(&Arc::as_ptr(&target)) {
                    break;
                }
                if seen_symbols.is_empty() {
                    seen_symbols.insert(Arc::as_ptr(&symbol));
                }
                seen_symbols.insert(Arc::as_ptr(&target));
            }
            flags |= target.flags;
            symbol = target;
        }
        flags
    }

    pub fn get_symbol_for_private_identifier_expression(
        &mut self,
        node: &Arc<Node>,
    ) -> Option<Arc<Symbol>> { ::tsox_core::fntrace::enter("get_symbol_for_private_identifier_expression"); 
        let existing = self
            .symbol_node_links
            .get(node)
            .and_then(|l| l.resolved_symbol.clone());
        if let Some(existing) = existing {
            return Some(existing);
        }
        let resolved = self.lookup_symbol_for_private_identifier_declaration(node.text(), node);
        self.symbol_node_links
            .get_mut(node)
            .unwrap()
            .resolved_symbol = resolved.clone();
        resolved
    }

    pub fn get_symbol_from_type_reference(&mut self, node: &Arc<Node>) -> Option<Arc<Symbol>> { ::tsox_core::fntrace::enter("get_symbol_from_type_reference"); 
        let existing = self
            .symbol_node_links
            .get(node)
            .and_then(|l| l.resolved_symbol.clone());
        if let Some(existing) = existing {
            return Some(existing);
        }
        let resolved = self.resolve_type_reference_name(node, SymbolFlags::TYPE, false);
        self.symbol_node_links
            .get_mut(node)
            .unwrap()
            .resolved_symbol = resolved.clone();
        resolved
    }

    pub fn get_symbol_of_part_of_right_hand_side_of_import_equals(
        &mut self,
        entity_name: &Arc<Node>,
    ) -> Option<Arc<Symbol>> { ::tsox_core::fntrace::enter("get_symbol_of_part_of_right_hand_side_of_import_equals"); 
        let mut entity_name = Arc::clone(entity_name);
        if entity_name.kind == SyntaxKind::Identifier
            && is_right_side_of_qualified_name_or_property_access(&entity_name)
        {
            if let Some(parent) = entity_name.parent() {
                entity_name = parent;
            }
        }
        let parent_is_qualified = entity_name
            .parent()
            .map(|p| p.kind == SyntaxKind::QualifiedName)
            .unwrap_or(false);
        if entity_name.kind == SyntaxKind::Identifier || parent_is_qualified {
            return self.resolve_entity_name(&entity_name, SymbolFlags::NAMESPACE, false, true, None);
        }
        self.resolve_entity_name(
            &entity_name,
            SymbolFlags::VALUE | SymbolFlags::TYPE | SymbolFlags::NAMESPACE,
            false,
            true,
            None,
        )
    }

    pub fn get_syntactic_nullishness_semantics(&self, node: &Arc<Node>) -> PredicateSemantics { ::tsox_core::fntrace::enter("get_syntactic_nullishness_semantics"); 
        let node = skip_outer_expressions(node, OuterExpressionKinds::ALL);
        match node.kind {
            SyntaxKind::AwaitExpression
            | SyntaxKind::CallExpression
            | SyntaxKind::TaggedTemplateExpression
            | SyntaxKind::ElementAccessExpression
            | SyntaxKind::MetaProperty
            | SyntaxKind::NewExpression
            | SyntaxKind::PropertyAccessExpression
            | SyntaxKind::YieldExpression
            | SyntaxKind::ThisKeyword => PredicateSemantics::Sometimes,
            SyntaxKind::BinaryExpression => {
                let expr = node.as_binary_expression();
                match expr.operator_token.kind {
                    SyntaxKind::BarBarToken
                    | SyntaxKind::BarBarEqualsToken
                    | SyntaxKind::AmpersandAmpersandToken
                    | SyntaxKind::AmpersandAmpersandEqualsToken => PredicateSemantics::Sometimes,
                    SyntaxKind::CommaToken | SyntaxKind::EqualsToken => {
                        self.get_syntactic_nullishness_semantics(&expr.right)
                    }
                    SyntaxKind::QuestionQuestionToken | SyntaxKind::QuestionQuestionEqualsToken => {
                        let left_semantics =
                            self.get_syntactic_nullishness_semantics(&expr.left);
                        let mut result = left_semantics & PredicateSemantics::Never;
                        if left_semantics.intersects(PredicateSemantics::Always) {
                            result = result
                                | self.get_syntactic_nullishness_semantics(&expr.right);
                        }
                        result
                    }
                    _ => PredicateSemantics::Never,
                }
            }
            SyntaxKind::ConditionalExpression => {
                let expr = node.as_conditional_expression();
                self.get_syntactic_nullishness_semantics(&expr.when_true)
                    | self.get_syntactic_nullishness_semantics(&expr.when_false)
            }
            SyntaxKind::NullKeyword => PredicateSemantics::Always,
            SyntaxKind::Identifier => {
                if let Some(resolved) = self.get_resolved_symbol(&node) {
                    if self
                        .undefined_symbol
                        .as_ref()
                        .is_some_and(|u| resolved.id() == u.id())
                    {
                        return PredicateSemantics::Always;
                    }
                }
                PredicateSemantics::Sometimes
            }
            _ => PredicateSemantics::Never,
        }
    }
}

pub(crate) fn get_symbol_path(symbol: &Symbol) -> String { ::tsox_core::fntrace::enter("get_symbol_path"); 
    match symbol.parent().as_ref() {
        Some(parent) => format!("{}.{}", get_symbol_path(parent), symbol.name),
        None => symbol.name.clone(),
    }
}
