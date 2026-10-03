use crate::checker::nodecopy_builder::NodeBuilderImpl;
use crate::checker::nodecopy_recovery::RecoveryBoundary;
use std::cell::RefCell;
use std::rc::Rc;
use std::sync::Arc;
use tsox_frontend::ast::Node;
use tsox_frontend::ast::Symbol;
use tsox_frontend::ast::SymbolFlags;

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
) -> PropertyNameNodeKind { ::tsox_core::fntrace::enter("classify_property_name"); 
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

fn is_identifier_text(text: &str) -> bool { ::tsox_core::fntrace::enter("is_identifier_text"); 
    let mut chars = text.chars();
    match chars.next() {
        Some(c) if c.is_ascii_alphabetic() || c == '_' || c == '$' => {}
        _ => return false,
    }
    chars.all(|c| c.is_ascii_alphanumeric() || c == '_' || c == '$')
}

pub fn is_numeric_literal_name(name: &str) -> bool { ::tsox_core::fntrace::enter("is_numeric_literal_name"); 
    !name.is_empty() && name.chars().all(|c| c.is_ascii_digit() || c == '.')
}

pub fn is_external_module_symbol(symbol: &Symbol) -> bool { ::tsox_core::fntrace::enter("is_external_module_symbol"); 
    symbol.is_external_module()
}

pub fn get_meaning_of_entity_name_reference(_node: &Arc<Node>) -> SymbolFlags { ::tsox_core::fntrace::enter("get_meaning_of_entity_name_reference"); 
    SymbolFlags::TYPE
}

pub struct ExistingNodeTreeVisitor {}

impl ExistingNodeTreeVisitor {
    pub fn new(_b: &mut NodeBuilderImpl, _bound: &Rc<RefCell<RecoveryBoundary>>) -> Self { ::tsox_core::fntrace::enter("new"); 
        ExistingNodeTreeVisitor {}
    }

    pub fn visit_node(&mut self, node: &Arc<Node>) -> Option<Arc<Node>> { ::tsox_core::fntrace::enter("visit_node"); 
        Some(Arc::clone(node))
    }

    pub fn visit_nodes(&mut self, nodes: &[Arc<Node>]) -> Vec<Arc<Node>> { ::tsox_core::fntrace::enter("visit_nodes"); 
        nodes.to_vec()
    }
}
