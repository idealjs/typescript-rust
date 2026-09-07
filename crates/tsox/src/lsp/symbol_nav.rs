use std::sync::Arc;

pub(super) fn resolve_identifier_symbol(
    symbol_map: &crate::ast::NodeSymbolMap,
    node: &Arc<crate::ast::Node>,
) -> Option<Arc<crate::ast::Symbol>> {
    use crate::ast::{NodeData, SymbolFlags};
    let name = match &node.data {
        NodeData::Identifier(data) => data.text.as_str(),
        _ => return None,
    };
    let mut current: Option<&Arc<crate::ast::Node>> = Some(node);
    while let Some(n) = current {
        if let Some(locals) = symbol_map.locals.get(&n.id()) {
            if let Some(sym) = locals.get(name) {
                return Some(Arc::clone(sym));
            }
        }
        if let Some(container_sym) = symbol_map.symbols.get(&n.id()) {
            if let Some(sym) = container_sym.members.get(name) {
                return Some(Arc::clone(sym));
            }
            if container_sym.flags.intersects(SymbolFlags::MODULE) {
                if let Some(sym) = container_sym.exports.get(name) {
                    return Some(Arc::clone(sym));
                }
            }
        }
        current = n.parent.as_ref();
    }
    None
}

pub(super) fn resolve_symbol_for_node(
    symbol_map: &crate::ast::NodeSymbolMap,
    node: &Arc<crate::ast::Node>,
) -> Option<Arc<crate::ast::Symbol>> {
    if node.kind == crate::ast::SyntaxKind::Identifier {
        if let Some(parent) = node.parent.as_ref() {
            if let Some(name) = parent.name() {
                if Arc::ptr_eq(name, node) {
                    if let Some(sym) = symbol_map.symbol_of(parent) {
                        return Some(Arc::clone(sym));
                    }
                }
            }
        }
    }

    resolve_identifier_symbol(symbol_map, node)
}

pub(super) fn is_declaration_name(
    symbol_map: &crate::ast::NodeSymbolMap,
    node: &Arc<crate::ast::Node>,
) -> bool {
    if node.kind != crate::ast::SyntaxKind::Identifier {
        return false;
    }
    if let Some(parent) = node.parent.as_ref() {
        if let Some(name) = parent.name() {
            if Arc::ptr_eq(name, node) {
                return symbol_map.symbol_of(parent).is_some();
            }
        }
    }
    false
}

pub(super) fn is_property_access_name(node: &Arc<crate::ast::Node>) -> bool {
    use crate::ast::NodeData;
    if node.kind != crate::ast::SyntaxKind::Identifier {
        return false;
    }
    if let Some(parent) = node.parent.as_ref() {
        if let NodeData::PropertyAccessExpression(data) = &parent.data {
            return Arc::ptr_eq(&data.name, node);
        }
    }
    false
}

pub(super) fn walk_all_nodes(
    node: &Arc<crate::ast::Node>,
    visitor: &mut impl FnMut(&Arc<crate::ast::Node>),
) {
    visitor(node);
    let mut children = Vec::new();
    crate::ast::for_each_child(node, |child| {
        children.push(Arc::clone(child));
        false
    });
    for child in children {
        walk_all_nodes(&child, visitor);
    }
}

pub(super) fn find_all_references(
    program: &crate::compiler::Program,
    target_symbol: &Arc<crate::ast::Symbol>,
) -> Vec<(Arc<crate::ast::SourceFile>, Arc<crate::ast::Node>)> {
    use crate::ast::SyntaxKind;
    let symbol_map = program.symbol_map();
    let mut refs = Vec::new();
    for sf in program.source_files() {
        let sf = Arc::clone(sf);
        walk_all_nodes(&sf.node, &mut |node: &Arc<crate::ast::Node>| {
            if node.kind != SyntaxKind::Identifier || is_property_access_name(node) {
                return;
            }
            if let Some(sym) = resolve_symbol_for_node(symbol_map, node) {
                if Arc::ptr_eq(&sym, target_symbol) {
                    refs.push((Arc::clone(&sf), Arc::clone(node)));
                }
            }
        });
    }
    refs
}
