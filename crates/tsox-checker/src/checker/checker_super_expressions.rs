use crate::checker::checker::*;
use tsox_frontend::ast::{Node, NodeData, SyntaxKind, is_class_like};

impl Checker {
    // Go checkSuperExpression 的合法性检查（类型解析在 get_type_of_node）
    pub(crate) fn check_super_expression(&mut self, node: &Arc<Node>) {
        let is_call_expression = node
            .parent()
            .is_some_and(|p| matches!(&p.data, NodeData::CallExpression(c) if Arc::ptr_eq(&c.expression, node)));
        let mut container = get_super_container(node, true);
        if !is_call_expression {
            while container
                .as_ref()
                .is_some_and(|c| c.kind == SyntaxKind::ArrowFunction)
            {
                container = get_super_container(container.as_ref().unwrap(), true);
            }
        }
        let legal = container.as_ref().is_some_and(|c| {
            if is_call_expression {
                return c.kind == SyntaxKind::Constructor;
            }
            let Some(parent) = c.parent() else {
                return false;
            };
            if !(is_class_like(parent.as_ref()) || parent.kind == SyntaxKind::ObjectLiteralExpression)
            {
                return false;
            }
            let static_member = c.has_syntactic_modifier(tsox_frontend::ast::ModifierFlags::Static);
            if static_member {
                return matches!(
                    c.kind,
                    SyntaxKind::MethodDeclaration
                        | SyntaxKind::MethodSignature
                        | SyntaxKind::GetAccessor
                        | SyntaxKind::SetAccessor
                        | SyntaxKind::PropertyDeclaration
                        | SyntaxKind::ClassStaticBlockDeclaration
                );
            }
            matches!(
                c.kind,
                SyntaxKind::MethodDeclaration
                    | SyntaxKind::MethodSignature
                    | SyntaxKind::GetAccessor
                    | SyntaxKind::SetAccessor
                    | SyntaxKind::PropertyDeclaration
                    | SyntaxKind::PropertySignature
                    | SyntaxKind::Constructor
            )
        });
        if legal {
            return;
        }
        let in_computed_name = node.parent().is_some_and(|mut p| loop {
            if let Some(next) = p.parent() {
                p = next;
            } else {
                break false;
            }
            match p.kind {
                SyntaxKind::ComputedPropertyName => break true,
                _ => continue,
            }
        });
        let message = if in_computed_name {
            tsox_core::diagnostics::messages_generated::
                X_SUPER_CANNOT_BE_REFERENCED_IN_A_COMPUTED_PROPERTY_NAME
        } else if is_call_expression {
            tsox_core::diagnostics::messages_generated::
                SUPER_CALLS_ARE_NOT_PERMITTED_OUTSIDE_CONSTRUCTORS_OR_IN_NESTED_FUNCTIONS_INSIDE_CONSTRUCTORS
        } else if container.as_ref().and_then(|c| c.parent()).is_none_or(|p| {
            !(is_class_like(p.as_ref()) || p.kind == SyntaxKind::ObjectLiteralExpression)
        }) {
            tsox_core::diagnostics::messages_generated::
                X_SUPER_CAN_ONLY_BE_REFERENCED_IN_MEMBERS_OF_DERIVED_CLASSES_OR_OBJECT_LITERAL_EXPRESSIONS
        } else {
            tsox_core::diagnostics::messages_generated::
                X_SUPER_PROPERTY_ACCESS_IS_PERMITTED_ONLY_IN_A_CONSTRUCTOR_MEMBER_FUNCTION_OR_MEMBER_ACCESSOR_OF_A_DERIVED_CLASS
        };
        let already = self
            .diagnostics
            .get_all()
            .iter()
            .any(|d| d.code == message.code && d.loc == node.loc);
        if !already {
            self.diagnostics.add(tsox_frontend::ast::Diagnostic::new(
                self.current_file.clone(),
                node.loc,
                message,
                Vec::new(),
            ));
        }
    }
}

// Go ast.GetSuperContainer
pub(crate) fn get_super_container(node: &Arc<Node>, stop_on_functions: bool) -> Option<Arc<Node>> {
    let mut current = node.parent()?;
    loop {
        match current.kind {
            SyntaxKind::ComputedPropertyName => {
                current = current.parent()?;
            }
            SyntaxKind::FunctionDeclaration
            | SyntaxKind::FunctionExpression
            | SyntaxKind::ArrowFunction => {
                if !stop_on_functions {
                    match current.parent() {
                        Some(p) => current = p,
                        None => return None,
                    }
                    continue;
                }
                return Some(current);
            }
            SyntaxKind::PropertyDeclaration
            | SyntaxKind::PropertySignature
            | SyntaxKind::MethodDeclaration
            | SyntaxKind::MethodSignature
            | SyntaxKind::Constructor
            | SyntaxKind::GetAccessor
            | SyntaxKind::SetAccessor
            | SyntaxKind::ClassStaticBlockDeclaration => return Some(current),
            SyntaxKind::Decorator => {
                let decorated = current.parent()?;
                if decorated.kind == SyntaxKind::Parameter
                    && decorated.parent().is_some_and(|gp| {
                        is_class_element_kind(gp.kind)
                    })
                {
                    current = decorated.parent()?;
                } else if is_class_element_kind(decorated.kind) {
                    current = decorated;
                }
            }
            _ => {}
        }
        match current.parent() {
            Some(p) => current = p,
            None => return None,
        }
    }
}

fn is_class_element_kind(kind: SyntaxKind) -> bool {
    matches!(
        kind,
        SyntaxKind::PropertyDeclaration
            | SyntaxKind::PropertySignature
            | SyntaxKind::MethodDeclaration
            | SyntaxKind::MethodSignature
            | SyntaxKind::GetAccessor
            | SyntaxKind::SetAccessor
    )
}
