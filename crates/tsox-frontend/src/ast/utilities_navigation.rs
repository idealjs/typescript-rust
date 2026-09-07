use crate::ast::*;
use std::sync::Arc;

pub fn find_ancestor<F>(node: &Arc<Node>, callback: F) -> Option<Arc<Node>>
where
    F: Fn(&Node) -> bool,
{
    let mut current: Option<&Arc<Node>> = Some(node);
    while let Some(n) = current {
        if callback(n) {
            return Some(Arc::clone(n));
        }
        current = n.parent.as_ref();
    }
    None
}

pub fn find_ancestor_kind(node: &Arc<Node>, kind: SyntaxKind) -> Option<Arc<Node>> {
    find_ancestor(node, |n| n.kind == kind)
}

pub fn get_source_file_of_node(node: &Arc<Node>) -> Option<Arc<Node>> {
    find_ancestor_kind(node, SyntaxKind::SourceFile)
}

pub fn is_node_descendant_of(node: &Arc<Node>, ancestor: &Arc<Node>) -> bool {
    let mut current: Option<&Arc<Node>> = Some(node);
    while let Some(n) = current {
        if Arc::ptr_eq(n, ancestor) {
            return true;
        }
        current = n.parent.as_ref();
    }
    false
}

pub fn get_root_declaration(node: &Arc<Node>) -> Arc<Node> {
    let mut current = Arc::clone(node);
    while current.kind == SyntaxKind::BindingElement {
        match &current.parent {
            Some(parent) => match &parent.parent {
                Some(grandparent) => {
                    current = Arc::clone(grandparent);
                }
                None => break,
            },
            None => break,
        }
    }
    current
}

pub fn get_combined_modifier_flags(node: &Arc<Node>) -> ModifierFlags {
    get_combined_flags(node, |n| n.syntactic_modifier_flags())
}

pub fn get_combined_node_flags(node: &Arc<Node>) -> NodeFlags {
    get_combined_flags(node, |n| n.flags)
}

fn get_combined_flags<F, T: std::ops::BitOr<Output = T>>(node: &Arc<Node>, get_flags: F) -> T
where
    F: Fn(&Node) -> T,
{
    let root = get_root_declaration(node);
    let mut flags = get_flags(&root);
    let mut current = if root.kind == SyntaxKind::VariableDeclaration {
        root.parent.clone()
    } else {
        None
    };
    if let Some(parent) = &current {
        if parent.kind == SyntaxKind::VariableDeclarationList {
            flags = flags | get_flags(parent);
            current = parent.parent.clone();
        }
    }
    if let Some(parent) = &current {
        if parent.kind == SyntaxKind::VariableStatement {
            flags = flags | get_flags(parent);
        }
    }
    flags
}

pub fn get_name_of_declaration(declaration: &Arc<Node>) -> Option<Arc<Node>> {
    let non_assigned = get_non_assigned_name_of_declaration(declaration);
    if non_assigned.is_some() {
        return non_assigned;
    }
    if is_function_expression(declaration)
        || is_arrow_function(declaration)
        || is_class_expression(declaration)
    {
        return get_assigned_name(declaration);
    }
    None
}

fn get_non_assigned_name_of_declaration(declaration: &Arc<Node>) -> Option<Arc<Node>> {
    match declaration.kind {
        SyntaxKind::ExportAssignment => {
            if let Some(expr) = declaration.expression() {
                if is_identifier(expr) {
                    return Some(Arc::clone(expr));
                }
            }
            None
        }
        _ => declaration.name().map(Arc::clone),
    }
}

fn get_assigned_name(node: &Arc<Node>) -> Option<Arc<Node>> {
    let parent = node.parent.as_ref()?;
    match parent.kind {
        SyntaxKind::PropertyAssignment => parent.name().map(Arc::clone),
        SyntaxKind::BindingElement => parent.name().map(Arc::clone),
        SyntaxKind::VariableDeclaration => {
            if let NodeData::VariableDeclaration(d) = &parent.data {
                if is_identifier(&d.name) {
                    return Some(Arc::clone(&d.name));
                }
            }
            None
        }
        _ => None,
    }
}
