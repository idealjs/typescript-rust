use super::builder::NodeBuilderImpl;
use super::recovery::RecoveryBoundary;
use crate::ast::{Node, Symbol, SymbolFlags};
use std::cell::RefCell;
use std::rc::Rc;
use std::sync::Arc;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PropertyNameNodeKind {
    Identifier,
    NumericLiteral,
    StringLiteral,
}

pub fn classify_property_name(
    name: &str,
    string_named: bool,
    is_method: bool,
) -> PropertyNameNodeKind {
    if is_method && name == "new" {
        return PropertyNameNodeKind::StringLiteral;
    }

    if is_identifier_text(name) {
        return PropertyNameNodeKind::Identifier;
    }
    if !string_named && is_numeric_literal_name(name) {
        PropertyNameNodeKind::NumericLiteral
    } else {
        PropertyNameNodeKind::StringLiteral
    }
}

fn is_identifier_text(text: &str) -> bool {
    let mut chars = text.chars();
    match chars.next() {
        Some(c) if c.is_ascii_alphabetic() || c == '_' || c == '$' => {}
        _ => return false,
    }
    chars.all(|c| c.is_ascii_alphanumeric() || c == '_' || c == '$')
}

fn is_numeric_literal_name(name: &str) -> bool {
    !name.is_empty() && name.chars().all(|c| c.is_ascii_digit() || c == '.')
}

pub fn is_external_module_symbol(symbol: &Symbol) -> bool {
    symbol.is_external_module()
}

pub fn get_meaning_of_entity_name_reference(node: &Arc<Node>) -> SymbolFlags {
    SymbolFlags::TYPE
}

pub struct ExistingNodeTreeVisitor {}

impl ExistingNodeTreeVisitor {
    pub fn new(_b: &mut NodeBuilderImpl, _bound: &Rc<RefCell<RecoveryBoundary>>) -> Self {
        ExistingNodeTreeVisitor {}
    }

    pub fn visit_node(&mut self, node: &Arc<Node>) -> Option<Arc<Node>> {
        Some(Arc::clone(node))
    }

    pub fn visit_nodes(&mut self, nodes: &[Arc<Node>]) -> Vec<Arc<Node>> {
        nodes.to_vec()
    }
}
