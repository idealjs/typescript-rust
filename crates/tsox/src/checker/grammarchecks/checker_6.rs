#![allow(unused_imports)]

use super::*;

impl Checker {
    pub fn check_grammar_modifiers(&mut self, node: &Arc<Node>) -> bool {
        let modifiers = match node.modifiers() {
            Some(ml) => Arc::clone(ml),
            None => return false,
        };

        if is_this_parameter(node) {
            return self.grammar_error_on_first_token(
                node,
                &NEITHER_DECORATORS_NOR_MODIFIERS_MAY_BE_APPLIED_TO_THIS_PARAMETERS,
            );
        }

        let block_scope_kind = if is_variable_statement(node) {
            if let NodeData::VariableStatement(data) = &node.data {
                data.declaration_list.flags & NodeFlags::BlockScoped
            } else {
                NodeFlags::empty()
            }
        } else {
            NodeFlags::empty()
        };

        let mut flags = ModifierFlags::empty();
        let mut last_static: Option<Arc<Node>> = None;
        let mut last_override: Option<Arc<Node>> = None;
        let mut last_async: Option<Arc<Node>> = None;
        let mut last_declare: Option<Arc<Node>> = None;

        for modifier in &modifiers.list.nodes {
            if modifier.kind == SyntaxKind::Decorator {
                flags |= ModifierFlags::Decorator;
                continue;
            }

            if modifier.kind != SyntaxKind::ReadonlyKeyword {
                if node.kind == SyntaxKind::PropertySignature
                    || node.kind == SyntaxKind::MethodSignature
                {
                    let text = token_to_string(modifier.kind);
                    return self.grammar_error_on_node_with_args(
                        modifier,
                        &X_0_MODIFIER_CANNOT_APPEAR_ON_A_TYPE_MEMBER,
                        &[text.to_string()],
                    );
                }
                if node.kind == SyntaxKind::IndexSignature {
                    let text = token_to_string(modifier.kind);
                    return self.grammar_error_on_node_with_args(
                        modifier,
                        &X_0_MODIFIER_CANNOT_APPEAR_ON_AN_INDEX_SIGNATURE,
                        &[text.to_string()],
                    );
                }
            }

            if modifier.kind != SyntaxKind::InKeyword
                && modifier.kind != SyntaxKind::OutKeyword
                && modifier.kind != SyntaxKind::ConstKeyword
            {
                if node.kind == SyntaxKind::TypeParameter {
                    let text = token_to_string(modifier.kind);
                    return self.grammar_error_on_node_with_args(
                        modifier,
                        &X_0_MODIFIER_CANNOT_APPEAR_ON_A_TYPE_PARAMETER,
                        &[text.to_string()],
                    );
                }
            }

            match modifier.kind {
                SyntaxKind::ConstKeyword
                | SyntaxKind::OverrideKeyword
                | SyntaxKind::PublicKeyword
                | SyntaxKind::ProtectedKeyword
                | SyntaxKind::PrivateKeyword
                | SyntaxKind::StaticKeyword
                | SyntaxKind::AccessorKeyword
                | SyntaxKind::ReadonlyKeyword => {
                    if let Some(result) = self.check_modifier_kind_a(
                        node,
                        modifier,
                        &mut flags,
                        &mut last_static,
                        &mut last_override,
                    ) {
                        return result;
                    }
                }
                SyntaxKind::ExportKeyword
                | SyntaxKind::DefaultKeyword
                | SyntaxKind::DeclareKeyword
                | SyntaxKind::AbstractKeyword
                | SyntaxKind::AsyncKeyword
                | SyntaxKind::InKeyword
                | SyntaxKind::OutKeyword => {
                    if let Some(result) = self.check_modifier_kind_b(
                        node,
                        modifier,
                        &mut flags,
                        block_scope_kind,
                        &mut last_async,
                        &mut last_declare,
                    ) {
                        return result;
                    }
                }
                _ => {}
            }
        }

        self.check_modifier_tail_positions(
            node,
            flags,
            &last_static,
            &last_override,
            &last_async,
            &last_declare,
        )
    }
}
