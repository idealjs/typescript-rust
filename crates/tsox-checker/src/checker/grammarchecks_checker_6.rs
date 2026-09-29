#![allow(unused_imports)]

use crate::checker::grammarchecks::*;

impl Checker {
    pub fn check_grammar_modifiers(&mut self, node: &Arc<Node>) -> bool {
        let modifiers = match node.modifiers() {
            Some(ml) => Arc::clone(ml),
            None => return false,
        };

        if self.report_obvious_decorator_errors(node) || self.report_obvious_modifier_errors(node)
        {
            return true;
        }

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
                if !self.node_can_be_decorated(node) {
                    let no_body = match &node.data {
                        NodeData::MethodDeclaration(d) => d.body.is_none(),
                        _ => false,
                    };
                    return if no_body {
                        self.grammar_error_on_first_token(
                            node,
                            &A_DECORATOR_CAN_ONLY_DECORATE_A_METHOD_IMPLEMENTATION_NOT_AN_OVERLOAD,
                        )
                    } else {
                        self.grammar_error_on_first_token(node, &DECORATORS_ARE_NOT_VALID_HERE)
                    };
                }
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
                // Go：类内 static 索引签名合法（构造侧索引），其余修饰符报错
                if node.kind == SyntaxKind::IndexSignature
                    && !(modifier.kind == SyntaxKind::StaticKeyword
                        && node.parent().is_some_and(|p| {
                            matches!(
                                p.kind,
                                SyntaxKind::ClassDeclaration | SyntaxKind::ClassExpression
                            )
                        }))
                {
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

    // Go ast.NodeCanBeDecorated
    pub(crate) fn node_can_be_decorated(&self, node: &Arc<Node>) -> bool {
        if self.legacy_decorators
            && node.name().is_some_and(|n| n.kind == SyntaxKind::PrivateIdentifier)
        {
            return false;
        }
        let parent = node.parent();
        let grandparent = parent.as_ref().and_then(|p| p.parent());
        match node.kind {
            SyntaxKind::ClassDeclaration => true,
            SyntaxKind::ClassExpression => !self.legacy_decorators,
            SyntaxKind::PropertyDeclaration => parent.is_some_and(|p| {
                (self.legacy_decorators && p.kind == SyntaxKind::ClassDeclaration)
                    || (!self.legacy_decorators
                        && matches!(
                            p.kind,
                            SyntaxKind::ClassDeclaration | SyntaxKind::ClassExpression
                        )
                        && !node.has_syntactic_modifier(ModifierFlags::Abstract)
                        && !node.has_syntactic_modifier(ModifierFlags::Ambient))
            }),
            SyntaxKind::MethodDeclaration
            | SyntaxKind::GetAccessor
            | SyntaxKind::SetAccessor => {
                let has_body = match &node.data {
                    NodeData::MethodDeclaration(d) => d.body.is_some(),
                    NodeData::GetAccessorDeclaration(d) => d.body.is_some(),
                    NodeData::SetAccessorDeclaration(d) => d.body.is_some(),
                    _ => false,
                };
                parent.is_some_and(|p| {
                    has_body
                        && ((self.legacy_decorators && p.kind == SyntaxKind::ClassDeclaration)
                            || (!self.legacy_decorators
                                && matches!(
                                    p.kind,
                                    SyntaxKind::ClassDeclaration | SyntaxKind::ClassExpression
                                )))
                })
            }
            SyntaxKind::Parameter => {
                if !self.legacy_decorators {
                    return false;
                }
                let Some(parent) = parent else {
                    return false;
                };
                let parent_has_body = match &parent.data {
                    NodeData::ConstructorDeclaration(d) => d.body.is_some(),
                    NodeData::MethodDeclaration(d) => d.body.is_some(),
                    NodeData::SetAccessorDeclaration(d) => d.body.is_some(),
                    _ => false,
                };
                parent_has_body
                    && matches!(
                        parent.kind,
                        SyntaxKind::Constructor
                            | SyntaxKind::MethodDeclaration
                            | SyntaxKind::SetAccessor
                    )
                    && !is_this_parameter(node)
                    && grandparent.is_some_and(|g| g.kind == SyntaxKind::ClassDeclaration)
            }
            _ => false,
        }
    }
}
