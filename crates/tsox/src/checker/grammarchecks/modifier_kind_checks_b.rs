#![allow(unused_imports)]

use super::*;

impl Checker {
    pub(crate) fn check_modifier_kind_b(
        &mut self,
        node: &Arc<Node>,
        modifier: &Arc<Node>,
        flags: &mut ModifierFlags,
        block_scope_kind: NodeFlags,
        last_async: &mut Option<Arc<Node>>,
        last_declare: &mut Option<Arc<Node>>,
    ) -> Option<bool> {
        match modifier.kind {
            SyntaxKind::ExportKeyword => {
                if flags.contains(ModifierFlags::Export) {
                    return Some(self.grammar_error_on_node_with_args(
                        modifier,
                        &X_0_MODIFIER_ALREADY_SEEN,
                        &["export".to_string()],
                    ));
                } else if flags.contains(ModifierFlags::Ambient)
                    && !modifier.flags.contains(NodeFlags::Reparsed)
                {
                    return Some(self.grammar_error_on_node_with_args(
                        modifier,
                        &X_0_MODIFIER_MUST_PRECEDE_1_MODIFIER,
                        &["export".to_string(), "declare".to_string()],
                    ));
                } else if flags.contains(ModifierFlags::Abstract)
                    && !modifier.flags.contains(NodeFlags::Reparsed)
                {
                    return Some(self.grammar_error_on_node_with_args(
                        modifier,
                        &X_0_MODIFIER_MUST_PRECEDE_1_MODIFIER,
                        &["export".to_string(), "abstract".to_string()],
                    ));
                } else if flags.contains(ModifierFlags::Async)
                    && !modifier.flags.contains(NodeFlags::Reparsed)
                {
                    return Some(self.grammar_error_on_node_with_args(
                        modifier,
                        &X_0_MODIFIER_MUST_PRECEDE_1_MODIFIER,
                        &["export".to_string(), "async".to_string()],
                    ));
                } else if is_parent_class_like(node) {
                    return Some(self.grammar_error_on_node_with_args(
                        modifier,
                        &X_0_MODIFIER_CANNOT_APPEAR_ON_CLASS_ELEMENTS_OF_THIS_KIND,
                        &["export".to_string()],
                    ));
                } else if node.kind == SyntaxKind::Parameter {
                    return Some(self.grammar_error_on_node_with_args(
                        modifier,
                        &X_0_MODIFIER_CANNOT_APPEAR_ON_A_PARAMETER,
                        &["export".to_string()],
                    ));
                }
                *flags |= ModifierFlags::Export;
            }
            SyntaxKind::DefaultKeyword => {
                if block_scope_kind == NodeFlags::Using {
                    return Some(self.grammar_error_on_node_with_args(
                        modifier,
                        &X_0_MODIFIER_CANNOT_APPEAR_ON_A_USING_DECLARATION,
                        &["default".to_string()],
                    ));
                } else if !flags.contains(ModifierFlags::Export)
                    && !modifier.flags.contains(NodeFlags::Reparsed)
                {
                    return Some(self.grammar_error_on_node_with_args(
                        modifier,
                        &X_0_MODIFIER_MUST_PRECEDE_1_MODIFIER,
                        &["export".to_string(), "default".to_string()],
                    ));
                }
                *flags |= ModifierFlags::Default;
            }
            SyntaxKind::DeclareKeyword => {
                if flags.contains(ModifierFlags::Ambient) {
                    return Some(self.grammar_error_on_node_with_args(
                        modifier,
                        &X_0_MODIFIER_ALREADY_SEEN,
                        &["declare".to_string()],
                    ));
                } else if flags.contains(ModifierFlags::Async) {
                    return Some(self.grammar_error_on_node_with_args(
                        modifier,
                        &X_0_MODIFIER_CANNOT_BE_USED_IN_AN_AMBIENT_CONTEXT,
                        &["async".to_string()],
                    ));
                } else if flags.contains(ModifierFlags::Override) {
                    return Some(self.grammar_error_on_node_with_args(
                        modifier,
                        &X_0_MODIFIER_CANNOT_BE_USED_IN_AN_AMBIENT_CONTEXT,
                        &["override".to_string()],
                    ));
                } else if is_parent_class_like(node) && node.kind != SyntaxKind::PropertyDeclaration
                {
                    return Some(self.grammar_error_on_node_with_args(
                        modifier,
                        &X_0_MODIFIER_CANNOT_APPEAR_ON_CLASS_ELEMENTS_OF_THIS_KIND,
                        &["declare".to_string()],
                    ));
                } else if node.kind == SyntaxKind::Parameter {
                    return Some(self.grammar_error_on_node_with_args(
                        modifier,
                        &X_0_MODIFIER_CANNOT_APPEAR_ON_A_PARAMETER,
                        &["declare".to_string()],
                    ));
                }
                *flags |= ModifierFlags::Ambient;
                *last_declare = Some(Arc::clone(modifier));
            }
            SyntaxKind::AbstractKeyword => {
                if flags.contains(ModifierFlags::Abstract) {
                    return Some(self.grammar_error_on_node_with_args(
                        modifier,
                        &X_0_MODIFIER_ALREADY_SEEN,
                        &["abstract".to_string()],
                    ));
                }
                if node.kind != SyntaxKind::ClassDeclaration
                    && node.kind != SyntaxKind::ConstructorType
                {
                    if node.kind != SyntaxKind::MethodDeclaration
                        && node.kind != SyntaxKind::PropertyDeclaration
                        && node.kind != SyntaxKind::GetAccessor
                        && node.kind != SyntaxKind::SetAccessor
                    {
                        return Some(self.grammar_error_on_node(
                            modifier,
                            &X_ABSTRACT_MODIFIER_CAN_ONLY_APPEAR_ON_A_CLASS_METHOD_OR_PROPERTY_DECLARATION,
                        ));
                    }

                    let parent_is_abstract_class = node
                        .parent
                        .as_ref()
                        .map(|p| {
                            p.kind == SyntaxKind::ClassDeclaration
                                && p.has_syntactic_modifier(ModifierFlags::Abstract)
                        })
                        .unwrap_or(false);
                    if !parent_is_abstract_class {
                        let message = if node.kind == SyntaxKind::PropertyDeclaration {
                            &ABSTRACT_PROPERTIES_CAN_ONLY_APPEAR_WITHIN_AN_ABSTRACT_CLASS
                        } else {
                            &ABSTRACT_METHODS_CAN_ONLY_APPEAR_WITHIN_AN_ABSTRACT_CLASS
                        };
                        return Some(self.grammar_error_on_node(modifier, message));
                    }
                    if flags.contains(ModifierFlags::Static) {
                        return Some(self.grammar_error_on_node_with_args(
                            modifier,
                            &X_0_MODIFIER_CANNOT_BE_USED_WITH_1_MODIFIER,
                            &["static".to_string(), "abstract".to_string()],
                        ));
                    }
                    if flags.contains(ModifierFlags::Private) {
                        return Some(self.grammar_error_on_node_with_args(
                            modifier,
                            &X_0_MODIFIER_CANNOT_BE_USED_WITH_1_MODIFIER,
                            &["private".to_string(), "abstract".to_string()],
                        ));
                    }
                }
                *flags |= ModifierFlags::Abstract;
            }
            SyntaxKind::AsyncKeyword => {
                if flags.contains(ModifierFlags::Async) {
                    return Some(self.grammar_error_on_node_with_args(
                        modifier,
                        &X_0_MODIFIER_ALREADY_SEEN,
                        &["async".to_string()],
                    ));
                } else if flags.contains(ModifierFlags::Ambient)
                    || node
                        .parent
                        .as_ref()
                        .map(|p| p.flags.contains(NodeFlags::Ambient))
                        .unwrap_or(false)
                {
                    return Some(self.grammar_error_on_node_with_args(
                        modifier,
                        &X_0_MODIFIER_CANNOT_BE_USED_IN_AN_AMBIENT_CONTEXT,
                        &["async".to_string()],
                    ));
                } else if node.kind == SyntaxKind::Parameter {
                    return Some(self.grammar_error_on_node_with_args(
                        modifier,
                        &X_0_MODIFIER_CANNOT_APPEAR_ON_A_PARAMETER,
                        &["async".to_string()],
                    ));
                } else if flags.contains(ModifierFlags::Abstract) {
                    return Some(self.grammar_error_on_node_with_args(
                        modifier,
                        &X_0_MODIFIER_CANNOT_BE_USED_WITH_1_MODIFIER,
                        &["async".to_string(), "abstract".to_string()],
                    ));
                }
                *flags |= ModifierFlags::Async;
                *last_async = Some(Arc::clone(modifier));
            }
            SyntaxKind::InKeyword | SyntaxKind::OutKeyword => {
                let in_out_flag = if modifier.kind == SyntaxKind::InKeyword {
                    ModifierFlags::In
                } else {
                    ModifierFlags::Out
                };
                let in_out_text = if modifier.kind == SyntaxKind::InKeyword {
                    "in"
                } else {
                    "out"
                };
                if node.kind != SyntaxKind::TypeParameter {
                    return Some(self.grammar_error_on_node_with_args(
                        modifier,
                        &X_0_MODIFIER_CAN_ONLY_APPEAR_ON_A_TYPE_PARAMETER_OF_A_CLASS_INTERFACE_OR_TYPE_ALIAS,
                        &[in_out_text.to_string()],
                    ));
                }
                if flags.contains(in_out_flag) {
                    return Some(self.grammar_error_on_node_with_args(
                        modifier,
                        &X_0_MODIFIER_ALREADY_SEEN,
                        &[in_out_text.to_string()],
                    ));
                }
                if in_out_flag.contains(ModifierFlags::In) && flags.contains(ModifierFlags::Out) {
                    return Some(self.grammar_error_on_node_with_args(
                        modifier,
                        &X_0_MODIFIER_MUST_PRECEDE_1_MODIFIER,
                        &["in".to_string(), "out".to_string()],
                    ));
                }
                *flags |= in_out_flag;
            }
            _ => {}
        }
        None
    }
}
