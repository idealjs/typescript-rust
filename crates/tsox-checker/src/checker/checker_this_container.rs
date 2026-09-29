use std::sync::Arc;

use super::grammarchecks::is_this_parameter;
use tsox_frontend::ast::{Node, NodeData, SyntaxKind, is_class_like, is_function_like_kind};

/// Go ast.GetThisContainer：向上找 this 的拥有者（跳过箭头函数/装饰器等）
pub(crate) fn get_this_container(
    node: &Arc<Node>,
    include_arrow_functions: bool,
    include_class_computed_property_name: bool,
) -> Arc<Node> {
    let mut current = Arc::clone(node);
    loop {
        let Some(next) = current.parent() else {
            return current;
        };
        current = next;
        match current.kind {
            SyntaxKind::ComputedPropertyName => {
                if include_class_computed_property_name
                    && current
                        .parent()
                        .and_then(|p| p.parent())
                        .is_some_and(|n| is_class_like(n.as_ref()))
                {
                    return current;
                }
                match current.parent().and_then(|p| p.parent()) {
                    Some(skip_to) => current = skip_to,
                    None => return current,
                }
            }
            SyntaxKind::Decorator => {
                let parameter_of_class_element = current
                    .parent()
                    .is_some_and(|p| p.kind == SyntaxKind::Parameter)
                    && current
                        .parent()
                        .and_then(|p| p.parent())
                        .is_some_and(|n| is_class_element_kind(n.kind));
                if parameter_of_class_element {
                    match current.parent().and_then(|p| p.parent()) {
                        Some(skip_to) => current = skip_to,
                        None => return current,
                    }
                } else if current.parent().is_some_and(|n| is_class_element_kind(n.kind)) {
                    match current.parent() {
                        Some(skip_to) => current = skip_to,
                        None => return current,
                    }
                }
            }
            SyntaxKind::ArrowFunction => {
                if !include_arrow_functions {
                    continue;
                }
                return current;
            }
            SyntaxKind::FunctionDeclaration
            | SyntaxKind::FunctionExpression
            | SyntaxKind::ModuleDeclaration
            | SyntaxKind::ClassStaticBlockDeclaration
            | SyntaxKind::PropertyDeclaration
            | SyntaxKind::PropertySignature
            | SyntaxKind::MethodDeclaration
            | SyntaxKind::MethodSignature
            | SyntaxKind::Constructor
            | SyntaxKind::GetAccessor
            | SyntaxKind::SetAccessor
            | SyntaxKind::CallSignature
            | SyntaxKind::ConstructSignature
            | SyntaxKind::IndexSignature
            | SyntaxKind::EnumDeclaration
            | SyntaxKind::SourceFile => return current,
            _ => {}
        }
    }
}

fn is_class_element_kind(kind: SyntaxKind) -> bool {
    matches!(
        kind,
        SyntaxKind::PropertyDeclaration
            | SyntaxKind::MethodDeclaration
            | SyntaxKind::GetAccessor
            | SyntaxKind::SetAccessor
    )
}

/// Go Checker.GetThisParameter：首个 this 参数
pub(crate) fn get_this_parameter(container: &Arc<Node>) -> Option<Arc<Node>> {
    let parameters = match &container.data {
        NodeData::FunctionExpression(d) => &d.parameters,
        NodeData::FunctionDeclaration(d) => &d.parameters,
        NodeData::ArrowFunction(d) => &d.parameters,
        NodeData::MethodDeclaration(d) => &d.parameters,
        NodeData::MethodSignatureDeclaration(d) => &d.parameters,
        NodeData::ConstructorDeclaration(d) => &d.parameters,
        NodeData::GetAccessorDeclaration(d) => &d.parameters,
        NodeData::SetAccessorDeclaration(d) => &d.parameters,
        _ => return None,
    };
    parameters
        .iter()
        .find(|p| is_this_parameter(p) || parameter_named_this(p))
        .cloned()
}

/// Go Checker.isInParameterInitializerBeforeContainingFunction
pub(crate) fn is_in_parameter_initializer_before_containing_function(node: &Arc<Node>) -> bool {
    let mut in_binding_initializer = false;
    let mut current = Arc::clone(node);
    while let Some(parent) = current.parent()
        && !is_function_like_kind(parent.kind)
    {
        if parent.kind == SyntaxKind::Parameter
            && (in_binding_initializer
                || parameter_has_initializer_child(&parent, &current))
        {
            return true;
        }
        if parent.kind == SyntaxKind::BindingElement
            && binding_element_has_initializer_child(&parent, &current)
        {
            in_binding_initializer = true;
        }
        current = parent;
    }
    false
}

fn parameter_has_initializer_child(parameter: &Arc<Node>, node: &Arc<Node>) -> bool {
    match &parameter.data {
        NodeData::ParameterDeclaration(d) => d
            .initializer
            .as_ref()
            .is_some_and(|i| Arc::ptr_eq(i, node)),
        _ => false,
    }
}

fn binding_element_has_initializer_child(element: &Arc<Node>, node: &Arc<Node>) -> bool {
    match &element.data {
        NodeData::BindingElement(d) => d
            .initializer
            .as_ref()
            .is_some_and(|i| Arc::ptr_eq(i, node)),
        _ => false,
    }
}

fn parameter_named_this(param: &Arc<Node>) -> bool {
    match &param.data {
        NodeData::ParameterDeclaration(d) => d.name.kind == SyntaxKind::ThisKeyword,
        _ => false,
    }
}


