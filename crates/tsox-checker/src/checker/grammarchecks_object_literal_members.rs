#![allow(unused_imports)]

use crate::checker::grammarchecks::*;

impl Checker {
    pub(crate) fn check_grammar_object_literal_member_modifiers(
        &mut self,
        properties: &[Arc<Node>],
    ) {
        for prop in properties.iter() {
            if !matches!(
                prop.kind,
                SyntaxKind::MethodDeclaration
                    | SyntaxKind::GetAccessor
                    | SyntaxKind::SetAccessor
                    | SyntaxKind::PropertyAssignment
                    | SyntaxKind::ShorthandPropertyAssignment
            ) {
                continue;
            }
            let modifier_list = prop.modifiers();
            let is_async_only_method = prop.kind == SyntaxKind::MethodDeclaration
                && modifier_list.as_ref().is_some_and(|m| {
                    m.nodes.len() == 1 && m.nodes[0].kind == SyntaxKind::AsyncKeyword
                });
            for modifier in prop.modifier_nodes() {
                if modifier.kind == SyntaxKind::AtToken {
                    continue;
                }
                if prop.kind == SyntaxKind::MethodDeclaration
                    && modifier.kind == SyntaxKind::AsyncKeyword
                {
                    continue;
                }
                self.grammar_error_on_node_with_args(
                    modifier,
                    &X_0_MODIFIER_CANNOT_BE_USED_HERE,
                    &[token_to_string(modifier.kind).to_string()],
                );
            }
            if prop.kind == SyntaxKind::MethodDeclaration
                && modifier_list
                    .as_ref()
                    .is_some_and(|m| !m.nodes.is_empty())
                && !is_async_only_method
            {
                self.grammar_error_on_node(prop, &MODIFIERS_CANNOT_APPEAR_HERE);
                continue;
            }
            if !matches!(
                prop.kind,
                SyntaxKind::MethodDeclaration
                    | SyntaxKind::PropertyAssignment
                    | SyntaxKind::ShorthandPropertyAssignment
            ) {
                continue;
            }
            if let Some(postfix) = prop_postfix_token(prop) {
                let message = if postfix.kind == SyntaxKind::QuestionToken {
                    &AN_OBJECT_MEMBER_CANNOT_BE_DECLARED_OPTIONAL
                } else {
                    &A_DEFINITE_ASSIGNMENT_ASSERTION_IS_NOT_PERMITTED_IN_THIS_CONTEXT
                };
                self.grammar_error_on_node(&postfix, message);
            }
        }
    }
}

fn prop_postfix_token(prop: &Arc<Node>) -> Option<Arc<Node>> {
    match &prop.data {
        NodeData::MethodDeclaration(d) => d.postfix_token.clone(),
        NodeData::PropertyAssignment(d) => d.postfix_token.clone(),
        NodeData::ShorthandPropertyAssignment(d) => d.postfix_token.clone(),
        _ => None,
    }
}
