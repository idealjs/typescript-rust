use std::sync::Arc;

use tsox_frontend::ast::{ModifierFlags, Node, Symbol, SyntaxKind};
use tsox_frontend::ast::has_syntactic_modifier;
use crate::checker::types::Type;

pub fn has_decorators(node: &Node) -> bool {
    has_syntactic_modifier(node, ModifierFlags::Decorator)
}

pub fn is_accessibility_modifier_set(flags: ModifierFlags) -> bool {
    flags.intersects(ModifierFlags::NonPublicAccessibilityModifier)
}

pub fn is_js_doc_non_nullable_type(node: &Node) -> bool {
    node.kind == SyntaxKind::JSDocNonNullableType
}

pub(crate) struct InheritanceInfo {
    pub prop: Arc<Symbol>,
    pub containing_type: Arc<Type>,
}
