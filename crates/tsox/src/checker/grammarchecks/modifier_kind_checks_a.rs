#![allow(unused_imports)]

use super::*;

impl Checker {
    pub(crate) fn check_modifier_kind_a(
        &mut self,
        node: &Arc<Node>,
        modifier: &Arc<Node>,
        flags: &mut ModifierFlags,
        last_static: &mut Option<Arc<Node>>,
        last_override: &mut Option<Arc<Node>>,
    ) -> Option<bool> {
        match modifier.kind {
            SyntaxKind::ConstKeyword => {
                if node.kind != SyntaxKind::EnumDeclaration
                    && node.kind != SyntaxKind::TypeParameter
                {
                    let anchor = match &node.data {
                        NodeData::PropertyDeclaration(d) => Some(Arc::clone(&d.name)),
                        NodeData::MethodDeclaration(d) => Some(Arc::clone(&d.name)),
                        _ => None,
                    };
                    let anchor = anchor.unwrap_or_else(|| Arc::clone(node));
                    return Some(self.grammar_error_on_node_with_args(
                        &anchor,
                        &A_CLASS_MEMBER_CANNOT_HAVE_THE_0_KEYWORD,
                        &["const".to_string()],
                    ));
                }
            }
            SyntaxKind::OverrideKeyword => {
                if flags.contains(ModifierFlags::Override) {
                    return Some(self.grammar_error_on_node_with_args(
                        modifier,
                        &X_0_MODIFIER_ALREADY_SEEN,
                        &["override".to_string()],
                    ));
                } else if flags.contains(ModifierFlags::Ambient) {
                    return Some(self.grammar_error_on_node_with_args(
                        modifier,
                        &X_0_MODIFIER_CANNOT_BE_USED_WITH_1_MODIFIER,
                        &["override".to_string(), "declare".to_string()],
                    ));
                } else if flags.contains(ModifierFlags::Readonly)
                    && !modifier.flags.contains(NodeFlags::Reparsed)
                {
                    return Some(self.grammar_error_on_node_with_args(
                        modifier,
                        &X_0_MODIFIER_MUST_PRECEDE_1_MODIFIER,
                        &["override".to_string(), "readonly".to_string()],
                    ));
                } else if flags.contains(ModifierFlags::Accessor)
                    && !modifier.flags.contains(NodeFlags::Reparsed)
                {
                    return Some(self.grammar_error_on_node_with_args(
                        modifier,
                        &X_0_MODIFIER_MUST_PRECEDE_1_MODIFIER,
                        &["override".to_string(), "accessor".to_string()],
                    ));
                } else if flags.contains(ModifierFlags::Async)
                    && !modifier.flags.contains(NodeFlags::Reparsed)
                {
                    return Some(self.grammar_error_on_node_with_args(
                        modifier,
                        &X_0_MODIFIER_MUST_PRECEDE_1_MODIFIER,
                        &["override".to_string(), "async".to_string()],
                    ));
                }
                *flags |= ModifierFlags::Override;
                *last_override = Some(Arc::clone(modifier));
            }
            SyntaxKind::PublicKeyword
            | SyntaxKind::ProtectedKeyword
            | SyntaxKind::PrivateKeyword => {
                let text = visibility_to_string(modifier.kind);

                if flags.intersects(ModifierFlags::AccessibilityModifier) {
                    return Some(
                        self.grammar_error_on_node(modifier, &ACCESSIBILITY_MODIFIER_ALREADY_SEEN),
                    );
                } else if flags.contains(ModifierFlags::Override)
                    && !modifier.flags.contains(NodeFlags::Reparsed)
                {
                    return Some(self.grammar_error_on_node_with_args(
                        modifier,
                        &X_0_MODIFIER_MUST_PRECEDE_1_MODIFIER,
                        &[text.to_string(), "override".to_string()],
                    ));
                } else if flags.contains(ModifierFlags::Static)
                    && !modifier.flags.contains(NodeFlags::Reparsed)
                {
                    return Some(self.grammar_error_on_node_with_args(
                        modifier,
                        &X_0_MODIFIER_MUST_PRECEDE_1_MODIFIER,
                        &[text.to_string(), "static".to_string()],
                    ));
                } else if flags.contains(ModifierFlags::Accessor)
                    && !modifier.flags.contains(NodeFlags::Reparsed)
                {
                    return Some(self.grammar_error_on_node_with_args(
                        modifier,
                        &X_0_MODIFIER_MUST_PRECEDE_1_MODIFIER,
                        &[text.to_string(), "accessor".to_string()],
                    ));
                } else if flags.contains(ModifierFlags::Readonly)
                    && !modifier.flags.contains(NodeFlags::Reparsed)
                {
                    return Some(self.grammar_error_on_node_with_args(
                        modifier,
                        &X_0_MODIFIER_MUST_PRECEDE_1_MODIFIER,
                        &[text.to_string(), "readonly".to_string()],
                    ));
                } else if flags.contains(ModifierFlags::Async)
                    && !modifier.flags.contains(NodeFlags::Reparsed)
                {
                    return Some(self.grammar_error_on_node_with_args(
                        modifier,
                        &X_0_MODIFIER_MUST_PRECEDE_1_MODIFIER,
                        &[text.to_string(), "async".to_string()],
                    ));
                } else if is_parent_module_block_or_source_file(node) {
                    return Some(self.grammar_error_on_node_with_args(
                        modifier,
                        &X_0_MODIFIER_CANNOT_APPEAR_ON_A_MODULE_OR_NAMESPACE_ELEMENT,
                        &[text.to_string()],
                    ));
                } else if flags.contains(ModifierFlags::Abstract) {
                    if modifier.kind == SyntaxKind::PrivateKeyword {
                        return Some(self.grammar_error_on_node_with_args(
                            modifier,
                            &X_0_MODIFIER_CANNOT_BE_USED_WITH_1_MODIFIER,
                            &[text.to_string(), "abstract".to_string()],
                        ));
                    } else if !modifier.flags.contains(NodeFlags::Reparsed) {
                        return Some(self.grammar_error_on_node_with_args(
                            modifier,
                            &X_0_MODIFIER_MUST_PRECEDE_1_MODIFIER,
                            &[text.to_string(), "abstract".to_string()],
                        ));
                    }
                } else if node
                    .name()
                    .is_some_and(|n| n.kind == SyntaxKind::PrivateIdentifier)
                    && matches!(
                        node.kind,
                        SyntaxKind::PropertyDeclaration
                            | SyntaxKind::MethodDeclaration
                            | SyntaxKind::GetAccessor
                            | SyntaxKind::SetAccessor
                    )
                {
                    return Some(self.grammar_error_on_node(
                        modifier,
                        &AN_ACCESSIBILITY_MODIFIER_CANNOT_BE_USED_WITH_A_PRIVATE_IDENTIFIER,
                    ));
                }
                *flags |= modifier_to_flag(modifier.kind);
            }
            SyntaxKind::StaticKeyword => {
                if flags.contains(ModifierFlags::Static) {
                    return Some(self.grammar_error_on_node_with_args(
                        modifier,
                        &X_0_MODIFIER_ALREADY_SEEN,
                        &["static".to_string()],
                    ));
                } else if flags.contains(ModifierFlags::Readonly)
                    && !modifier.flags.contains(NodeFlags::Reparsed)
                {
                    return Some(self.grammar_error_on_node_with_args(
                        modifier,
                        &X_0_MODIFIER_MUST_PRECEDE_1_MODIFIER,
                        &["static".to_string(), "readonly".to_string()],
                    ));
                } else if flags.contains(ModifierFlags::Async)
                    && !modifier.flags.contains(NodeFlags::Reparsed)
                {
                    return Some(self.grammar_error_on_node_with_args(
                        modifier,
                        &X_0_MODIFIER_MUST_PRECEDE_1_MODIFIER,
                        &["static".to_string(), "async".to_string()],
                    ));
                } else if flags.contains(ModifierFlags::Accessor)
                    && !modifier.flags.contains(NodeFlags::Reparsed)
                {
                    return Some(self.grammar_error_on_node_with_args(
                        modifier,
                        &X_0_MODIFIER_MUST_PRECEDE_1_MODIFIER,
                        &["static".to_string(), "accessor".to_string()],
                    ));
                } else if is_parent_module_block_or_source_file(node) {
                    return Some(self.grammar_error_on_node_with_args(
                        modifier,
                        &X_0_MODIFIER_CANNOT_APPEAR_ON_A_MODULE_OR_NAMESPACE_ELEMENT,
                        &["static".to_string()],
                    ));
                } else if node.kind == SyntaxKind::Parameter {
                    return Some(self.grammar_error_on_node_with_args(
                        modifier,
                        &X_0_MODIFIER_CANNOT_APPEAR_ON_A_PARAMETER,
                        &["static".to_string()],
                    ));
                } else if flags.contains(ModifierFlags::Abstract) {
                    return Some(self.grammar_error_on_node_with_args(
                        modifier,
                        &X_0_MODIFIER_CANNOT_BE_USED_WITH_1_MODIFIER,
                        &["static".to_string(), "abstract".to_string()],
                    ));
                } else if flags.contains(ModifierFlags::Override)
                    && !modifier.flags.contains(NodeFlags::Reparsed)
                {
                    return Some(self.grammar_error_on_node_with_args(
                        modifier,
                        &X_0_MODIFIER_MUST_PRECEDE_1_MODIFIER,
                        &["static".to_string(), "override".to_string()],
                    ));
                }
                *flags |= ModifierFlags::Static;
                *last_static = Some(Arc::clone(modifier));
            }
            SyntaxKind::AccessorKeyword => {
                if flags.contains(ModifierFlags::Accessor) {
                    return Some(self.grammar_error_on_node_with_args(
                        modifier,
                        &X_0_MODIFIER_ALREADY_SEEN,
                        &["accessor".to_string()],
                    ));
                } else if flags.contains(ModifierFlags::Readonly) {
                    return Some(self.grammar_error_on_node_with_args(
                        modifier,
                        &X_0_MODIFIER_CANNOT_BE_USED_WITH_1_MODIFIER,
                        &["accessor".to_string(), "readonly".to_string()],
                    ));
                } else if flags.contains(ModifierFlags::Ambient) {
                    return Some(self.grammar_error_on_node_with_args(
                        modifier,
                        &X_0_MODIFIER_CANNOT_BE_USED_WITH_1_MODIFIER,
                        &["accessor".to_string(), "declare".to_string()],
                    ));
                } else if node.kind != SyntaxKind::PropertyDeclaration {
                    return Some(self.grammar_error_on_node(
                        modifier,
                        &X_ACCESSOR_MODIFIER_CAN_ONLY_APPEAR_ON_A_PROPERTY_DECLARATION,
                    ));
                }
                *flags |= ModifierFlags::Accessor;
            }
            SyntaxKind::ReadonlyKeyword => {
                if flags.contains(ModifierFlags::Readonly) {
                    return Some(self.grammar_error_on_node_with_args(
                        modifier,
                        &X_0_MODIFIER_ALREADY_SEEN,
                        &["readonly".to_string()],
                    ));
                } else if node.kind != SyntaxKind::PropertyDeclaration
                    && node.kind != SyntaxKind::PropertySignature
                    && node.kind != SyntaxKind::IndexSignature
                    && node.kind != SyntaxKind::Parameter
                {
                    return Some(self.grammar_error_on_node(
                        modifier,
                        &X_READONLY_MODIFIER_CAN_ONLY_APPEAR_ON_A_PROPERTY_DECLARATION_OR_INDEX_SIGNATURE,
                    ));
                } else if flags.contains(ModifierFlags::Accessor) {
                    return Some(self.grammar_error_on_node_with_args(
                        modifier,
                        &X_0_MODIFIER_CANNOT_BE_USED_WITH_1_MODIFIER,
                        &["readonly".to_string(), "accessor".to_string()],
                    ));
                }
                *flags |= ModifierFlags::Readonly;
            }
            _ => {}
        }
        None
    }
}
