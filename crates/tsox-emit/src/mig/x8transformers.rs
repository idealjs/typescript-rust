#![allow(dead_code, unused_imports, unused_variables)]

use std::sync::Arc;

use tsox_frontend::ast::{self, Node, ModifierFlags, SyntaxKind};
use tsox_core::core;
use tsox_core::core::text::TextRange;

fn last_modifier_of(node: &Node, pred: impl Fn(&Node) -> bool) -> Option<Arc<Node>> {
    if ast::can_have_modifiers(node) {
        let nodes = node.modifier_nodes();
        return nodes.iter().rev().find(|m| pred(m)).cloned();
    }
    None
}

pub fn move_range_past_decorators(node: &Node) -> TextRange {
    let last_decorator = last_modifier_of(node, |m| ast::is_decorator(m));
    if let Some(last_decorator) = last_decorator {
        return TextRange::new(last_decorator.end(), node.end());
    }
    move_range_past_modifiers(node)
}

pub fn move_range_past_modifiers(node: &Node) -> TextRange {
    if ast::is_property_declaration(node) || ast::is_method_declaration(node) {
        return TextRange::new(node.name().map(|n| n.pos()).unwrap_or(node.pos()), node.end());
    }

    if let Some(last_modifier) = last_modifier_of(node, |_| true) {
        return TextRange::new(last_modifier.end(), node.end());
    }

    if let Some(modifiers) = node.modifiers() {
        if !modifiers.nodes.is_empty() {
            let last_mod = modifiers.nodes.last().unwrap();
            return TextRange::new(last_mod.end(), node.end());
        }
    }
    TextRange::new(node.pos(), node.end())
}

pub fn mask_modifier_flags(node: &Arc<Node>, modifier_mask: ModifierFlags, modifier_additions: ModifierFlags) -> ModifierFlags {
    let mut flags = (ast::get_combined_modifier_flags(node) & modifier_mask) | modifier_additions;
    if flags.contains(ModifierFlags::Default) && !flags.contains(ModifierFlags::Export) {
        flags ^= ModifierFlags::Export;
    }
    flags
}

pub fn needs_scope_marker(result: &Node) -> bool {
    !ast::is_any_import_or_re_export(result)
        && !ast::is_export_assignment(result)
        && !ast::has_syntactic_modifier(result, ModifierFlags::Export)
        && !ast::is_ambient_module(result)
}
