use crate::checker::checker::*;
use crate::checker::checker_classes::{
    class_decl_extends_null, class_extends_heritage_element,
};
use tsox_frontend::ast::{Node, NodeData, SyntaxKind, is_class_like};

impl Checker {
    // Go checkSuperExpression 的合法性检查（类型解析在 get_type_of_node）
    pub(crate) fn check_super_expression(&mut self, node: &Arc<Node>) {
        let is_call_expression = node
            .parent()
            .is_some_and(|p| matches!(&p.data, NodeData::CallExpression(c) if Arc::ptr_eq(&c.expression, node)));
        let immediate_container = get_super_container(node, true);
        let mut container = immediate_container.clone();
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
            self.check_legal_super_expression(node, &immediate_container, &container, is_call_expression);
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
        // Go checkSuperExpression 合法分支尾段：TS17011/TS2335/TS2336
        fn check_legal_super_expression(
            &mut self,
            node: &Arc<Node>,
            immediate_container: &Option<Arc<Node>>,
            container: &Option<Arc<Node>>,
            is_call_expression: bool,
        ) {
            if !is_call_expression
                && let Some(immediate) = immediate_container
                && immediate.kind == SyntaxKind::Constructor
            {
                self.check_this_before_super(
                    node,
                    immediate,
                    tsox_core::diagnostics::messages_generated::
                        X_SUPER_MUST_BE_CALLED_BEFORE_ACCESSING_A_PROPERTY_OF_SUPER_IN_THE_CONSTRUCTOR_OF_A_DERIVED_CLASS,
                );
            }
            let Some(container) = container.as_ref() else {
                return;
            };
            let Some(container_parent) = container.parent() else {
                return;
            };
            if container_parent.kind == SyntaxKind::ObjectLiteralExpression {
                return;
            }
            if !is_class_like(container_parent.as_ref()) {
                return;
            }
            if class_extends_heritage_element(&container_parent).is_none() {
                self.diagnostics.add(tsox_frontend::ast::Diagnostic::new(
                    self.current_file.clone(),
                    node.loc,
                    tsox_core::diagnostics::messages_generated::
                        X_SUPER_CAN_ONLY_BE_REFERENCED_IN_A_DERIVED_CLASS,
                    Vec::new(),
                ));
                return;
            }
            if class_decl_extends_null(&container_parent) {
                return;
            }
            if container.kind == SyntaxKind::Constructor
                && is_in_constructor_argument_initializer(node, container)
            {
                self.diagnostics.add(tsox_frontend::ast::Diagnostic::new(
                    self.current_file.clone(),
                    node.loc,
                    tsox_core::diagnostics::messages_generated::
                        X_SUPER_CANNOT_BE_REFERENCED_IN_CONSTRUCTOR_ARGUMENTS,
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

// Go isInConstructorArgumentInitializer：super 位于构造器参数初始化器中
//（遇函数声明即止）
fn is_in_constructor_argument_initializer(node: &Arc<Node>, ctor: &Arc<Node>) -> bool {
    let mut current = node.parent();
    while let Some(n) = current {
        if tsox_frontend::ast::is_function_like_declaration(&n) {
            return false;
        }
        if n.kind == SyntaxKind::Parameter
            && n.parent().is_some_and(|p| Arc::ptr_eq(&p, ctor))
        {
            return true;
        }
        current = n.parent();
    }
    false
}
