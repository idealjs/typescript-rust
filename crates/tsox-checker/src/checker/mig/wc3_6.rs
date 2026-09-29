#![allow(unused_imports)]

use crate::checker::checker_checker::*;
use crate::checker::mig::wc3::NodeAccessExt;
use crate::checker::mig::wc3::{ReferenceHint, is_invalid_computed_property_name, is_internal_module_import_equals_declaration};
use crate::checker::jsx_impl_chunk::is_jsx_intrinsic_tag_name;
use crate::checker::mig::m2c_3::should_mark_identifier_alias_referenced;
use r24k4_defs::is_expression_node_r24k4;
use std::sync::Arc;

#[path = "r24k4_defs.rs"]
pub mod r24k4_defs;
use tsox_core::diagnostics::messages_generated as msg;
use tsox_frontend::ast::{self, Node, Symbol, SyntaxKind};

impl Checker {
    pub fn mark_linked_references(
        &mut self,
        location: &Arc<Node>,
        hint: ReferenceHint,
        prop_symbol: Option<&Arc<Symbol>>,
        parent_type: Option<&Arc<Type>>,
    ) {
        if !self.can_collect_symbol_alias_accessibility_data {
            return;
        }
        if location.flags.contains(NodeFlags::Ambient)
            && !ast::is_property_signature_declaration(location)
            && !ast::is_property_declaration(location)
        {
            return;
        }
        match hint {
            ReferenceHint::Identifier => {
                self.mark_identifier_alias_referenced(location);
            }
            ReferenceHint::Property => {
                self.mark_property_alias_referenced(location, prop_symbol, parent_type);
            }
            ReferenceHint::ExportAssignment => {
                self.mark_export_assignment_alias_referenced(location);
            }
            ReferenceHint::Jsx => {
                self.mark_jsx_alias_referenced(location);
            }
            ReferenceHint::ExportImportEquals => {
                self.mark_import_equals_alias_referenced(location);
            }
            ReferenceHint::ExportSpecifier => {
                self.mark_export_specifier_alias_referenced(location);
            }
            ReferenceHint::Decorator => {
                self.mark_decorator_alias_referenced(location);
            }
            ReferenceHint::Unspecified => {
                self.mark_linked_references_unspecified(location);
            }
        }
    }

    fn mark_linked_references_unspecified(&mut self, location: &Arc<Node>) {
        if location.flags.contains(NodeFlags::InWithStatement) {
            return;
        }
        if ast::is_jsx_tag_name(location) && is_jsx_intrinsic_tag_name(location) {
            return;
        }
        if ast::is_identifier(location) {
            let parent = location.parent();
            if let Some(parent) = parent {
                if ast::is_shorthand_property_assignment(&parent)
                    && Arc::ptr_eq(&parent.name().unwrap(), location)
                    && parent
                        .as_shorthand_property_assignment()
                        .object_assignment_initializer
                        .is_some()
                    && !crate::checker::checker_object_literal_is_destructuring_target::is_assignment_target(&parent.parent().unwrap())
                {
                    return;
                }
            }
            let found = ast::mig::m3e_4::find_many_ancestors(
                Some(location),
                &[
                    &|n: &Arc<Node>| ast::is_meta_property(n),
                    &|n: &Arc<Node>| ast::is_decorator(n),
                    &|n: &Arc<Node>| ast::is_for_in_or_of_statement(n),
                    &|n: &Arc<Node>| ast::is_computed_property_name(n),
                    &|n: &Arc<Node>| ast::is_heritage_clause(n),
                ],
            );
            let (meta_property, decorator, for_node, computed_name, heritage_clause) = (
                found[0].clone(),
                found[1].clone(),
                found[2].clone(),
                found[3].clone(),
                found[4].clone(),
            );
            if meta_property.is_some() {
                return;
            }
            if let Some(decorator) = decorator.as_ref() {
                let decorated = decorator.parent();
                if let Some(decorated) = decorated {
                    let decorated_parent = decorated.parent();
                    let decorated_grandparent =
                        decorated_parent.as_ref().and_then(|p| p.parent());
                    if !ast::mig::m3g_3::node_can_be_decorated(
                        self.legacy_decorators,
                        &decorated,
                        decorated_parent.as_ref(),
                        decorated_grandparent.as_ref(),
                    ) {
                        return;
                    }
                }
            }
            if let Some(for_node) = for_node.as_ref() {
                let data = for_node.as_for_in_or_of_statement();
                let initializer = &data.initializer;
                if ast::is_variable_declaration_list(initializer)
                    && initializer
                        .as_variable_declaration_list()
                        .declarations
                        .nodes
                        .is_empty()
                    && (Arc::ptr_eq(location, &data.expression)
                        || ast::is_node_descendant_of(location, &data.expression))
                {
                    return;
                }
            }
            if let Some(computed_name) = computed_name.as_ref() {
                if computed_name
                    .parent()
                    .as_ref()
                    .is_some_and(|p| ast::is_enum_member(p))
                {
                    return;
                }
                if is_invalid_computed_property_name(computed_name) {
                    return;
                }
            }
            if let Some(heritage_clause) = heritage_clause.as_ref() {
                if heritage_clause
                    .parent()
                    .as_ref()
                    .is_some_and(|p| ast::is_interface_declaration(p))
                {
                    return;
                }
                if heritage_clause
                    .parent()
                    .as_ref()
                    .is_some_and(|p| ast::is_class_like(p))
                    && crate::checker::mig::wc3::r25k2_defs::r25k2_as_heritage_clause(heritage_clause).token
                        == SyntaxKind::ExtendsKeyword
                {
                    if let (Some(parent), Some(first_extends)) = (
                        heritage_clause.parent(),
                        heritage_clause
                            .parent()
                            .as_ref()
                            .and_then(|p| ast::get_class_extends_heritage_element(p)),
                    ) {
                        if !Arc::ptr_eq(location, &first_extends)
                            && !ast::is_node_descendant_of(location, &first_extends)
                        {
                            return;
                        }
                    }
                }
            }
            let is_expression_context = is_expression_node_r24k4(location)
                || location
                    .parent()
                    .as_ref()
                    .is_some_and(|p| ast::is_shorthand_property_assignment(p));
            if is_expression_context && should_mark_identifier_alias_referenced(location) {
                let parent = location.parent();
                if parent
                    .as_ref()
                    .is_some_and(|p| ast::is_property_access_or_qualified_name(p))
                {
                    let parent = parent.unwrap();
                    let left = if ast::is_property_access_expression(&parent) {
                        parent.expression().cloned()
                    } else {
                        Some(parent.as_qualified_name().left.clone())
                    };
                    if let Some(left) = left {
                        if !Arc::ptr_eq(&left, location) {
                            return;
                        }
                    }
                }
                self.mark_identifier_alias_referenced(location);
                return;
            }
        }
        if ast::is_property_access_or_qualified_name(location) {
            let mut top_prop = Arc::clone(location);
            while ast::is_property_access_or_qualified_name(&top_prop) {
                if ast::mig::m3g_3::is_part_of_type_node(&top_prop) {
                    return;
                }
                top_prop = top_prop.parent().unwrap();
            }
            self.mark_property_alias_referenced(location, None, None);
            return;
        }
        if ast::is_export_assignment(location) {
            self.mark_export_assignment_alias_referenced(location);
            return;
        }
        if ast::mig::m3g::is_jsx_opening_like_element(location) || ast::is_jsx_opening_fragment(location) {
            self.mark_jsx_alias_referenced(location);
            return;
        }
        if ast::is_import_equals_declaration(location) {
            if is_internal_module_import_equals_declaration(location)
                || self.check_external_import_or_export_declaration(location)
            {
                self.mark_import_equals_alias_referenced(location);
                return;
            }
            return;
        }
        if ast::is_export_specifier(location) {
            self.mark_export_specifier_alias_referenced(location);
            return;
        }
        if !self.compiler_options.emit_decorator_metadata.is_true() {
            return;
        }
        if !ast::can_have_decorators(location)
            || !location
                .has_syntactic_modifier(tsox_frontend::ast::ModifierFlags::Decorator)
            || location.modifiers().is_none()
        {
            return;
        }
        let parent = location.parent().unwrap();
        let grandparent = parent.parent().unwrap();
        if !ast::mig::m3g_3::node_can_be_decorated(
            self.legacy_decorators,
            location,
            Some(&parent),
            Some(&grandparent),
        ) {
            return;
        }
        self.mark_decorator_alias_referenced(location);
    }
}
