use super::*;
use crate::ast::node::Node;
use std::sync::Arc;

#[test]
fn symbol_flags_composites() {
    let flags = SymbolFlags::Function.union(SymbolFlags::Class);
    assert!(flags.contains(SymbolFlags::Function));
    assert!(flags.contains(SymbolFlags::Class));
    assert!(!flags.contains(SymbolFlags::Interface));
}

#[test]
fn symbol_creation() {
    let sym = Symbol::new(SymbolFlags::Function, "foo");
    assert_eq!(sym.name, "foo");
    assert!(sym.flags.contains(SymbolFlags::Function));
    assert_eq!(sym.id(), sym.id());
}

#[test]
fn symbol_table_operations() {
    let mut table = SymbolTable::new();
    let sym = Arc::new(Symbol::new(SymbolFlags::VARIABLE, "x"));
    table.insert("x", sym);
    assert!(table.get("x").is_some());
    assert!(table.get("y").is_none());
    assert_eq!(table.len(), 1);
}

#[test]
fn flow_flags() {
    let flags = FlowFlags::START | FlowFlags::ASSIGNMENT;
    assert!(flags.contains(FlowFlags::START));
    assert!(flags.contains(FlowFlags::ASSIGNMENT));
    assert!(!flags.contains(FlowFlags::CALL));
}

#[test]
fn node_symbol_map() {
    let node = Arc::new(Node::new(
        crate::ast::SyntaxKind::Identifier,
        crate::ast::NodeData::Identifier(crate::ast::IdentifierData {
            text: "x".to_string(),
        }),
    ));
    let mut map = NodeSymbolMap::new();
    let sym = Arc::new(Symbol::new(SymbolFlags::VARIABLE, "x"));
    map.set_symbol(&node, Arc::clone(&sym));
    assert!(map.symbol_of(&node).is_some());
    assert_eq!(map.symbol_of(&node).unwrap().name, "x");
}

#[test]
fn container_flags() {
    let flags = ContainerFlags::IS_CONTAINER | ContainerFlags::HAS_LOCALS;
    assert!(flags.contains(ContainerFlags::IS_CONTAINER));
    assert!(flags.contains(ContainerFlags::HAS_LOCALS));
    assert!(!flags.contains(ContainerFlags::IS_INTERFACE));
}
