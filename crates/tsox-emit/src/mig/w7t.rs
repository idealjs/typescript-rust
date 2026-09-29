#![allow(dead_code, unused_imports, unused_variables)]

use std::sync::Arc;

use tsox_frontend::ast::*;
use crate::mig::m4q::r33k12_defs::{
    FindAncestorResult, RuntimeSyntaxTransformer, is_auto_accessor_property_declaration,
    to_find_ancestor_result,
};

impl RuntimeSyntaxTransformer {
    pub fn is_first_declaration_in_scope(&self, node: &Arc<Node>) -> bool {
        if let Some(name) = node.name() {
            if is_identifier(&name) {
                let text = name.text();
                if let Some(first_declaration) =
                    self.current_scope_first_declarations_of_name.get(text)
                {
                    return Arc::ptr_eq(first_declaration, node);
                }
            }
        }
        false
    }

    pub fn is_export_of_namespace(&self, node: &Node) -> bool {
        self.current_namespace.is_some()
            && !self
                .current_scope
                .as_ref()
                .is_some_and(|s| s.kind == SyntaxKind::Block)
            && node
                .modifiers()
                .map_or(ModifierFlags::empty(), |m| m.modifier_flags)
                .intersects(ModifierFlags::Export)
    }
}

pub fn is_static_property_declaration_or_class_static_block(node: &Node) -> bool {
    is_class_static_block_declaration(node)
        || (is_property_declaration(node) && has_static_modifier(node))
}

pub fn is_non_static_method_or_accessor_with_private_name(member: &Node) -> bool {
    !is_static(member)
        && (is_method_or_accessor(member) || is_auto_accessor_property_declaration(member))
        && member.name().is_some_and(|n| is_private_identifier(&n))
}

pub fn is_not_declare_modifier(mod_: &Node) -> bool {
    mod_.kind != SyntaxKind::DeclareKeyword
}

pub fn is_not_export_or_default_or_decorator(node: &Node) -> bool {
    !(is_decorator(node) || node.kind == SyntaxKind::ExportKeyword || node.kind == SyntaxKind::DefaultKeyword)
}

pub fn is_parent_for_idd_diagnostic(node: &Node) -> FindAncestorResult {
    if is_export_assignment(node) {
        return FindAncestorResult::True;
    }
    if is_statement(node) {
        return FindAncestorResult::Quit;
    }
    to_find_ancestor_result(!is_parenthesized_expression(node) && !is_assertion_expression(node))
}
