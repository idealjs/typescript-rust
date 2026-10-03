use std::sync::Arc;

use super::m3b::{elements, properties};
use super::m3c_2::*;
use super::m3d::*;
use super::m3f::get_target_of_binding_or_assignment_element;
use super::m3f_3::is_assignment_pattern;
use crate::ast::node::Node;
use crate::ast::node_flags::ModifierFlags;
use crate::ast::syntax_kind_generated::SyntaxKind;

pub fn contains_object_rest_or_spread(node: &Node) -> bool { ::tsox_core::fntrace::enter("contains_object_rest_or_spread"); 
    let facts = node.subtree_facts();
    if facts.intersects(SubtreeFacts::ObjectRestOrSpread) {
        return true;
    }
    if facts.intersects(SubtreeFacts::ESObjectRestOrSpread) {
        let pattern_elements = if node.kind == SyntaxKind::ObjectLiteralExpression {
            properties(node)
        } else {
            elements(node)
        };
        for element in pattern_elements {
            if let Some(target) = get_target_of_binding_or_assignment_element(&element) {
                if is_assignment_pattern(&target) {
                    let target_facts = target.subtree_facts();
                    if target_facts.intersects(SubtreeFacts::ObjectRestOrSpread) {
                        return true;
                    }
                    if target_facts.intersects(SubtreeFacts::ESObjectRestOrSpread)
                        && contains_object_rest_or_spread(&target)
                    {
                        return true;
                    }
                }
            }
        }
    }
    false
}

pub fn create_modifiers_from_modifier_flags(
    flags: ModifierFlags,
    create_modifier: impl Fn(SyntaxKind) -> Arc<Node>,
) -> Vec<Arc<Node>> { ::tsox_core::fntrace::enter("create_modifiers_from_modifier_flags"); 
    let mut result = Vec::new();
    if flags.contains(ModifierFlags::Export) {
        result.push(create_modifier(SyntaxKind::ExportKeyword));
    }
    if flags.contains(ModifierFlags::Ambient) {
        result.push(create_modifier(SyntaxKind::DeclareKeyword));
    }
    if flags.contains(ModifierFlags::Default) {
        result.push(create_modifier(SyntaxKind::DefaultKeyword));
    }
    if flags.contains(ModifierFlags::Const) {
        result.push(create_modifier(SyntaxKind::ConstKeyword));
    }
    if flags.contains(ModifierFlags::Public) {
        result.push(create_modifier(SyntaxKind::PublicKeyword));
    }
    if flags.contains(ModifierFlags::Private) {
        result.push(create_modifier(SyntaxKind::PrivateKeyword));
    }
    if flags.contains(ModifierFlags::Protected) {
        result.push(create_modifier(SyntaxKind::ProtectedKeyword));
    }
    if flags.contains(ModifierFlags::Abstract) {
        result.push(create_modifier(SyntaxKind::AbstractKeyword));
    }
    if flags.contains(ModifierFlags::Static) {
        result.push(create_modifier(SyntaxKind::StaticKeyword));
    }
    if flags.contains(ModifierFlags::Override) {
        result.push(create_modifier(SyntaxKind::OverrideKeyword));
    }
    if flags.contains(ModifierFlags::Readonly) {
        result.push(create_modifier(SyntaxKind::ReadonlyKeyword));
    }
    if flags.contains(ModifierFlags::Accessor) {
        result.push(create_modifier(SyntaxKind::AccessorKeyword));
    }
    if flags.contains(ModifierFlags::Async) {
        result.push(create_modifier(SyntaxKind::AsyncKeyword));
    }
    if flags.contains(ModifierFlags::In) {
        result.push(create_modifier(SyntaxKind::InKeyword));
    }
    if flags.contains(ModifierFlags::Out) {
        result.push(create_modifier(SyntaxKind::OutKeyword));
    }
    result
}
