#![allow(unused_imports)]

use crate::checker::checker::*;
use crate::checker::inference_inference_key_2::InferenceFlags;
use crate::checker::mapper::new_simple_type_mapper;
use std::sync::Arc;
use tsox_frontend::ast::mig::m3e::{get_function_flags, FunctionFlags};
use tsox_frontend::ast::mig::m3f::get_reparsed_node_for_node;
use tsox_frontend::ast::node_data_generated::{
    is_binding_element, is_expression_statement, is_export_declaration, is_export_specifier,
    is_parameter_declaration, is_private_identifier, is_property_access_expression,
    is_source_file,
};
use crate::checker::mig::m1b::is_node_descendant_of;
use crate::checker::mig::m1e::r20k2_defs::{is_in_js_file, is_static};
use crate::checker::mig::m2a::r20k6_defs::PatternAmbientModule;
use crate::checker::mig::m2b::r22k6_defs::map_type_ext;
use super::m2c::r24k11_defs::{
    declaration_belongs_to_private_ambient_member, find_best_pattern_match,
    get_enclosing_block_scope_container, get_pattern_ambient_module_augmentation,
    get_pattern_ambient_module_augmentation_target, pattern_ambient_modules,
};
use crate::checker::services_checker_7::IndexKind;
use crate::checker::checker_es_symbol::walk_up_parenthesized_expressions;
use crate::checker::checker_this_container::{
    get_this_parameter, is_in_parameter_initializer_before_containing_function,
};
use super::wc3::NodeAccessExt;
use tsox_frontend::ast::utilities::{is_class_like, is_function_like};
use tsox_frontend::ast::NodeData;
use tsox_core::core::compiler_options_kinds::ScriptTarget;

use super::m2c::r18k3_defs::{
    get_string_literal_value, LanguageFeatureMinimumTarget, UnusedKind,
};
use crate::checker::checker_this_container::get_this_container;
use super::wc1b::is_tuple_type;
use super::wc2_2::{get_mapped_type_optionality, get_number_literal_value};
use super::wc3::TYPE_FLAGS_INSTANTIABLE;
use crate::checker::types_type_id::TYPE_FLAGS_NULLABLE;
use super::wc3_3::is_generic_tuple_type;

pub fn should_mark_identifier_alias_referenced(node: &Arc<Node>) -> bool { ::tsox_core::fntrace::enter("should_mark_identifier_alias_referenced"); 
    if let Some(parent) = node.parent() {
        if is_property_access_expression(&parent)
            && parent.expression().is_some_and(|e| Arc::ptr_eq(e, node))
        {
            return false;
        }
        if is_export_specifier(&parent) && node_is_type_only(&parent) {
            return false;
        }
        if let Some(grandparent) = parent.parent() {
            if let Some(great_grandparent) = grandparent.parent() {
                if is_export_declaration(&great_grandparent) && node_is_type_only(&great_grandparent)
                {
                    return false;
                }
            }
        }
    }
    true
}

fn node_is_type_only(node: &Node) -> bool { ::tsox_core::fntrace::enter("node_is_type_only"); 
    match &node.data {
        NodeData::ExportSpecifier(d) => d.is_type_only,
        NodeData::ExportDeclaration(d) => d.is_type_only,
        _ => false,
    }
}

pub fn signature_has_literal_types(s: &Arc<Signature>) -> bool { ::tsox_core::fntrace::enter("signature_has_literal_types"); 
    s.flags.intersects(SignatureFlags::HasLiteralTypes)
}

pub fn signature_has_rest_parameter(sig: &Arc<Signature>) -> bool { ::tsox_core::fntrace::enter("signature_has_rest_parameter"); 
    sig.flags.intersects(SignatureFlags::HasRestParameter)
}

pub fn some_signature(signatures: &[Arc<Signature>], f: &dyn Fn(&Arc<Signature>) -> bool) -> bool { ::tsox_core::fntrace::enter("some_signature"); 
    for sig in signatures {
        match crate::checker::mig::wc1c::signature_composite(sig) {
            Some(composite) => {
                if composite.is_union && some_signature(&composite.signatures, f) {
                    return true;
                }
            }
            None => {
                if f(sig) {
                    return true;
                }
            }
        }
    }
    false
}

pub fn some_type(t: &Arc<Type>, f: &dyn Fn(&Arc<Type>) -> bool) -> bool { ::tsox_core::fntrace::enter("some_type"); 
    if t.flags.intersects(TypeFlags::Union) {
        if let Some(types) = t.types() {
            return types.iter().any(f);
        }
    }
    f(t)
}

pub fn super_call_is_root_level_in_constructor(super_call: &Arc<Node>, body: &Arc<Node>) -> bool { ::tsox_core::fntrace::enter("super_call_is_root_level_in_constructor"); 
    let Some(parent) = super_call.parent() else {
        return false;
    };
    let super_call_parent = walk_up_parenthesized_expressions(&parent);
    is_expression_statement(&super_call_parent)
        && super_call_parent
            .parent()
            .is_some_and(|p| Arc::ptr_eq(&p, body))
}

impl Checker {
    pub fn symbol_referenced(&mut self, symbol: &Arc<Symbol>, meaning: SymbolFlags) { ::tsox_core::fntrace::enter("symbol_referenced"); 
        if let Some(links) = self.symbol_reference_links.get_mut(symbol) {
            links.reference_kinds |= meaning;
        }
    }

    pub fn symbol_is_value(&mut self, symbol: &Arc<Symbol>) -> bool { ::tsox_core::fntrace::enter("symbol_is_value"); 
        self.symbol_is_value_ex(symbol, false)
    }

    pub fn symbol_is_value_ex(
        &mut self,
        symbol: &Arc<Symbol>,
        include_type_only_members: bool,
    ) -> bool { ::tsox_core::fntrace::enter("symbol_is_value_ex"); 
        symbol.flags.intersects(SymbolFlags::VALUE)
            || (symbol.flags.intersects(SymbolFlags::Alias)
                && self
                    .get_symbol_flags_ex(symbol, !include_type_only_members, false)
                    .intersects(SymbolFlags::VALUE))
    }

    pub fn set_requires_scope_change_cache(&mut self, node: &Arc<Node>, value: Tristate) { ::tsox_core::fntrace::enter("set_requires_scope_change_cache"); 
        if let Some(links) = self.node_links.get_mut(node) {
            links.declaration_requires_scope_change = value;
        }
    }

    pub fn should_check_erasable_syntax(&self, node: &Arc<Node>) -> bool { ::tsox_core::fntrace::enter("should_check_erasable_syntax"); 
        self.compiler_options.erasable_syntax_only.is_true() && !is_in_js_file(node)
    }

    pub fn set_node_links_for_private_identifier_scope(&mut self, node: &Arc<Node>) { ::tsox_core::fntrace::enter("set_node_links_for_private_identifier_scope"); 
        if let Some(name) = node.name() {
            if is_private_identifier(&name) {
                if self.language_version < ScriptTarget::ES2022
                    || self.language_version < ScriptTarget::ES2022
                    || !self.compiler_options.get_use_define_for_class_fields()
                {
                    let mut lexical_scope = get_enclosing_block_scope_container(node);
                    while let Some(scope) = lexical_scope {
                        if let Some(links) = self.node_links.get_mut(&scope) {
                            links.flags |=
                                NodeCheckFlags::ContainsClassWithPrivateIdentifiers;
                        }
                        lexical_scope = get_enclosing_block_scope_container(&scope);
                    }
                }
            }
        }
    }

    pub fn restrictive_mapper_worker(&mut self, t: &Arc<Type>) -> Arc<Type> { ::tsox_core::fntrace::enter("restrictive_mapper_worker"); 
        if t.flags.intersects(TypeFlags::TypeParameter) {
            return self.get_restrictive_type_parameter(t);
        }
        Arc::clone(t)
    }

    pub fn should_defer_index_type(&mut self, t: &Arc<Type>, index_flags: IndexFlags) -> bool { ::tsox_core::fntrace::enter("should_defer_index_type"); 
        t.flags.intersects(TYPE_FLAGS_INSTANTIABLE)
            || is_generic_tuple_type(t)
            || (self.is_generic_mapped_type(t)
                && self.get_name_type_from_mapped_type(t).is_some())
            || (t.flags.intersects(TypeFlags::Union)
                && !index_flags.intersects(IndexFlags::NoReducibleCheck)
                && self.is_generic_reducible_type(t))
            || (t.flags.intersects(TypeFlags::Intersection)
                && self.maybe_type_of_kind(t, TYPE_FLAGS_INSTANTIABLE)
                && t.types()
                    .is_some_and(|ts| {
                        ts.iter().any(|c| self.is_empty_anonymous_object_type(c))
                    }))
    }

    pub fn should_normalize_intersection(&mut self, t: &Arc<Type>) -> bool { ::tsox_core::fntrace::enter("should_normalize_intersection"); 
        let Some(constituents) = t.types() else {
            return false;
        };
        let mut has_instantiable = false;
        let mut has_nullable_or_empty = false;
        for constituent in constituents {
            has_instantiable =
                has_instantiable || constituent.flags.intersects(TYPE_FLAGS_INSTANTIABLE);
            has_nullable_or_empty = has_nullable_or_empty
                || constituent.flags.intersects(TYPE_FLAGS_NULLABLE)
                || self.is_empty_anonymous_object_type(constituent);
            if has_instantiable && has_nullable_or_empty {
                return true;
            }
        }
        false
    }

    pub fn should_report_errors_from_widening_with_contextual_signature(
        &mut self,
        declaration: &Arc<Node>,
        widening_kind: WideningKind,
    ) -> bool { ::tsox_core::fntrace::enter("should_report_errors_from_widening_with_contextual_signature"); 
        let signature = self.get_contextual_signature_for_function_like_declaration(declaration);
        let Some(signature) = signature else {
            return true;
        };
        let Some(return_type) = self.get_return_type_of_signature(&signature) else {
            return false;
        };
        let mut return_type = return_type;
        let flags = get_function_flags(Some(declaration));
        match widening_kind {
            WideningKind::FunctionReturn => {
                if flags.contains(FunctionFlags::GENERATOR) {
                    return_type = self
                        .get_iteration_type_of_generator_function_return_type(
                            IterationTypeKind::RETURN,
                            &return_type,
                            flags.contains(FunctionFlags::ASYNC),
                        )
                        .unwrap_or(return_type);
                } else if flags.contains(FunctionFlags::ASYNC) {
                    return_type =
                        self.get_awaited_type_no_alias(&return_type).unwrap_or(return_type);
                }
                self.is_generic_type(&return_type)
            }
            WideningKind::GeneratorYield => {
                let yield_type = self.get_iteration_type_of_generator_function_return_type(
                    IterationTypeKind::YIELD,
                    &return_type,
                    flags.contains(FunctionFlags::ASYNC),
                );
                yield_type.is_some_and(|t| self.is_generic_type(&t))
            }
            WideningKind::GeneratorNext => {
                let next_type = self.get_iteration_type_of_generator_function_return_type(
                    IterationTypeKind::NEXT,
                    &return_type,
                    flags.contains(FunctionFlags::ASYNC),
                );
                next_type.is_some_and(|t| self.is_generic_type(&t))
            }
            _ => false,
        }
    }

    pub fn skipped_generic_function(&mut self, node: &Arc<Node>, check_mode: CheckMode) { ::tsox_core::fntrace::enter("skipped_generic_function"); 
        if check_mode.intersects(CheckMode::Inferential) {
            for info in self.inference_context_infos.iter_mut().rev() {
                if info.context.is_some()
                    && info.node.as_ref().is_some_and(|n| is_node_descendant_of(node, n))
                {
                    if let Some(context) = info.context.as_mut() {
                        if let Some(owned) = Arc::get_mut(context) {
                            owned.flags |= InferenceFlags::SkippedGenericFunction;
                        }
                    }
                    break;
                }
            }
        }
    }

    pub fn symbol_has_non_method_declaration(&mut self, symbol: &Arc<Symbol>) -> bool { ::tsox_core::fntrace::enter("symbol_has_non_method_declaration"); 
        self.for_each_property(symbol, &mut |_c, prop: &Arc<Symbol>| {
            !prop.flags.intersects(SymbolFlags::Method)
        })
    }

    pub fn transform_type_of_members(
        &mut self,
        t: &Arc<Type>,
        f: &dyn Fn(&Arc<Type>) -> Arc<Type>,
    ) -> SymbolTable { ::tsox_core::fntrace::enter("transform_type_of_members"); 
        let mut members = SymbolTable::new();
        for property in self.get_properties_of_object_type(t) {
            let original = self.get_type_of_symbol(&property);
            let updated = f(&original);
            let property = if !Arc::ptr_eq(&updated, &original) {
                self.create_symbol_with_type(&property, &updated)
            } else {
                property
            };
            members.insert(property.name.clone(), property);
        }
        members
    }

    pub fn try_get_name_from_type(&self, t: &Arc<Type>) -> Option<String> { ::tsox_core::fntrace::enter("try_get_name_from_type"); 
        if t.flags.intersects(TypeFlags::UniqueESSymbol) {
            return t.as_unique_es_symbol_type().map(|d| d.name.clone());
        }
        if t.flags.intersects(TypeFlags::StringLiteral) {
            return Some(get_string_literal_value(t));
        }
        if t.flags.intersects(TypeFlags::NumberLiteral) {
            return Some(get_number_literal_value(t).to_string());
        }
        None
    }

    pub fn type_resolution_has_property(&mut self, r: &TypeResolution) -> bool { ::tsox_core::fntrace::enter("type_resolution_has_property"); 
        match r.property_name {
            TypeSystemPropertyName::Type => r
                .target
                .as_symbol()
                .and_then(|s| self.value_symbol_links.get(s.as_ref()))
                .map(|l| l.resolved_type.is_some())
                .unwrap_or(false),
            TypeSystemPropertyName::DeclaredType => r
                .target
                .as_symbol()
                .and_then(|s| self.type_alias_links.get(s.as_ref()))
                .map(|l| l.declared_type.is_some())
                .unwrap_or(false),
            TypeSystemPropertyName::ResolvedBaseTypes => r
                .target
                .as_type()
                .is_some_and(|t| t.as_interface_type().is_some_and(|i| i.base_types_resolved)),
            TypeSystemPropertyName::ResolvedBaseConstructorType => r
                .target
                .as_type()
                .is_some_and(|t| {
                    t.as_interface_type()
                        .is_some_and(|i| i.resolved_base_constructor_type.get().is_some())
                }),
            TypeSystemPropertyName::ResolvedReturnType => r
                .target
                .as_signature()
                .is_some_and(|s| s.resolved_return_type.get().is_some()),
            TypeSystemPropertyName::ResolvedBaseConstraint => r
                .target
                .as_type()
                .is_some_and(|t| {
                    t.as_constrained_type()
                        .is_some_and(|c| c.resolved_base_constraint.get().is_some())
                }),
            TypeSystemPropertyName::WriteType => r
                .target
                .as_symbol()
                .and_then(|s| self.value_symbol_links.get(s.as_ref()))
                .map(|l| l.write_type.is_some())
                .unwrap_or(false),
            TypeSystemPropertyName::AliasTarget => r
                .target
                .as_symbol()
                .and_then(|s| self.alias_symbol_links.get(s.as_ref()))
                .map(|l| l.alias_target.is_some())
                .unwrap_or(false),
            _ => panic!("Unhandled case in typeResolutionHasProperty"),
        }
    }

    pub fn try_get_rest_type_of_signature(
        &mut self,
        signature: &Arc<Signature>,
    ) -> Option<Arc<Type>> { ::tsox_core::fntrace::enter("try_get_rest_type_of_signature"); 
        if !signature_has_rest_parameter(signature) {
            return None;
        }
        let last_parameter = &signature.parameters[signature.parameters.len() - 1];
        let mut rest_type = self.get_type_of_symbol(last_parameter);
        if is_tuple_type(&rest_type) {
            rest_type = self.get_rest_type_of_tuple_type(&rest_type);
        }
        self.get_index_type_of_type(&rest_type, IndexKind::Number)
    }

    pub fn try_get_this_type_at(&mut self, node: &Arc<Node>) -> Option<Arc<Type>> { ::tsox_core::fntrace::enter("try_get_this_type_at"); 
        self.try_get_this_type_at_ex(node, true, None)
    }

    pub fn try_get_this_type_at_ex_reparsed(
        &mut self,
        node: &Arc<Node>,
        include_global_this: bool,
        container: Option<&Arc<Node>>,
    ) -> Option<Arc<Type>> { ::tsox_core::fntrace::enter("try_get_this_type_at_ex_reparsed"); 
        let reparsed = get_reparsed_node_for_node(node);
        if reparsed.flags.intersects(NodeFlags::JSDoc)
            && !reparsed.flags.intersects(NodeFlags::Reparsed)
        {
            return None;
        }
        let reparsed_container = container.map(get_reparsed_node_for_node);
        self.try_get_this_type_at_ex(&reparsed, include_global_this, reparsed_container.as_ref())
    }

    pub fn try_get_this_type_at_ex(
        &mut self,
        node: &Arc<Node>,
        include_global_this: bool,
        container: Option<&Arc<Node>>,
    ) -> Option<Arc<Type>> { ::tsox_core::fntrace::enter("try_get_this_type_at_ex"); 
        let container = match container {
            Some(container) => Arc::clone(container),
            None => get_this_container(node, false, false),
        };
        if is_function_like(&container)
            && (!is_in_parameter_initializer_before_containing_function(node)
                || get_this_parameter(&container).is_some())
        {
            let sig = self
                .get_signature_of_full_signature_type(&container)
                .or_else(|| self.get_signature_from_declaration(&container));
            let mut this_type = sig.as_ref().and_then(|s| self.get_this_type_of_signature(s));
            if this_type.is_none() {
                this_type = self.get_contextual_this_parameter_type(&container);
            }
            if let Some(this_type) = this_type {
                return Some(self.get_flow_type_of_reference(node, &this_type));
            }
        }
        if let Some(parent) = container.parent() {
            if is_class_like(&parent) {
                let Some(symbol) = self.get_symbol_of_declaration(&parent) else {
                    return None;
                };
                let t = if is_static(&container) {
                    self.get_type_of_symbol(&symbol)
                } else {
                    match self
                        .get_declared_type_of_symbol(&symbol)
                        .as_interface_type()
                        .and_then(|i| i.this_type.clone())
                    {
                        Some(t) => t,
                        // Go getDeclaredTypeOfClassOrInterface：class 恒造 thisType
                        // （isThisType 类型参数、constraint=声明型）；本仓 class
                        // 实例型为 TypeData::Object 无 thisType 槽，按
                        // checker_this_expressions 同构补造，不再退化 unknown
                        None => {
                            let instance = self.container_instance_type_of(&parent);
                            self.create_this_type(&parent, instance)
                        }
                    }
                };
                return Some(self.get_flow_type_of_reference(node, &t));
            }
        }
        if is_source_file(&container) {
            if self
                .get_source_file_of_node(&container)
                .is_some_and(|file| file.external_module_indicator.is_some())
            {
                return Some(self.undefined_type());
            }
            if include_global_this {
                if let Some(global_this) = self.global_this_symbol.clone() {
                    return Some(self.get_type_of_symbol(&global_this));
                }
            }
        }
        None
    }

    pub fn try_get_type_from_type_node(&mut self, node: &Arc<Node>) -> Option<Arc<Type>> { ::tsox_core::fntrace::enter("try_get_type_from_type_node"); 
        let type_node = node.type_node();
        if let Some(type_node) = type_node {
            return Some(self.get_type_from_type_node(&type_node));
        }
        None
    }

    pub fn try_get_declared_type_of_symbol(
        &mut self,
        symbol: &Arc<Symbol>,
    ) -> Option<Arc<Type>> { ::tsox_core::fntrace::enter("try_get_declared_type_of_symbol"); 
        if symbol
            .flags
            .intersects(SymbolFlags::Class | SymbolFlags::Interface)
        {
            return Some(self.get_declared_type_of_class_or_interface(symbol));
        }
        if symbol.flags.intersects(SymbolFlags::TypeParameter) {
            return Some(self.get_declared_type_of_type_parameter(symbol));
        }
        if symbol.flags.intersects(SymbolFlags::TypeAlias) {
            return Some(self.get_declared_type_of_type_alias(symbol));
        }
        if symbol.flags.intersects(SymbolFlags::ENUM) {
            return Some(self.get_declared_type_of_enum(symbol));
        }
        if symbol.flags.intersects(SymbolFlags::EnumMember) {
            return Some(self.get_declared_type_of_enum_member(symbol));
        }
        if symbol.flags.intersects(SymbolFlags::Alias) {
            return Some(self.get_declared_type_of_alias(symbol));
        }
        None
    }

    pub fn try_create_type_reference(
        &mut self,
        target: &Arc<Type>,
        type_arguments: &[Arc<Type>],
    ) -> Arc<Type> { ::tsox_core::fntrace::enter("try_create_type_reference"); 
        if !type_arguments.is_empty() && Arc::ptr_eq(target, &self.empty_generic_type()) {
            return self.unknown_type();
        }
        self.create_type_reference(target, type_arguments)
    }

    pub fn try_create_awaited_type(&mut self, t: &Arc<Type>) -> Option<Arc<Type>> { ::tsox_core::fntrace::enter("try_create_awaited_type"); 
        let awaited_symbol = self.get_global_awaited_symbol();
        if let Some(awaited_symbol) = awaited_symbol {
            let unwrapped = self.unwrap_awaited_type(t);
            return Some(
                self.get_type_alias_instantiation(&awaited_symbol, &[unwrapped], None),
            );
        }
        None
    }

    pub fn unwrap_awaited_type(&mut self, t: &Arc<Type>) -> Arc<Type> { ::tsox_core::fntrace::enter("unwrap_awaited_type"); 
        if t.flags.intersects(TypeFlags::Union) {
            return map_type_ext(self, t, &mut |c, inner| Some(c.unwrap_awaited_type(inner)))
                .unwrap_or_else(|| Arc::clone(t));
        }
        if self.is_awaited_type_instantiation(t) {
            return Arc::clone(&t.alias.as_ref().unwrap().type_arguments[0]);
        }
        Arc::clone(t)
    }

    pub fn type_has_protected_accessible_base(
        &mut self,
        target: &Arc<Symbol>,
        t: &Arc<Type>,
    ) -> bool { ::tsox_core::fntrace::enter("type_has_protected_accessible_base"); 
        let base_types = self.get_base_types(&self.get_target_type(t));
        if base_types.is_empty() {
            return false;
        }
        let first_base = &base_types[0];
        if first_base.flags.intersects(TypeFlags::Intersection) {
            let Some(members) = first_base.types() else {
                return false;
            };
            let (mixin_flags, _) = self.find_mixins(members);
            for (i, intersection_member) in members.iter().enumerate() {
                if !mixin_flags[i]
                    && intersection_member
                        .object_flags
                        .intersects(ObjectFlags::Class | ObjectFlags::Interface)
                {
                    if intersection_member
                        .symbol
                        .as_ref()
                        .is_some_and(|s| Arc::ptr_eq(s, target))
                    {
                        return true;
                    }
                    if self.type_has_protected_accessible_base(target, intersection_member) {
                        return true;
                    }
                }
            }
            return false;
        }
        if first_base.symbol.as_ref().is_some_and(|s| Arc::ptr_eq(s, target)) {
            return true;
        }
        self.type_has_protected_accessible_base(target, first_base)
    }

    pub fn unused_is_error(&self, kind: UnusedKind) -> bool { ::tsox_core::fntrace::enter("unused_is_error"); 
        match kind {
            UnusedKind::Local => self.compiler_options.no_unused_locals.is_true(),
            UnusedKind::Parameter => self.compiler_options.no_unused_parameters.is_true(),
        }
    }

    pub fn widen_type_for_variable_like_declaration(
        &mut self,
        t: Option<Arc<Type>>,
        declaration: &Arc<Node>,
        report_errors: bool,
    ) -> Arc<Type> { ::tsox_core::fntrace::enter("widen_type_for_variable_like_declaration"); 
        let mut t = match t {
            Some(t) => t,
            None => {
                let t = if is_parameter_declaration(declaration)
                    && declaration
                        .as_parameter_declaration()
                        .dot_dot_dot_token
                        .is_some()
                {
                    self.any_array_type()
                } else {
                    self.any_type()
                };
                if report_errors && !declaration_belongs_to_private_ambient_member(declaration) {
                    self.report_implicit_any(declaration, &t, WideningKind::Normal);
                }
                return t;
            }
        };
        if t.flags.intersects(TypeFlags::ESSymbol)
            && declaration
                .parent()
                .is_some_and(|parent| self.is_global_symbol_constructor(&parent))
        {
            t = self.get_es_symbol_like_type_for_node(declaration);
        }
        if report_errors {
            self.report_errors_from_widening(declaration, &t, WideningKind::Normal);
        }
        if t.flags.intersects(TypeFlags::UniqueESSymbol)
            && (is_binding_element(declaration) || declaration.type_node().is_none())
            && !t.symbol.as_ref().is_some_and(|s| {
                self.get_symbol_of_declaration(declaration)
                    .is_some_and(|d| Arc::ptr_eq(s, &d))
            })
        {
            t = self.es_symbol_type();
        }
        self.get_widened_type(&t)
    }

    pub fn widen_type_inferred_from_initializer(
        &mut self,
        declaration: &Arc<Node>,
        t: &Arc<Type>,
    ) -> Arc<Type> { ::tsox_core::fntrace::enter("widen_type_inferred_from_initializer"); 
        let widened = self.get_widened_literal_type_for_initializer(declaration, t);
        if is_in_js_file(declaration) {
            if self.is_empty_literal_type(&widened) {
                self.report_implicit_any(declaration, &self.any_type(), WideningKind::Normal);
                return self.any_type();
            }
            if self.is_empty_array_literal_type(&widened) {
                let any_array_type = self.any_array_type();
                self.report_implicit_any(declaration, &any_array_type, WideningKind::Normal);
                return self.any_array_type();
            }
        }
        widened
    }

    pub fn substitute_indexed_mapped_type(
        &mut self,
        object_type: &Arc<Type>,
        index: &Arc<Type>,
    ) -> Arc<Type> { ::tsox_core::fntrace::enter("substitute_indexed_mapped_type"); 
        let Some(type_parameter) = self.get_type_parameter_from_mapped_type(object_type) else {
            return Arc::clone(object_type);
        };
        let mapper = Arc::new(new_simple_type_mapper(type_parameter, Arc::clone(index)));
        let template_mapper = self.combine_type_mappers(object_type.mapper(), Some(&mapper));
        let template_source = object_type
            .target()
            .cloned()
            .unwrap_or_else(|| Arc::clone(object_type));
        let Some(template_type) = self.get_template_type_from_mapped_type(&template_source)
        else {
            return Arc::clone(object_type);
        };
        let instantiated_template_type =
            self.instantiate_type(&template_type, template_mapper.as_ref());
        let mut is_optional = get_mapped_type_optionality(object_type) > 0;
        if !is_optional {
            if self.is_generic_type(object_type) {
                let modifiers_type = self.get_modifiers_type_from_mapped_type(object_type);
                is_optional = self.get_combined_mapped_type_optionality(&modifiers_type) > 0;
            } else {
                is_optional = self.could_access_optional_property(object_type, index);
            }
        }
        self.add_optionality_ex(&instantiated_template_type, true, is_optional)
    }

    pub fn try_resolve_pattern_ambient_module(
        &mut self,
        resolved_symbol: Option<Arc<Symbol>>,
        module_reference: &str,
        import_attributes_type: &Arc<Type>,
    ) -> Option<Arc<Symbol>> { ::tsox_core::fntrace::enter("try_resolve_pattern_ambient_module"); 
        if self.is_empty_object_type(import_attributes_type) && resolved_symbol.is_some() {
            return resolved_symbol;
        }
        let pattern_ambient_modules_list = pattern_ambient_modules();
        if !pattern_ambient_modules_list.is_empty() {
            let mut candidates: Vec<&PatternAmbientModule> = pattern_ambient_modules_list
                .iter()
                .filter(|v| {
                    let module_attributes_type =
                        self.get_type_of_module_import_attributes(&v.symbol);
                    v.pattern.matches(module_reference)
                        && self.is_type_assignable_to(import_attributes_type, &module_attributes_type)
                })
                .collect();
            if !candidates.is_empty() {
                let augmentation = get_pattern_ambient_module_augmentation(module_reference);
                let augmentation_target =
                    get_pattern_ambient_module_augmentation_target(module_reference);
                if candidates.len() == 1 {
                    let merged_candidate = self.get_merged_symbol(&candidates[0].symbol);
                    if let (Some(augmentation), Some(augmentation_target)) =
                        (augmentation, augmentation_target)
                    {
                        if Arc::ptr_eq(&augmentation_target, &merged_candidate) {
                            return Some(self.get_merged_symbol(&augmentation));
                        }
                    }
                    return Some(merged_candidate);
                }
                let mut best_type_candidates: Vec<&PatternAmbientModule> = Vec::new();
                'outer: for (i, candidate) in candidates.iter().enumerate() {
                    let candidate_type =
                        self.get_type_of_module_import_attributes(&candidate.symbol);
                    for (j, other) in candidates.iter().enumerate() {
                        let other_type =
                            self.get_type_of_module_import_attributes(&other.symbol);
                        if i != j
                            && self.is_type_strict_subtype_of(&other_type, &candidate_type)
                            && !self.is_type_identical_to(&other_type, &candidate_type)
                        {
                            continue 'outer;
                        }
                    }
                    best_type_candidates.push(candidate);
                }
                if best_type_candidates.len() == 1 {
                    let merged_candidate =
                        self.get_merged_symbol(&best_type_candidates[0].symbol);
                    if let (Some(augmentation), Some(augmentation_target)) =
                        (augmentation, augmentation_target)
                    {
                        if Arc::ptr_eq(&augmentation_target, &merged_candidate) {
                            return Some(self.get_merged_symbol(&augmentation));
                        }
                    }
                    return Some(merged_candidate);
                }
                let pattern = find_best_pattern_match(
                    &best_type_candidates,
                    |v| &v.pattern,
                    module_reference,
                );
                let Some(pattern) = pattern else {
                    return resolved_symbol;
                };
                let merged_candidate = self.get_merged_symbol(&pattern.symbol);
                if let (Some(augmentation), Some(augmentation_target)) =
                    (augmentation, augmentation_target)
                {
                    if Arc::ptr_eq(&augmentation_target, &merged_candidate) {
                        return Some(self.get_merged_symbol(&augmentation));
                    }
                }
                return Some(merged_candidate);
            }
        }
        resolved_symbol
    }
}
