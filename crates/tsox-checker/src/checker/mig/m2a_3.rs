#![allow(unused_imports)]
use crate::checker::mig::m2a::r19k11_defs::*;
use crate::checker::mig::m2a::r18k8_flags::*;
use tsox_frontend::ast::mig::m3e::get_function_flags;
use tsox_frontend::ast::mig::m3e_4::{get_first_identifier, get_heritage_clause_element_name, get_this_container};
use tsox_frontend::ast::mig::m3f_4::{is_call_like_expression, is_call_or_new_expression};
use tsox_frontend::ast::mig::m3g_2::is_non_local_alias;
use crate::checker::mig::m2b_2::ObjectLiteralDiscriminator;
use crate::checker::mig::m2d::is_const_enum_or_const_enum_only_module;
use crate::checker::mig::m3a_2::{get_containing_class_excluding_class_decorators, new_diagnostic_for_node};
use crate::checker::mig::wc1b::is_type_any;
use crate::checker::mig::wc3::ThisAssignmentDeclarationKind;
use crate::checker::mig::wc3_2::is_export_or_export_expression;
use crate::checker::utilities_get_assignment_target::is_in_type_query;
use crate::checker::utilities_has_only_expression_initialization::is_this_property;
use crate::checker::utilities_is_optional_symbol::is_numeric_literal_name;
use crate::checker::utilities_is_private_within_ambient::get_containing_object_literal;

use crate::checker::checker_checker::*;
use crate::checker::types::Signature;
use crate::checker::utilities_token_is_identifier_or_keyword::is_unit_type;
use crate::checker::exports_union_reduction::get_declaration_modifier_flags_from_symbol;
use crate::binder::mig::m3h::get_symbol_name_for_private_identifier;
use crate::checker::mig::wc3::r18k4_node_ext::NodeAccessExt;
use std::sync::Arc;
use tsox_core::diagnostics::messages_generated;
use tsox_frontend::ast::{self, Node, Symbol, SyntaxKind};

impl Checker {
    pub fn is_this_property_access_in_constructor(
        &mut self,
        node: &Arc<Node>,
        prop: &Arc<Symbol>,
    ) -> bool {
        let mut constructor: Option<Arc<Node>> = None;
        let (kind, location) = self.is_constructor_declared_this_property(prop);
        if kind == ThisAssignmentDeclarationKind::Constructor {
            constructor = location;
        } else if is_this_property(node) && self.is_auto_typed_property(prop) {
            constructor = self.get_declaring_constructor(prop);
        }
        let container = get_this_container(node, true, false);
        constructor.is_some_and(|ctor| Arc::ptr_eq(&container, &ctor))
    }

    pub fn is_this_property_and_this_typed(&mut self, node: &Arc<Node>) -> bool {
        if node
            .expression()
            .is_some_and(|e| e.kind == SyntaxKind::ThisKeyword)
        {
            let container = get_this_container(node, false, false);
            if ast::is_function_like(&container) {
                if let Some(containing_literal) = get_containing_object_literal(&container) {
                    let contextual_type =
                        self.get_apparent_type_of_contextual_type(&containing_literal, ContextFlags::None);
                    let t = self.get_this_type_of_object_literal_from_contextual_type(
                        &containing_literal,
                        contextual_type.as_ref(),
                    );
                    return t.is_some_and(|t| !is_type_any(&t));
                }
            }
        }
        false
    }

    pub fn is_thisless_interface(&mut self, symbol: &Arc<Symbol>) -> bool {
        for declaration in &symbol.declarations {
            if ast::is_interface_declaration(declaration) {
                if declaration.flags.intersects(NODE_FLAGS_CONTAINS_THIS) {
                    return false;
                }
                for node in ast::get_extends_heritage_clause_elements(declaration) {
                    let Some(name) = get_heritage_clause_element_name(&node) else {
                        continue;
                    };
                    if ast::is_entity_name(&name) || ast::is_entity_name_expression(&name) {
                        let base_symbol =
                            self.resolve_entity_name(&name, SymbolFlags::TYPE, true, false, None);
                        if base_symbol.as_ref().map(|s| s.flags.intersects(SymbolFlags::Interface)) != Some(true)
                            || self
                                .get_declared_type_of_class_or_interface(base_symbol.as_ref().unwrap())
                                .as_interface_type()
                                .unwrap()
                                .this_type
                                .is_some()
                        {
                            return false;
                        }
                    }
                }
            }
        }
        true
    }

    pub fn is_type_matched_by_template_literal_or_string_mapping(
        &mut self,
        t: &Arc<Type>,
        template: &Arc<Type>,
    ) -> bool {
        if template.flags.contains(TypeFlags::TemplateLiteral) {
            return self.is_type_matched_by_template_literal_type(
                t,
                template.as_template_literal_type().unwrap(),
            );
        }
        self.is_member_of_string_mapping(t, template)
    }

    pub fn is_type_usable_as_index_signature_declaration(&mut self, t: &Arc<Type>) -> bool {
        let string_number_symbol_type = self.string_number_symbol_type.clone();
        self.is_type_assignable_to(t, &string_number_symbol_type)
    }

    pub fn is_uncalled_function_reference(&mut self, node: &Arc<Node>, symbol: &Arc<Symbol>) -> bool {
        if symbol
            .flags
            .intersects(SymbolFlags::Function | SymbolFlags::Method)
        {
            let mut parent = ast::find_ancestor(&node.parent().unwrap(), |n: &Node| {
                !ast::is_access_expression(n)
            });
            if parent.is_none() {
                parent = node.parent();
            }
            if let Some(parent) = parent {
                if is_call_like_expression(&parent) {
                    return is_call_or_new_expression(&parent)
                        && ast::is_identifier(node)
                        && self.has_matching_argument(&parent, node);
                }
            }
            return symbol.declarations.iter().all(|d| {
                !ast::is_function_like(d) || self.is_deprecated_declaration(d)
            });
        }
        true
    }

    pub fn is_uniform_union_type(&mut self, t: &Arc<Type>) -> bool {
        if t.object_flags.intersects(ObjectFlags::PrimitiveUnion) {
            if !t
                .object_flags
                .intersects(OBJECT_FLAGS_IS_UNIFORM_ENUM_COMPUTED)
            {
                let is_uniform = self.compute_is_uniform_union_type(t.types().unwrap());
                let mut flags = OBJECT_FLAGS_IS_UNIFORM_ENUM_COMPUTED;
                if is_uniform {
                    flags |= OBJECT_FLAGS_IS_UNIFORM_ENUM;
                }
                let t_mut = Arc::as_ptr(t) as *mut Type;
                unsafe {
                    (*t_mut).object_flags |= flags;
                }
            }
            return t.object_flags.intersects(OBJECT_FLAGS_IS_UNIFORM_ENUM);
        }
        false
    }

    pub fn is_unit_like_type(&mut self, t: &Arc<Type>) -> bool {
        let t = self.get_base_constraint_or_type(t);
        if t.flags.contains(TypeFlags::Intersection) {
            return t
                .as_intersection_type()
                .unwrap()
                .union_or_intersection
                .types
                .iter()
                .any(|u| is_unit_type(u));
        }
        is_unit_type(&t)
    }

    pub fn is_untyped_function_call(
        &mut self,
        func_type: &Arc<Type>,
        apparent_func_type: &Arc<Type>,
        num_call_signatures: usize,
        num_construct_signatures: usize,
    ) -> bool {
        is_type_any(func_type)
            || (is_type_any(apparent_func_type)
                && func_type.flags.contains(TypeFlags::TypeParameter))
            || (num_call_signatures == 0
                && num_construct_signatures == 0
                && !apparent_func_type.flags.contains(TypeFlags::Union)
                && !self
                    .get_reduced_type(apparent_func_type)
                    .flags
                    .contains(TypeFlags::Never)
                && {
                    let global_function_type = self.global_function_type();
                    self.is_type_assignable_to(func_type, &global_function_type)
                })
    }

    pub fn is_unwrapped_return_type_undefined_void_or_any(
        &mut self,
        fn_: &Arc<Node>,
        return_type: &Arc<Type>,
    ) -> bool {
        let t = self.unwrap_return_type(return_type, get_function_flags(Some(fn_)));
        t.as_ref().is_some_and(|t| {
            self.maybe_type_of_kind(t, TYPE_FLAGS_VOID)
                || t.flags.intersects(TYPE_FLAGS_ANY | TypeFlags::Undefined)
        })
    }

    pub fn is_valid_base_type(&mut self, t: &Arc<Type>) -> bool {
        if t.flags.contains(TypeFlags::TypeParameter) {
            if let Some(constraint) = self.get_base_constraint_of_type(t) {
                return self.is_valid_base_type(&constraint);
            }
        }
        (t.flags.intersects(TYPE_FLAGS_OBJECT | TypeFlags::NonPrimitive | TypeFlags::Any)
            && !self.is_generic_mapped_type(t))
            || (t.flags.contains(TypeFlags::Intersection)
                && t.types()
                    .unwrap()
                    .iter()
                    .all(|x| self.is_valid_base_type(x)))
    }

    pub fn is_valid_index_key_type(&mut self, t: &Arc<Type>) -> bool {
        t.flags
            .intersects(TYPE_FLAGS_STRING | TypeFlags::Number | TypeFlags::ESSymbol)
            || self.is_pattern_literal_type(t)
            || (t.flags.contains(TypeFlags::Intersection)
                && !self.is_generic_type(t)
                && t.types()
                    .unwrap()
                    .iter()
                    .any(|x| self.is_valid_index_key_type(x)))
    }

    pub fn is_valid_override_of(&mut self, source_prop: &Arc<Symbol>, target_prop: &Arc<Symbol>) -> bool {
        !self.for_each_property(target_prop, &mut |c: &mut Checker, tp: &Arc<Symbol>| {
            if get_declaration_modifier_flags_from_symbol(tp).intersects(MODIFIER_FLAGS_PROTECTED) {
                let base_class = c.get_declaring_class(tp).unwrap();
                return !c.is_property_in_class_derived_from(source_prop, &base_class);
            }
            false
        })
    }

    pub fn is_var_const_like(&mut self, node: &Arc<Node>) -> bool {
        let block_scope_kind = self.get_combined_node_flags_cached(node) & NODE_FLAGS_BLOCK_SCOPED;
        block_scope_kind == NODE_FLAGS_CONST
            || block_scope_kind == NODE_FLAGS_USING
            || block_scope_kind == NODE_FLAGS_AWAIT_USING
    }

    pub fn lookup_or_issue_error(
        &mut self,
        location: &Arc<Node>,
        message: &tsox_core::diagnostics::Message,
        args: &[Box<dyn std::fmt::Display>],
    ) -> Option<ast::Diagnostic> {
        Some(self.add_diagnostic(new_diagnostic_for_node(
            Some(location),
            message.clone(),
            args.iter().map(|a| a.to_string()).collect(),
        )))
    }

    pub fn lookup_symbol_for_private_identifier_declaration(
        &mut self,
        prop_name: &str,
        location: &Arc<Node>,
    ) -> Option<Arc<Symbol>> {
        let mut containing_class = get_containing_class_excluding_class_decorators(location);
        while let Some(class) = containing_class {
            let Some(symbol) = self.get_symbol_of_declaration(&class) else {
                containing_class = ast::get_containing_class(&class);
                continue;
            };
            let name = get_symbol_name_for_private_identifier(&symbol, prop_name);
            if let Some(prop) = symbol.members.get(&name) {
                return Some(Arc::clone(prop));
            }
            if let Some(prop) = symbol.exports.get(&name) {
                return Some(Arc::clone(prop));
            }
            containing_class = ast::get_containing_class(&class);
        }
        None
    }

    pub fn map_type(&mut self, t: &Arc<Type>, f: &mut dyn FnMut(&Arc<Type>) -> Option<Arc<Type>>) -> Option<Arc<Type>> {
        self.map_type_ex(t, f, false)
    }

    pub fn map_type_with_alias(
        &mut self,
        t: &Arc<Type>,
        f: &mut dyn FnMut(&Arc<Type>) -> Option<Arc<Type>>,
        alias: Option<&TypeAlias>,
    ) -> Option<Arc<Type>> {
        if t.flags.contains(TypeFlags::Union) && alias.is_some() {
            let mapped = t
                .types()
                .unwrap()
                .iter()
                .filter_map(|s| f(s))
                .collect::<Vec<_>>();
            return Some(self.get_union_type_ex(mapped, UnionReduction::Literal));
        }
        self.map_type(t, f)
    }

    pub fn map_type_ex(
        &mut self,
        t: &Arc<Type>,
        f: &mut dyn FnMut(&Arc<Type>) -> Option<Arc<Type>>,
        no_reductions: bool,
    ) -> Option<Arc<Type>> {
        if t.flags.contains(TypeFlags::Never) {
            return Some(Arc::clone(t));
        }
        if !t.flags.contains(TypeFlags::Union) {
            return f(t);
        }
        let mut types = t.types().unwrap_or(&[]).to_vec();
        if let Some(origin) = t.as_union_type().and_then(|u| u.origin.clone()) {
            if origin.flags.contains(TypeFlags::Union) {
                types = origin.types().unwrap_or(&[]).to_vec();
            }
        }
        let mut mapped_types: Vec<Arc<Type>> = Vec::new();
        let mut changed = false;
        for s in &types {
            let mapped = if s.flags.contains(TypeFlags::Union) {
                self.map_type_ex(s, f, no_reductions)
            } else {
                f(s)
            };
            if !mapped
                .as_ref()
                .is_some_and(|m| Arc::ptr_eq(m, s))
            {
                changed = true;
            }
            if let Some(mapped) = mapped {
                mapped_types.push(mapped);
            }
        }
        if changed {
            if mapped_types.is_empty() {
                return None;
            }
            let reduction = if no_reductions {
                UnionReduction::None
            } else {
                UnionReduction::Literal
            };
            return Some(self.get_union_type_ex(mapped_types, reduction));
        }
        Some(Arc::clone(t))
    }

    pub fn mark_alias_referenced(&mut self, symbol: &Arc<Symbol>, location: &Arc<Node>) {
        if !self.can_collect_symbol_alias_accessibility_data {
            return;
        }
        if is_non_local_alias(Some(symbol.as_ref()), SymbolFlags::VALUE) && !is_in_type_query(location) {
            let target = self.resolve_alias(symbol);
            if self
                .get_symbol_flags_ex(symbol, true, false)
                .intersects(SymbolFlags::VALUE | SymbolFlags::ExportValue)
            {
                let preserved = self.compiler_options.should_preserve_const_enums()
                    && is_export_or_export_expression(location);
                if self.compiler_options.get_isolated_modules()
                    || preserved
                    || !is_const_enum_or_const_enum_only_module(
                        &self.get_export_symbol_of_value_symbol_if_exported(&target),
                    )
                {
                    self.mark_alias_symbol_as_referenced(symbol);
                }
            }
        }
    }

    pub fn mark_alias_symbol_as_referenced(&mut self, symbol: &Arc<Symbol>) {
        let already_referenced = self
            .alias_symbol_links
            .get_mut(symbol)
            .map(|links| links.referenced)
            .unwrap_or(false);
        if !already_referenced {
            if let Some(links) = self.alias_symbol_links.get_mut(symbol) {
                links.referenced = true;
            }
            let node = self
                .get_declaration_of_alias_symbol(symbol)
                .expect("Unexpected nil in markAliasSymbolAsReferenced");
            if ast::is_import_equals_declaration(&node)
                && node.as_import_equals_declaration().module_reference.kind
                    != SyntaxKind::ExternalModuleReference
            {
                let resolved = self.resolve_symbol(symbol);
                if self.get_symbol_flags(&resolved).intersects(SymbolFlags::VALUE) {
                    let left = get_first_identifier(
                        &node.as_import_equals_declaration().module_reference,
                    );
                    self.mark_identifier_alias_referenced(&left);
                }
            }
        }
    }

    pub fn mark_decorator_medata_data_type_node_as_referenced(&mut self, node: &Arc<Node>) {
        if let Some(entity_name) = self.get_entity_name_for_decorator_metadata(Some(node)) {
            if ast::is_entity_name(&entity_name) {
                self.mark_entity_name_or_entity_expression_as_reference(&entity_name, true);
            }
        }
    }

    pub fn mark_export_as_referenced(&mut self, node: &Arc<Node>) {
        let symbol = self.get_symbol_of_declaration(node);
        if let Some(symbol) = symbol {
            let target = self.resolve_alias(&symbol);
            let mark_alias = self
                .unknown_symbol
                .as_ref()
                .is_some_and(|unknown| Arc::ptr_eq(&target, unknown))
                || (self
                    .get_symbol_flags_ex(&symbol, true, false)
                    .intersects(SymbolFlags::VALUE)
                    && !is_const_enum_or_const_enum_only_module(&target));
            if mark_alias {
                self.mark_alias_symbol_as_referenced(&symbol);
            }
        }
    }

    pub fn mark_export_assignment_alias_referenced(&mut self, location: &Arc<Node>) {
        let id = location.expression();
        if let Some(id) = id {
            if ast::is_identifier(&id) {
                let resolved = self.resolve_entity_name(&id, SymbolFlags::all(), true, true, Some(location));
                if let Some(sym) = resolved {
                    let sym = self.get_export_symbol_of_value_symbol_if_exported(&sym);
                    self.mark_alias_referenced(&sym, &id);
                }
            }
        }
    }

    pub fn mark_import_equals_alias_referenced(&mut self, location: &Arc<Node>) {
        if ast::has_syntactic_modifier(location, MODIFIER_FLAGS_EXPORT) {
            self.mark_export_as_referenced(location);
        }
    }

    pub fn mark_identifier_alias_referenced(&mut self, location: &Arc<Node>) {
        if ast::is_this_in_type_query(location) {
            return;
        }
        if let Some(symbol) = self.get_resolved_symbol(location) {
            let arguments_symbol = self.arguments_symbol.clone();
            let unknown_symbol = self.unknown_symbol.clone();
            let is_arguments_or_unknown = arguments_symbol
                .as_ref()
                .is_some_and(|a| Arc::ptr_eq(&symbol, a))
                || unknown_symbol
                    .as_ref()
                    .is_some_and(|u| Arc::ptr_eq(&symbol, u));
            if !is_arguments_or_unknown {
                self.mark_alias_referenced(&symbol, location);
            }
        }
    }
}

pub fn object_literal_discriminator_len(d: &ObjectLiteralDiscriminator) -> usize {
    d.props.len() + d.members.len()
}
