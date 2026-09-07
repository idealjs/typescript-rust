use crate::ast::*;
use std::sync::Arc;

pub fn has_syntactic_modifier(node: &Node, flags: ModifierFlags) -> bool {
    node.syntactic_modifier_flags().intersects(flags)
}

pub fn has_accessor_modifier(node: &Node) -> bool {
    has_syntactic_modifier(node, ModifierFlags::Accessor)
}

pub fn has_static_modifier(node: &Node) -> bool {
    has_syntactic_modifier(node, ModifierFlags::Static)
}

pub fn is_static(node: &Node) -> bool {
    (is_class_element(node) && has_static_modifier(node)) || is_class_static_block_declaration(node)
}

pub fn modifier_to_flag(token: SyntaxKind) -> ModifierFlags {
    match token {
        SyntaxKind::StaticKeyword => ModifierFlags::Static,
        SyntaxKind::PublicKeyword => ModifierFlags::Public,
        SyntaxKind::ProtectedKeyword => ModifierFlags::Protected,
        SyntaxKind::PrivateKeyword => ModifierFlags::Private,
        SyntaxKind::AbstractKeyword => ModifierFlags::Abstract,
        SyntaxKind::AccessorKeyword => ModifierFlags::Accessor,
        SyntaxKind::ExportKeyword => ModifierFlags::Export,
        SyntaxKind::DeclareKeyword => ModifierFlags::Ambient,
        SyntaxKind::ConstKeyword => ModifierFlags::Const,
        SyntaxKind::DefaultKeyword => ModifierFlags::Default,
        SyntaxKind::AsyncKeyword => ModifierFlags::Async,
        SyntaxKind::ReadonlyKeyword => ModifierFlags::Readonly,
        SyntaxKind::OverrideKeyword => ModifierFlags::Override,
        SyntaxKind::InKeyword => ModifierFlags::In,
        SyntaxKind::OutKeyword => ModifierFlags::Out,
        SyntaxKind::Decorator => ModifierFlags::Decorator,
        _ => ModifierFlags::empty(),
    }
}

pub fn modifiers_to_flags(modifiers: &[Arc<Node>]) -> ModifierFlags {
    let mut flags = ModifierFlags::empty();
    for modifier in modifiers {
        flags |= modifier_to_flag(modifier.kind);
    }
    flags
}
