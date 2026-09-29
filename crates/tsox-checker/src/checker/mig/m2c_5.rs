#![allow(unused_imports)]

use crate::checker::checker::*;
use std::sync::Arc;
use tsox_frontend::ast::node_data_generated::{
    is_get_accessor_declaration, is_identifier, is_import_declaration,
    is_import_equals_declaration, is_property_access_expression, is_qualified_name,
    is_set_accessor_declaration,
};
use tsox_frontend::ast::mig::m3e_4::get_first_identifier;
use tsox_frontend::ast::mig::m3g_2::{
    is_parse_tree_node, is_type_only_import_or_export_declaration,
};
use tsox_frontend::ast::mig::w7a::is_expando_property_declaration;
use crate::checker::mig::m1e::r20k2_defs::{
    get_source_file_of_node, has_syntactic_modifier, node_is_missing,
};
use crate::checker::mig::wc3::NodeAccessExt;

use super::m2c::r18k3_defs::{Flags, InternalFlags};
use super::m2c::r19k8_defs::{
    new_big_int_literal, new_identifier, new_keyword_expression, new_keyword_type_node,
    new_numeric_literal, new_prefix_unary_expression, new_string_literal,
    pseudo_big_int_to_string, TYPE_FLAGS_BIGINT_LIKE, TYPE_FLAGS_ESSYMBOL_LIKE,
};
use super::wc1b::is_tuple_type;
use super::wc3::ReferenceHint;

use crate::binder::referenceresolver_reference_resolver::ReferenceResolver;
use crate::checker::symboltracker::{NodeBuilderFlags, NodeBuilderInternalFlags, SymbolTracker};
use super::m2f::{replace_modifiers, NodeFactoryExt};
use super::m2g::r22k9_defs::NodeBuilderPseudoExt22;

#[path = "r26k4_defs.rs"]
pub mod r26k4_defs;
pub use r26k4_defs::TypeReferenceSerializationKind;

impl Checker {
    pub fn create_late_bound_index_signatures(
        &mut self,
        container: &Arc<Node>,
        enclosing_declaration: &Arc<Node>,
        flags: NodeBuilderFlags,
        internal_flags: NodeBuilderInternalFlags,
        mut tracker: Option<Box<dyn SymbolTracker>>,
    ) -> Vec<Arc<Node>> {
        let Some(sym) = self.get_symbol_of_declaration(container) else {
            return Vec::new();
        };
        let t = self.get_type_of_symbol(&sym);
        let static_infos = self.get_index_infos_of_type(&t);
        let instance_index_symbol = self.get_index_symbol(&sym);
        let mut instance_infos: Vec<Arc<IndexInfo>> = Vec::new();
        if let Some(instance_index_symbol) = instance_index_symbol {
            let sibling_symbols: Vec<Arc<Symbol>> = self
                .get_members_of_symbol(&sym)
                .iter()
                .map(|(_, s)| s.clone())
                .collect();
            instance_infos =
                self.get_index_infos_of_index_symbol(&instance_index_symbol, &sibling_symbols);
        }
        let (mut builder, release) = self.get_node_builder();
        let mut result: Vec<Arc<Node>> = Vec::new();
        for (i, info_list) in [static_infos, instance_infos].iter().enumerate() {
            let is_static = i == 0;
            if info_list.is_empty() {
                continue;
            }
            for info in info_list {
                builder.enter_context(
                    Some(enclosing_declaration),
                    flags,
                    internal_flags,
                    tracker.take(),
                );
                let index_signature =
                    builder
                        .impl_
                        .index_info_to_index_signature_declaration_helper(info, None);
                let index_signature = builder.exit_context(index_signature);
                if let Some(index_signature) = index_signature {
                    if is_static {
                        let mut mod_nodes = vec![builder.impl_.f.new_modifier(SyntaxKind::StaticKeyword)];
                        mod_nodes.extend(index_signature.modifier_nodes().iter().cloned());
                        let mods = builder.impl_.f.new_modifier_list(mod_nodes);
                        result.push(replace_modifiers(&builder.impl_.f, &index_signature, mods));
                    } else {
                        result.push(index_signature);
                    }
                }
            }
        }
        release();
        result
    }

    pub fn create_literal_const_value(
        &mut self,
        node: &Arc<Node>,
        tracker: Option<Box<dyn SymbolTracker>>,
    ) -> Option<Arc<Node>> {
        let Some(node_sym) = self.get_symbol_of_declaration(node) else {
            return None;
        };
        let t = self.get_type_of_symbol(&node_sym);
        let mut enum_result: Option<Arc<Node>> = None;
        if t.flags.intersects(TYPE_FLAGS_ENUM_LIKE) {
            let symbol = t.symbol.clone()?;
            let (mut builder, release) = self.get_node_builder();
            enum_result = builder.symbol_to_expression(
                &symbol,
                SymbolFlags::VALUE,
                Some(node),
                NodeBuilderFlags::None,
                NodeBuilderInternalFlags::empty(),
                tracker,
            );
            release();
        } else if self.true_type.get().is_some_and(|tt| Arc::ptr_eq(&t, tt)) {
            enum_result = Some(new_keyword_expression(SyntaxKind::TrueKeyword));
        } else if self.false_type.get().is_some_and(|ft| Arc::ptr_eq(&t, ft)) {
            enum_result = Some(new_keyword_expression(SyntaxKind::FalseKeyword));
        }
        if enum_result.is_some() {
            return enum_result;
        }
        if !t.flags.intersects(TYPE_FLAGS_LITERAL) {
            return None;
        }
        let Some(literal) = t.as_literal_type() else {
            return None;
        };
        match &literal.value {
            LiteralValue::String(value) => Some(new_string_literal(value)),
            LiteralValue::Number(value) => {
                if value.is_inf() {
                    if value.0 > 0.0 {
                        return Some(new_identifier("Infinity"));
                    }
                    return Some(new_prefix_unary_expression(
                        SyntaxKind::MinusToken,
                        new_identifier("Infinity"),
                    ));
                }
                if value.is_nan() {
                    return Some(new_identifier("NaN"));
                }
                if value.abs() != *value {
                    let text = value.to_string();
                    return Some(new_prefix_unary_expression(
                        SyntaxKind::MinusToken,
                        new_numeric_literal(&text[1..]),
                    ));
                }
                Some(new_numeric_literal(&value.to_string()))
            }
            LiteralValue::BigInt(value) => {
                Some(new_big_int_literal(&format!("{}n", pseudo_big_int_to_string(value))))
            }
            LiteralValue::Boolean(value) => {
                let kind = if *value {
                    SyntaxKind::TrueKeyword
                } else {
                    SyntaxKind::FalseKeyword
                };
                Some(new_keyword_expression(kind))
            }
            LiteralValue::None => None,
        }
    }

    pub fn create_return_type_of_signature_declaration(
        &mut self,
        signature_declaration: &Arc<Node>,
        enclosing_declaration: &Arc<Node>,
        flags: NodeBuilderFlags,
        internal_flags: NodeBuilderInternalFlags,
        tracker: Option<Box<dyn SymbolTracker>>,
    ) -> Arc<Node> {
        let signature = self.get_signature_from_declaration(signature_declaration);
        let (mut builder, release) = self.get_node_builder();
        builder.enter_context(Some(enclosing_declaration), flags, internal_flags, tracker);
        let result = match &signature {
            Some(sig) => {
                let (_params, cleanup) = builder.impl_.enter_signature_scope(sig);
                let r = builder.impl_.serialize_return_type_for_signature(sig, true);
                cleanup();
                Some(r)
            }
            None => None,
        };
        let out = builder.exit_context(result);
        release();
        out.unwrap_or_else(|| new_keyword_type_node(SyntaxKind::AnyKeyword))
    }

    pub fn create_type_of_declaration(
        &mut self,
        declaration: &Arc<Node>,
        enclosing_declaration: &Arc<Node>,
        flags: NodeBuilderFlags,
        internal_flags: NodeBuilderInternalFlags,
        tracker: Option<Box<dyn SymbolTracker>>,
    ) -> Arc<Node> {
        let symbol = self.get_symbol_of_declaration(declaration);
        let t = match &symbol {
            Some(symbol) => self.get_type_of_symbol(symbol),
            None => self.unknown_type(),
        };
        let (mut builder, release) = self.get_node_builder();
        builder.enter_context(
            Some(enclosing_declaration),
            flags | NodeBuilderFlags::MultilineObjectLiterals,
            internal_flags,
            tracker,
        );
        let result = builder.impl_.serialize_type_for_declaration(
            Some(declaration),
            &t,
            symbol.as_ref(),
            false,
        );
        let out = builder.exit_context(Some(result));
        release();
        out.unwrap_or_else(|| new_keyword_type_node(SyntaxKind::AnyKeyword))
    }

    pub fn create_type_of_expression(
        &mut self,
        expression: &Arc<Node>,
        enclosing_declaration: &Arc<Node>,
        flags: NodeBuilderFlags,
        internal_flags: NodeBuilderInternalFlags,
        tracker: Option<Box<dyn SymbolTracker>>,
    ) -> Arc<Node> {
        let (mut builder, release) = self.get_node_builder();
        builder.enter_context(
            Some(enclosing_declaration),
            flags | NodeBuilderFlags::MultilineObjectLiterals,
            internal_flags,
            tracker,
        );
        let result = builder.impl_.serialize_type_for_expression(expression);
        let out = builder.exit_context(Some(result));
        release();
        out.unwrap_or_else(|| new_keyword_type_node(SyntaxKind::AnyKeyword))
    }

    pub fn create_type_parameters_of_signature_declaration(
        &mut self,
        signature_declaration: &Arc<Node>,
        enclosing_declaration: &Arc<Node>,
        flags: NodeBuilderFlags,
        internal_flags: NodeBuilderInternalFlags,
        tracker: Option<Box<dyn SymbolTracker>>,
    ) -> Vec<Arc<Node>> {
        let Some(symbol) = self.get_symbol_of_declaration(signature_declaration) else {
            return Vec::new();
        };
        let (mut builder, release) = self.get_node_builder();
        let result = builder.symbol_to_type_parameter_declarations(
            &symbol,
            Some(enclosing_declaration),
            flags,
            internal_flags,
            tracker,
        );
        release();
        result.unwrap_or_default()
    }

    pub fn get_element_access_expression_name(&mut self, expression: &Arc<Node>) -> String {
        let mut resolver = self.get_emit_resolver();
        let reference_resolver = Arc::get_mut(&mut resolver)
            .expect("freshly created emit resolver is uniquely owned")
            .get_reference_resolver(self);
        reference_resolver.get_element_access_expression_name(expression)
    }

    pub fn get_properties_of_container_function(
        &mut self,
        node: Option<&Arc<Node>>,
    ) -> Vec<Arc<Symbol>> {
        let Some(node) = node else {
            return Vec::new();
        };
        let Some(s) = self.get_symbol_of_declaration(node) else {
            return Vec::new();
        };
        let t = self.get_type_of_symbol(&s);
        self.get_properties_of_type(&t)
    }

    pub fn get_referenced_export_container(
        &mut self,
        node: &Arc<Node>,
        prefix_locals: bool,
    ) -> Option<Arc<Node>> {
        let mut resolver = self.get_emit_resolver();
        let reference_resolver = Arc::get_mut(&mut resolver)
            .expect("freshly created emit resolver is uniquely owned")
            .get_reference_resolver(self);
        reference_resolver.get_referenced_export_container(node, prefix_locals)
    }

    pub fn get_referenced_import_declaration(
        &mut self,
        node: &Arc<Node>,
    ) -> Option<Arc<Node>> {
        if !is_parse_tree_node(node) {
            return r26k4_defs::jsx_links(node).import_ref;
        }
        let symbol = self.get_referenced_value_or_alias_symbol(node);
        if let Some(symbol) = symbol {
            if is_non_local_alias(&symbol, SymbolFlags::VALUE)
                && self
                    .get_type_only_alias_declaration_ex(&symbol, SymbolFlags::VALUE)
                    .is_none()
            {
                return self.get_declaration_of_alias_symbol(&symbol);
            }
        }
        None
    }

    pub fn set_referenced_import_declaration(
        &mut self,
        node: &Arc<Node>,
        import_ref: Option<Arc<Node>>,
    ) {
        r26k4_defs::set_jsx_links_import_ref(node, import_ref);
    }

    pub fn get_referenced_member_value_declaration(
        &mut self,
        node: &Arc<Node>,
    ) -> Option<Arc<Node>> {
        let mut resolver = self.get_emit_resolver();
        let reference_resolver = Arc::get_mut(&mut resolver)
            .expect("freshly created emit resolver is uniquely owned")
            .get_reference_resolver(self);
        reference_resolver.get_referenced_member_value_declaration(node)
    }

    pub fn get_referenced_value_declaration(
        &mut self,
        node: &Arc<Node>,
    ) -> Option<Arc<Node>> {
        if !is_parse_tree_node(node) {
            return None;
        }
        let mut resolver = self.get_emit_resolver();
        let reference_resolver = Arc::get_mut(&mut resolver)
            .expect("freshly created emit resolver is uniquely owned")
            .get_reference_resolver(self);
        reference_resolver.get_referenced_value_declaration(node)
    }

    pub fn get_referenced_value_declaration_unsafe(
        &mut self,
        node: &Arc<Node>,
    ) -> Option<Arc<Node>> {
        let mut resolver = self.get_emit_resolver();
        let reference_resolver = Arc::get_mut(&mut resolver)
            .expect("freshly created emit resolver is uniquely owned")
            .get_reference_resolver(self);
        reference_resolver.get_referenced_value_declaration(node)
    }

    pub fn get_referenced_value_declarations(
        &mut self,
        node: &Arc<Node>,
    ) -> Vec<Arc<Node>> {
        if !is_parse_tree_node(node) {
            return Vec::new();
        }
        let mut resolver = self.get_emit_resolver();
        let reference_resolver = Arc::get_mut(&mut resolver)
            .expect("freshly created emit resolver is uniquely owned")
            .get_reference_resolver(self);
        reference_resolver.get_referenced_value_declarations(node)
    }

    pub fn get_type_reference_serialization_kind(
        &mut self,
        type_name: &Arc<Node>,
        location: &Arc<Node>,
    ) -> TypeReferenceSerializationKind {
        let mut is_type_only = false;
        if is_qualified_name(type_name) {
            let root_value_symbol = self.resolve_entity_name(
                &get_first_identifier(type_name),
                SymbolFlags::VALUE,
                true,
                true,
                Some(location),
            );
            if let Some(root_value_symbol) = &root_value_symbol {
                if !root_value_symbol.declarations.is_empty() {
                    is_type_only = root_value_symbol
                        .declarations
                        .iter()
                        .all(|d| is_type_only_import_or_export_declaration(d));
                }
            }
        }
        let value_symbol =
            self.resolve_entity_name(type_name, SymbolFlags::VALUE, true, true, Some(location));
        let resolved_value_symbol = match &value_symbol {
            Some(value_symbol) if value_symbol.flags.intersects(SymbolFlags::Alias) => {
                Some(self.resolve_alias(value_symbol))
            }
            other => other.clone(),
        };
        is_type_only = is_type_only
            || value_symbol.as_ref().is_some_and(|s| {
                self.get_type_only_alias_declaration_ex(s, SymbolFlags::VALUE)
                    .is_some()
            });
        let type_symbol =
            self.resolve_entity_name(type_name, SymbolFlags::TYPE, true, true, Some(location));
        let resolved_type_symbol = match &type_symbol {
            Some(type_symbol) if type_symbol.flags.intersects(SymbolFlags::Alias) => {
                Some(self.resolve_alias(type_symbol))
            }
            other => other.clone(),
        };
        is_type_only = is_type_only
            || type_symbol.as_ref().is_some_and(|s| {
                self.get_type_only_alias_declaration_ex(s, SymbolFlags::TYPE)
                    .is_some()
            });
        if let Some(resolved_value_symbol) = &resolved_value_symbol {
            if resolved_type_symbol
                .as_ref()
                .is_some_and(|t| Arc::ptr_eq(t, resolved_value_symbol))
            {
                let global_promise_symbol = crate::checker::mig::m1c::r25k9_defs::Checker::get_global_promise_constructor_symbol_or_nil(self);
                if global_promise_symbol
                    .as_ref()
                    .is_some_and(|p| Arc::ptr_eq(p, resolved_value_symbol))
                {
                    return TypeReferenceSerializationKind::Promise;
                }
                let constructor_type = self.get_type_of_symbol(resolved_value_symbol);
                if self.is_constructor_type(&constructor_type) {
                    if is_type_only {
                        return TypeReferenceSerializationKind::TypeWithCallSignature;
                    }
                    return TypeReferenceSerializationKind::TypeWithConstructSignatureAndValue;
                }
            }
        }
        let Some(resolved_type_symbol) = resolved_type_symbol else {
            if is_type_only {
                return TypeReferenceSerializationKind::ObjectType;
            }
            return TypeReferenceSerializationKind::Unknown;
        };
        let type_ = self.get_declared_type_of_symbol(&resolved_type_symbol);
        if self.is_error_type(&type_) {
            if is_type_only {
                return TypeReferenceSerializationKind::ObjectType;
            }
            return TypeReferenceSerializationKind::Unknown;
        }
        if type_.flags.intersects(TYPE_FLAGS_ANY_OR_UNKNOWN) {
            TypeReferenceSerializationKind::ObjectType
        } else if self.is_type_assignable_to_kind(&type_, TypeFlags::Void | TYPE_FLAGS_NULLABLE | TypeFlags::Never) {
            TypeReferenceSerializationKind::VoidNullableOrNeverType
        } else if self.is_type_assignable_to_kind(&type_, TYPE_FLAGS_BOOLEAN_LIKE) {
            TypeReferenceSerializationKind::BooleanType
        } else if self.is_type_assignable_to_kind(&type_, TYPE_FLAGS_NUMBER_LIKE) {
            TypeReferenceSerializationKind::NumberLikeType
        } else if self.is_type_assignable_to_kind(&type_, TYPE_FLAGS_BIGINT_LIKE) {
            TypeReferenceSerializationKind::BigIntLikeType
        } else if self.is_type_assignable_to_kind(&type_, TYPE_FLAGS_STRING_LIKE) {
            TypeReferenceSerializationKind::StringLikeType
        } else if is_tuple_type(&type_) {
            TypeReferenceSerializationKind::ArrayLikeType
        } else if self.is_type_assignable_to_kind(&type_, TYPE_FLAGS_ESSYMBOL_LIKE) {
            TypeReferenceSerializationKind::ESSymbolType
        } else if self.is_function_type(&type_) {
            TypeReferenceSerializationKind::TypeWithCallSignature
        } else if self.is_array_type(&type_) {
            TypeReferenceSerializationKind::ArrayLikeType
        } else {
            TypeReferenceSerializationKind::ObjectType
        }
    }

    pub fn is_definitely_reference_to_global_symbol_object(&mut self, node: &Arc<Node>) -> bool {
        let Some(name) = node.name() else {
            return false;
        };
        let Some(expression) = node.expression() else {
            return false;
        };
        if !is_property_access_expression(node)
            || !is_identifier(name)
            || (!is_property_access_expression(expression) && !is_identifier(expression))
        {
            return false;
        }
        if expression.kind == SyntaxKind::Identifier {
            if expression.text() != "Symbol" {
                return false;
            }
            let resolved = self.get_resolved_symbol(expression);
            let global = self.get_global_symbol(
                "Symbol",
                SymbolFlags::VALUE | SymbolFlags::ExportValue,
                None,
            );
            return resolved.is_some_and(|r| global.is_some_and(|g| Arc::ptr_eq(&r, &g)));
        }
        let inner = expression;
        let Some(inner_expression) = inner.expression() else {
            return false;
        };
        let Some(inner_name) = inner.name() else {
            return false;
        };
        if inner_expression.kind != SyntaxKind::Identifier
            || inner_expression.text() != "globalThis"
            || inner_name.text() != "Symbol"
        {
            return false;
        }
        let resolved = self.get_resolved_symbol(inner_expression);
        resolved.is_some_and(|r| {
            self.global_this_symbol
                .as_ref()
                .is_some_and(|g| Arc::ptr_eq(&r, g))
        })
    }

    pub fn is_expando_function_declaration(&mut self, node: &Arc<Node>) -> bool {
        self.is_expando_function_declaration_unsafe(node)
    }

    pub fn is_expando_function_declaration_unsafe(&mut self, node: &Arc<Node>) -> bool {
        if !is_parse_tree_node(node) {
            return false;
        }
        let props = self.get_properties_of_container_function(Some(node));
        props
            .iter()
            .any(|p| {
                p.value_declaration
                    .as_deref()
                    .is_some_and(|d| is_expando_property_declaration(Some(d)))
            })
    }

    pub fn is_implementation_of_overload(&mut self, node: &Arc<Node>) -> bool {
        if node.body().is_some() {
            if is_get_accessor_declaration(node) || is_set_accessor_declaration(node) {
                return false;
            }
            let Some(symbol) = self.get_symbol_of_declaration(node) else {
                return false;
            };
            let signatures_of_symbol = self.get_signatures_of_symbol(Some(&symbol));
            if signatures_of_symbol.len() > 1 {
                return true;
            }
            if signatures_of_symbol.len() == 1 {
                let signature = &signatures_of_symbol[0];
                let full_signature_type = self.get_signature_of_full_signature_type(node);
                if full_signature_type
                    .as_ref()
                    .is_some_and(|s| Arc::ptr_eq(s, signature))
                {
                    return false;
                }
                if let Some(declaration) = &signature.declaration {
                    if !Arc::ptr_eq(declaration, node)
                        && !declaration.flags.intersects(NodeFlags::JSDoc)
                    {
                        return true;
                    }
                }
            }
        }
        false
    }

    pub fn is_import_required_by_augmentation(&mut self, decl: &Arc<Node>) -> bool {
        let Some(file) = get_source_file_of_node(decl) else {
            return false;
        };
        let Some(file_symbol) = self.get_symbol_of_declaration(&file) else {
            return false;
        };
        let Some(import_target) = self.get_external_module_file_from_declaration(decl) else {
            return false;
        };
        if Arc::ptr_eq(&import_target, &file) {
            return false;
        }
        let exports = self.get_exports_of_module(&file_symbol);
        for s in exports.iter() {
            let merged = self.get_merged_symbol(s);
            if !Arc::ptr_eq(&merged, s) && !merged.declarations.is_empty() {
                for d in &merged.declarations {
                    let decl_file = get_source_file_of_node(d);
                    if decl_file
                        .as_ref()
                        .is_some_and(|f| Arc::ptr_eq(f, &import_target))
                    {
                        return true;
                    }
                }
            }
        }
        false
    }

    pub fn is_late_bound(&mut self, node: &Arc<Node>) -> bool {
        let Some(symbol) = self.get_symbol_of_declaration_opt(node) else {
            return false;
        };
        symbol.check_flags.intersects(CheckFlags::Late)
    }

    pub fn is_name_resolvable(&mut self, location: &Arc<Node>, name: &str) -> bool {
        self.resolve_name(name, location, SymbolFlags::VALUE | SymbolFlags::TYPE | SymbolFlags::NAMESPACE, false)
        .is_some()
    }

    pub fn is_this_property_assignment_declaration_redundant(
        &mut self,
        node: &Arc<Node>,
    ) -> bool {
        let Some(s) = self.get_symbol_of_declaration_opt(node) else {
            return false;
        };
        let Some(parent) = s.parent() else {
            return false;
        };
        let parent_type = self.get_declared_type_of_symbol(&parent);
        for base in self.get_base_types(&parent_type) {
            let Some(base_prop) = self.get_property_of_type(&base, &s.name) else {
                continue;
            };
            if base_prop
                .flags
                .intersects(SymbolFlags::ACCESSOR | SymbolFlags::Method | SymbolFlags::Function)
            {
                return true;
            }
            let base_prop_readonly = self.is_readonly_symbol(&base_prop);
            if base_prop_readonly == self.is_readonly_symbol(&s)
                && s.flags.intersects(SymbolFlags::Optional)
                    == base_prop.flags.intersects(SymbolFlags::Optional)
            {
                let s_type = self.get_type_of_symbol(&s);
                let base_prop_type = self.get_type_of_symbol(&base_prop);
                if self.is_type_identical_to(&s_type, &base_prop_type) {
                    return true;
                }
            }
        }
        false
    }

    pub fn is_top_level_value_import_equals_with_entity_name(
        &mut self,
        node: &Arc<Node>,
    ) -> bool {
        if !self.can_collect_symbol_alias_accessibility_data {
            return true;
        }
        if !is_parse_tree_node(node)
            || node.kind != SyntaxKind::ImportEqualsDeclaration
            || !node.parent().is_some_and(|p| p.kind == SyntaxKind::SourceFile)
        {
            return false;
        }
        if is_import_equals_declaration(node)
            && (node_is_missing(Some(
                &node.as_import_equals_declaration().module_reference,
            )) || node.as_import_equals_declaration().module_reference.kind
                == SyntaxKind::ExternalModuleReference)
        {
            return false;
        }
        let symbol = self.get_symbol_of_declaration(node);
        self.get_emit_resolver().is_alias_resolved_to_value(self, symbol.as_ref(), false)
    }

    pub fn mark_linked_references_recursively(&mut self, file: &Arc<Node>) {
        if !is_parse_tree_node(file) {
            return;
        }
        file.for_each_child(|n| {
            self.mark_linked_references_recursively_visit(n);
            true
        });
    }

    fn mark_linked_references_recursively_visit(&mut self, n: &Arc<Node>) {
        if is_import_equals_declaration(n)
            && !has_syntactic_modifier(n, ModifierFlags::Export)
        {
            return;
        }
        if is_import_declaration(n) {
            return;
        }
        self.mark_linked_references(n, ReferenceHint::Unspecified, None, None);
        n.for_each_child(|child| {
            self.mark_linked_references_recursively_visit(child);
            true
        });
    }

    pub fn requires_adding_implicit_undefined_ex(
        &mut self,
        declaration: &Arc<Node>,
        symbol: &Arc<Symbol>,
        enclosing_declaration: &Arc<Node>,
    ) -> bool {
        if !is_parse_tree_node(declaration) {
            return false;
        }
        self.get_emit_resolver().requires_adding_implicit_undefined(
            self,
            declaration,
            Some(symbol),
            Some(enclosing_declaration),
        )
    }

    pub fn requires_adding_implicit_undefined_unsafe(
        &mut self,
        declaration: &Arc<Node>,
        symbol: &Arc<Symbol>,
        enclosing_declaration: &Arc<Node>,
    ) -> bool {
        if !is_parse_tree_node(declaration) {
            return false;
        }
        self.get_emit_resolver().requires_adding_implicit_undefined(
            self,
            declaration,
            Some(symbol),
            Some(enclosing_declaration),
        )
    }

    pub fn try_js_type_node_to_type_node(
        &mut self,
        type_node: &Arc<Node>,
        enclosing_declaration: &Arc<Node>,
        flags: NodeBuilderFlags,
        internal_flags: NodeBuilderInternalFlags,
        tracker: Option<Box<dyn SymbolTracker>>,
    ) -> Option<Arc<Node>> {
        let (mut builder, release) = self.get_node_builder();
        let result = builder.try_js_type_node_to_type_node(
            type_node,
            Some(enclosing_declaration),
            flags,
            internal_flags,
            tracker,
        );
        release();
        result
    }
}
