#![allow(unused_imports)]

use super::*;

impl Checker {
    pub(crate) fn check_modifier_tail_positions(
        &mut self,
        node: &Arc<Node>,
        flags: ModifierFlags,
        last_static: &Option<Arc<Node>>,
        last_override: &Option<Arc<Node>>,
        last_async: &Option<Arc<Node>>,
        last_declare: &Option<Arc<Node>>,
    ) -> bool {
        if node.kind == SyntaxKind::Constructor {
            if flags.contains(ModifierFlags::Static) {
                if let Some(last_static) = &last_static {
                    return self.grammar_error_on_node_with_args(
                        last_static,
                        &X_0_MODIFIER_CANNOT_APPEAR_ON_A_CONSTRUCTOR_DECLARATION,
                        &["static".to_string()],
                    );
                }
            }
            if flags.contains(ModifierFlags::Override) {
                if let Some(last_override) = &last_override {
                    return self.grammar_error_on_node_with_args(
                        last_override,
                        &X_0_MODIFIER_CANNOT_APPEAR_ON_A_CONSTRUCTOR_DECLARATION,
                        &["override".to_string()],
                    );
                }
            }
            if flags.contains(ModifierFlags::Async) {
                if let Some(last_async) = &last_async {
                    return self.grammar_error_on_node_with_args(
                        last_async,
                        &X_0_MODIFIER_CANNOT_APPEAR_ON_A_CONSTRUCTOR_DECLARATION,
                        &["async".to_string()],
                    );
                }
            }
            return false;
        }

        if (node.kind == SyntaxKind::ImportDeclaration
            || node.kind == SyntaxKind::ImportEqualsDeclaration)
            && flags.contains(ModifierFlags::Ambient)
        {
            if let Some(last_declare) = &last_declare {
                return self.grammar_error_on_node_with_args(
                    last_declare,
                    &A_0_MODIFIER_CANNOT_BE_USED_WITH_AN_IMPORT_DECLARATION,
                    &["declare".to_string()],
                );
            }
        }

        if flags.contains(ModifierFlags::Async) {
            if let Some(last_async_node) = &last_async {
                match node.kind {
                    SyntaxKind::MethodDeclaration
                    | SyntaxKind::FunctionDeclaration
                    | SyntaxKind::FunctionExpression
                    | SyntaxKind::ArrowFunction => {}
                    _ => {
                        return self.grammar_error_on_node_with_args(
                            last_async_node,
                            &X_0_MODIFIER_CANNOT_BE_USED_HERE,
                            &["async".to_string()],
                        );
                    }
                }
            }
        }

        false
    }
}
