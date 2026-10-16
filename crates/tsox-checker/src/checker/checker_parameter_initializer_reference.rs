#![allow(unused_imports)]

use crate::checker::checker_expressions::*;
use crate::checker::checker_prop_access_checker_4::is_immediately_invoked;
use crate::checker::mig::wc3::symbol_ptr_key;
use std::sync::Arc;
use tsox_frontend::ast::{Node, Symbol};
use tsox_core::diagnostics::messages_generated::{
    PARAMETER_0_CANNOT_REFERENCE_IDENTIFIER_1_DECLARED_AFTER_IT, PARAMETER_0_CANNOT_REFERENCE_ITSELF,
};
use tsox_frontend::scanner::mig::m3i::declaration_name_to_string;

impl Checker {
    pub(crate) fn check_parameter_initializer_reference(
        &mut self,
        node: &Arc<Node>,
        resolved: &Arc<Symbol>,
    ) {
        let (associated, within_deferred_context) = self.parameter_initializer_reference_state(node);
        let Some(associated) = associated else {
            return;
        };
        if within_deferred_context {
            return;
        }
        let resolution_meaning = SymbolFlags::VALUE | SymbolFlags::ExportValue;
        let late_bound = self.get_late_bound_symbol(resolved);
        let candidate = self.get_merged_symbol(&late_bound);
        let associated_symbol = self.get_symbol_of_declaration(&associated);
        if associated_symbol
            .as_ref()
            .is_some_and(|s| symbol_ptr_key(s) == symbol_ptr_key(&candidate))
        {
            self.error_message(
                node,
                PARAMETER_0_CANNOT_REFERENCE_ITSELF,
                &[declaration_name_to_string(associated.name())],
            );
            return;
        }
        let Some(value_declaration) = &candidate.value_declaration else {
            return;
        };
        if value_declaration.pos() <= associated.pos() {
            return;
        }
        let root = tsox_frontend::ast::get_root_declaration(&associated);
        let Some(root_parent) = root.parent() else {
            return;
        };
        let Some(locals) = self
            .program
            .symbol_map()
            .locals
            .get(&root_parent.id())
            .cloned()
        else {
            return;
        };
        let local = self.get_symbol(&locals, &candidate.name, resolution_meaning);
        if local
            .as_ref()
            .is_some_and(|s| symbol_ptr_key(s) == symbol_ptr_key(&candidate))
        {
            self.error_message(
                node,
                PARAMETER_0_CANNOT_REFERENCE_IDENTIFIER_1_DECLARED_AFTER_IT,
                &[
                    declaration_name_to_string(associated.name()),
                    declaration_name_to_string(Some(node)),
                ],
            );
        }
    }

    fn parameter_initializer_reference_state(
        &mut self,
        node: &Arc<Node>,
    ) -> (Option<Arc<Node>>, bool) {
        let name = match &node.data {
            tsox_frontend::ast::NodeData::Identifier(data) => data.text.as_str(),
            _ => return (None, false),
        };
        let resolution_meaning = SymbolFlags::VALUE | SymbolFlags::ExportValue;
        let mut associated: Option<Arc<Node>> = None;
        let mut within_deferred_context = false;
        let mut last_location: Option<Arc<Node>> = None;
        let mut current = Some(Arc::clone(node));
        while let Some(location) = current {
            let locals = self
                .program
                .symbol_map()
                .locals
                .get(&location.id())
                .cloned();
            if !Self::is_global_source_file(&location)
                && locals.is_some_and(|locals| {
                    self.get_symbol(&locals, name, resolution_meaning)
                        .is_some()
                })
            {
                break;
            }
            within_deferred_context |= Self::is_deferred_context(&location, last_location.as_ref());
            if associated.is_none()
                && matches!(
                    location.kind,
                    SyntaxKind::Parameter | SyntaxKind::BindingElement
                )
                && Self::initializer_or_binding_pattern_name_contains(
                    &location,
                    last_location.as_ref(),
                )
                && (location.kind == SyntaxKind::Parameter
                    || Self::part_of_parameter_declaration(&location))
            {
                associated = Some(Arc::clone(&location));
            }
            last_location = Some(Arc::clone(&location));
            current = location.parent();
        }
        (associated, within_deferred_context)
    }

    fn initializer_or_binding_pattern_name_contains(
        location: &Arc<Node>,
        last_location: Option<&Arc<Node>>,
    ) -> bool {
        let Some(last_location) = last_location else {
            return false;
        };
        match &location.data {
            tsox_frontend::ast::NodeData::ParameterDeclaration(d) => {
                d.initializer
                    .as_ref()
                    .is_some_and(|init| Arc::ptr_eq(init, last_location))
                    || Arc::ptr_eq(&d.name, last_location)
                        && tsox_frontend::ast::is_binding_pattern(&d.name)
            }
            tsox_frontend::ast::NodeData::BindingElement(d) => {
                d.initializer
                    .as_ref()
                    .is_some_and(|init| Arc::ptr_eq(init, last_location))
                    || d.name.as_ref().is_some_and(|name| {
                        Arc::ptr_eq(name, last_location)
                            && tsox_frontend::ast::is_binding_pattern(name)
                    })
            }
            _ => false,
        }
    }

    fn part_of_parameter_declaration(node: &Arc<Node>) -> bool {
        tsox_frontend::ast::get_root_declaration(node).kind == SyntaxKind::Parameter
    }

    fn is_deferred_context(location: &Arc<Node>, last_location: Option<&Arc<Node>>) -> bool {
        if !matches!(
            location.kind,
            SyntaxKind::ArrowFunction | SyntaxKind::FunctionExpression
        ) {
            let deferred_container = tsox_frontend::ast::is_function_like_declaration(location)
                || (location.kind == SyntaxKind::PropertyDeclaration
                    && !location
                        .has_syntactic_modifier(tsox_frontend::ast::ModifierFlags::Static));
            return tsox_frontend::ast::is_type_query_node(location)
                || (deferred_container && !Self::last_location_is_name(location, last_location));
        }
        if Self::last_location_is_name(location, last_location) {
            return false;
        }
        let generator = matches!(
            &location.data,
            tsox_frontend::ast::NodeData::FunctionExpression(d) if d.asterisk_token.is_some()
        );
        generator
            || location.has_syntactic_modifier(tsox_frontend::ast::ModifierFlags::Async)
            || !is_immediately_invoked(location)
    }

    fn last_location_is_name(location: &Arc<Node>, last_location: Option<&Arc<Node>>) -> bool {
        last_location
            .map(|last| location.name().is_some_and(|n| Arc::ptr_eq(n, last)))
            .unwrap_or(false)
    }
}
