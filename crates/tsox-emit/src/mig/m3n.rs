use super::m3m_2::create_get_symbol_accessibility_diagnostic_for_node;
use super::m3n_2::{
    get_accessor_name_visibility_diagnostic_message, get_method_name_visibility_diagnostic_message,
};
use std::sync::Arc;
use tsox_checker::checker::types::{SymbolAccessibility, SymbolAccessibilityResult};
use tsox_core::diagnostics::Message;
use tsox_frontend::ast::get_name_of_declaration;
use tsox_frontend::ast::node_data_generated::*;
use tsox_frontend::ast::node_node::Node;
use tsox_frontend::ast::syntax_kind_generated::SyntaxKind;

pub type DiagnosticSelector =
    fn(&Arc<Node>, &SymbolAccessibilityResult) -> Option<&'static Message>;

pub type GetSymbolAccessibilityDiagnostic =
    Box<dyn Fn(&SymbolAccessibilityResult) -> Option<SymbolAccessibilityDiagnostic>>;

pub struct SymbolAccessibilityDiagnostic {
    pub error_node: Option<Arc<Node>>,
    pub diagnostic_message: &'static Message,
    pub type_name: Option<Arc<Node>>,
}

pub fn wrap_simple_diagnostic_selector(
    node: &Arc<Node>,
    selector: DiagnosticSelector,
) -> GetSymbolAccessibilityDiagnostic {
    let node = Arc::clone(node);
    Box::new(move |symbol_accessibility_result| {
        let diagnostic_message = selector(&node, symbol_accessibility_result)?;
        Some(SymbolAccessibilityDiagnostic {
            error_node: Some(Arc::clone(&node)),
            diagnostic_message,
            type_name: get_name_of_declaration(&node),
        })
    })
}

pub fn wrap_named_diagnostic_selector(
    node: &Arc<Node>,
    selector: DiagnosticSelector,
) -> GetSymbolAccessibilityDiagnostic {
    let node = Arc::clone(node);
    Box::new(move |symbol_accessibility_result| {
        let diagnostic_message = selector(&node, symbol_accessibility_result)?;
        let name = get_name_of_declaration(&node);
        Some(SymbolAccessibilityDiagnostic {
            error_node: name.clone(),
            diagnostic_message,
            type_name: name,
        })
    })
}

pub fn wrap_fallback_error_diagnostic_selector(
    node: &Arc<Node>,
    selector: DiagnosticSelector,
) -> GetSymbolAccessibilityDiagnostic {
    let node = Arc::clone(node);
    Box::new(move |symbol_accessibility_result| {
        let diagnostic_message = selector(&node, symbol_accessibility_result)?;
        let error_node = get_name_of_declaration(&node).unwrap_or_else(|| Arc::clone(&node));
        Some(SymbolAccessibilityDiagnostic {
            error_node: Some(error_node),
            diagnostic_message,
            type_name: None,
        })
    })
}

pub fn select_diagnostic_based_on_module_name(
    symbol_accessibility_result: &SymbolAccessibilityResult,
    module_not_nameable: &'static Message,
    private_module: &'static Message,
    non_module: &'static Message,
) -> Option<&'static Message> {
    if !symbol_accessibility_result.error_module_name.is_empty() {
        if symbol_accessibility_result.accessibility == SymbolAccessibility::CannotBeNamed {
            return Some(module_not_nameable);
        }
        return Some(private_module);
    }
    Some(non_module)
}

pub fn select_diagnostic_based_on_module_name_no_name_check(
    symbol_accessibility_result: &SymbolAccessibilityResult,
    private_module: &'static Message,
    non_module: &'static Message,
) -> Option<&'static Message> {
    if !symbol_accessibility_result.error_module_name.is_empty() {
        return Some(private_module);
    }
    Some(non_module)
}

pub fn create_get_symbol_accessibility_diagnostic_for_node_name(
    node: &Arc<Node>,
) -> GetSymbolAccessibilityDiagnostic {
    if is_set_accessor_declaration(node) || is_get_accessor_declaration(node) {
        wrap_simple_diagnostic_selector(node, get_accessor_name_visibility_diagnostic_message)
    } else if is_method_declaration(node) || is_method_signature_declaration(node) {
        wrap_simple_diagnostic_selector(node, get_method_name_visibility_diagnostic_message)
    } else {
        create_get_symbol_accessibility_diagnostic_for_node(node)
    }
}

pub fn parent_kind_is(node: &Node, kind: SyntaxKind) -> bool {
    matches!(node.parent(), Some(p) if p.kind == kind)
}
