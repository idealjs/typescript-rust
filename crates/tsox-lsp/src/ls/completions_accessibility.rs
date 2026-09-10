use std::sync::Arc;

use tsox_checker::checker::types::{Type, TypeFlags};
use tsox_frontend::ast::{ModifierFlags, Node, NodeData, Symbol, SyntaxKind};

/// 成员可见性（Go isPropertyAccessible）：非公开成员需位于声明类内，
/// protected 额外允许派生类
pub(super) fn is_property_accessible(
    node: &Arc<Node>,
    containing_type: &Arc<Type>,
    property: &Arc<Symbol>,
) -> bool {
    if containing_type.flags.contains(TypeFlags::Any) {
        return true;
    }
    let Some(declaring_class) = declaring_class_of(property) else {
        return true;
    };
    if is_private_identifier_member(property) {
        return is_node_descendant_of(node, &declaring_class);
    }
    let flags = declaration_modifier_flags(property);
    if !flags.intersects(ModifierFlags::NonPublicAccessibilityModifier) {
        return true;
    }
    if is_node_descendant_of(node, &declaring_class) {
        return true;
    }
    if flags.contains(ModifierFlags::Protected)
        && let Some(owner) = enclosing_class_of(node)
    {
        return class_derives_from(&owner, &declaring_class);
    }
    false
}

fn declaring_class_of(symbol: &Arc<Symbol>) -> Option<Arc<Node>> {
    for decl in &symbol.declarations {
        let mut current = decl.parent.clone();
        while let Some(n) = current {
            if matches!(
                n.kind,
                SyntaxKind::ClassDeclaration | SyntaxKind::ClassExpression
            ) {
                return Some(n);
            }
            current = n.parent.clone();
        }
    }
    None
}

fn class_derives_from(class: &Arc<Node>, target: &Arc<Node>) -> bool {
    let heritage = match &class.data {
        NodeData::ClassDeclaration(d) => d.heritage_clauses.clone(),
        NodeData::ClassExpression(d) => d.heritage_clauses.clone(),
        _ => None,
    };
    let Some(heritage) = heritage else {
        return false;
    };
    for clause in heritage.iter() {
        let types = match &clause.data {
            NodeData::HeritageClause(d) => d.types.clone(),
            _ => continue,
        };
        for entry in types.iter() {
            let expr = match &entry.data {
                NodeData::ExpressionWithTypeArguments(d) => &d.expression,
                _ => entry,
            };
            let base = match &expr.data {
                NodeData::Identifier(d) => d.text.clone(),
                _ => continue,
            };
            if let Some(base_class) = class_named_like(class, &base) {
                if Arc::ptr_eq(&base_class, target) || class_derives_from(&base_class, target) {
                    return true;
                }
            }
        }
    }
    false
}

pub(super) fn class_named_like(scope: &Arc<Node>, name: &str) -> Option<Arc<Node>> {
    let mut current = Some(Arc::clone(scope));
    while let Some(n) = current {
        if n.kind == SyntaxKind::SourceFile {
            let mut found = None;
            tsox_frontend::ast::node_data_generated::for_each_child(&n, |c| {
                if matches!(c.kind, SyntaxKind::ClassDeclaration)
                    && c.name().is_some_and(|nm| nm.text() == name)
                {
                    found = Some(Arc::clone(c));
                    true
                } else {
                    false
                }
            });
            return found;
        }
        current = n.parent.clone();
    }
    None
}

pub(super) fn declaration_modifier_flags(symbol: &Arc<Symbol>) -> ModifierFlags {
    let mut flags = ModifierFlags::empty();
    for decl in &symbol.declarations {
        if let Some(mods) = decl.modifiers() {
            flags |= mods.modifier_flags;
        }
    }
    flags
}

pub(super) fn is_non_public_member(symbol: &Arc<Symbol>) -> bool {
    declaration_modifier_flags(symbol).intersects(ModifierFlags::NonPublicAccessibilityModifier)
}


pub(super) fn is_node_descendant_of(node: &Arc<Node>, ancestor: &Arc<Node>) -> bool {
    let mut current = Some(Arc::clone(node));
    while let Some(n) = current {
        if Arc::ptr_eq(&n, ancestor) {
            return true;
        }
        if n.kind == SyntaxKind::SourceFile {
            return false;
        }
        current = n.parent.clone();
    }
    false
}

pub(super) fn enclosing_class_of(node: &Arc<Node>) -> Option<Arc<Node>> {
    let mut current = node.parent.clone();
    while let Some(n) = current {
        if matches!(
            n.kind,
            SyntaxKind::ClassDeclaration | SyntaxKind::ClassExpression
        ) {
            return Some(n);
        }
        current = n.parent.clone();
    }
    None
}

fn is_private_identifier_member(symbol: &Arc<Symbol>) -> bool {
    symbol
        .declarations
        .iter()
        .any(|d| d.name().is_some_and(|n| n.kind == SyntaxKind::PrivateIdentifier))
}
