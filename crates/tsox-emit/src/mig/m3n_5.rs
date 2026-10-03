use super::m3m_3::create_get_isolated_declaration_errors;
use super::m3n::*;
use super::m3n_4::{find_ancestor_or_quit, FindAncestorResult};
#[path = "r33k8_defs.rs"]
pub mod r33k8_defs;
pub use r33k8_defs::{DeclarationEmitHost, FileReference, NodeId};
use std::sync::Arc;
use tsox_checker::checker::mig::m2d::EmitResolver;
use tsox_checker::checker::mig::m3a_2::new_diagnostic_for_node;
use tsox_checker::checker::types::{SymbolAccessibility, SymbolAccessibilityResult};
use tsox_core::diagnostics::Message;
use tsox_frontend::ast::diagnostic::Diagnostic;
use tsox_frontend::ast::get_name_of_declaration;
use tsox_frontend::ast::get_source_file_of_node;
use tsox_frontend::ast::mig::m3e_3::get_leftmost_expression;
use tsox_frontend::ast::mig::w7a::is_expando_property_declaration;
use tsox_frontend::ast::node_data_generated::*;
use tsox_frontend::ast::node_node::Node;
use tsox_frontend::ast::SourceFile;
use tsox_frontend::ast::Symbol;
use tsox_frontend::ast::SymbolFlags;
use tsox_frontend::scanner::mig::m3i::{declaration_name_to_string, get_text_of_node};

pub struct SymbolTrackerSharedState {
    pub late_marked_statements: Vec<Arc<Node>>,
    pub diagnostics: Vec<Diagnostic>,
    pub get_symbol_accessibility_diagnostic: Option<GetSymbolAccessibilityDiagnostic>,
    pub error_name_node: Option<Arc<Node>>,
    pub isolated_declarations: bool,
    pub strip_internal: bool,
    pub current_source_file: Option<Arc<SourceFile>>,
    pub report_expando_function_errors: Option<Box<dyn Fn(&Arc<Node>) -> Vec<Diagnostic>>>,
}

impl SymbolTrackerSharedState {
    pub fn add_diagnostic(&mut self, diag: Diagnostic) { ::tsox_core::fntrace::enter("add_diagnostic"); 
        self.diagnostics.push(diag);
    }

    pub fn empty() -> Self { ::tsox_core::fntrace::enter("empty"); 
        SymbolTrackerSharedState {
            late_marked_statements: Vec::new(),
            diagnostics: Vec::new(),
            get_symbol_accessibility_diagnostic: None,
            error_name_node: None,
            isolated_declarations: false,
            strip_internal: false,
            current_source_file: None,
            report_expando_function_errors: None,
        }
    }
}

pub struct SymbolTrackerImpl {
    pub resolver: EmitResolver,
    pub state: SymbolTrackerSharedState,
    pub host: DeclarationEmitHost,
    fallback_stack: Vec<Arc<Node>>,
    pub watched_class_symbol: Option<Arc<Symbol>>,
    pub class_symbol_tracked: bool,
    pub get_isolated_declaration_error: Box<dyn Fn(&Arc<Node>) -> Diagnostic>,
}

impl SymbolTrackerImpl {
    pub fn push_error_fallback_node(&mut self, node: Arc<Node>) { ::tsox_core::fntrace::enter("push_error_fallback_node"); 
        self.fallback_stack.push(node);
    }

    pub fn pop_error_fallback_node(&mut self) { ::tsox_core::fntrace::enter("pop_error_fallback_node"); 
        self.fallback_stack.pop();
    }

    pub fn error_fallback_node(&self) -> Option<&Arc<Node>> { ::tsox_core::fntrace::enter("error_fallback_node"); 
        self.fallback_stack.last()
    }

    pub fn error_location(&self) -> Option<&Arc<Node>> { ::tsox_core::fntrace::enter("error_location"); 
        self.state
            .error_name_node
            .as_ref()
            .or_else(|| self.error_fallback_node())
    }

    pub fn error_declaration_name_with_fallback(&self) -> String { ::tsox_core::fntrace::enter("error_declaration_name_with_fallback"); 
        if let Some(error_name_node) = &self.state.error_name_node {
            return declaration_name_to_string(Some(error_name_node));
        }
        if let Some(fallback) = self.error_fallback_node() {
            if let Some(name) = get_name_of_declaration(fallback) {
                return declaration_name_to_string(Some(&name));
            }
            if is_export_assignment(fallback) {
                let is_export_equals = matches!(&fallback.data, NodeData::ExportAssignment(d) if d.is_export_equals);
                if is_export_equals {
                    return "export=".to_string();
                }
                return "default".to_string();
            }
        }
        "(Missing)".to_string()
    }

    pub fn is_bound_expando(&self, node: &Arc<Node>) -> bool { ::tsox_core::fntrace::enter("is_bound_expando"); 
        let left = match &node.data {
            NodeData::BinaryExpression(d) => &d.left,
            _ => return false,
        };
        if !(is_expando_property_declaration(Some(node)) && is_property_access_expression(left)) {
            return false;
        }
        let leftmost = get_leftmost_expression(left, true);
        let Some(ref_decl) = self.resolver.get_referenced_value_declaration_unsafe(&leftmost)
        else {
            return false;
        };
        self.resolver.is_expando_function_declaration_unsafe(&ref_decl)
    }

    pub fn is_child_of_bound_expando(&self, node: &Arc<Node>) -> bool { ::tsox_core::fntrace::enter("is_child_of_bound_expando"); 
        find_ancestor_or_quit(node, |n| {
            if is_source_file(n) || is_block(n) {
                return FindAncestorResult::Quit;
            }
            super::m3n_4::to_find_ancestor_result(self.is_bound_expando(n))
        })
        .is_some()
    }

    pub fn report_inference_fallback(&mut self, node: &Arc<Node>) { ::tsox_core::fntrace::enter("report_inference_fallback"); 
        if !self.state.isolated_declarations {
            return;
        }
        let node_source = get_source_file_of_node(node);
        let same_file = match (&node_source, self.state.current_source_file.as_ref()) {
            (Some(src), Some(cur)) => Arc::ptr_eq(src, &cur.node),
            (None, None) => true,
            _ => false,
        };
        if !same_file {
            return;
        }
        if self.resolver.is_expando_function_declaration_unsafe(node) {
            let expando_diagnostics = self
                .state
                .report_expando_function_errors
                .as_ref()
                .map(|report| report(node))
                .unwrap_or_default();
            for diag in expando_diagnostics {
                self.state.add_diagnostic(diag);
            }
        }
        if !self.is_child_of_bound_expando(node) {
            let diag = (self.get_isolated_declaration_error)(node);
            self.state.add_diagnostic(diag);
        }
    }

    pub fn track_symbol(
        &mut self,
        symbol: &Arc<Symbol>,
        enclosing_declaration: &Arc<Node>,
        meaning: SymbolFlags,
    ) -> bool { ::tsox_core::fntrace::enter("track_symbol"); 
        if symbol.flags.intersects(SymbolFlags::TypeParameter) {
            return false;
        }
        if let Some(watched) = &self.watched_class_symbol {
            if Arc::ptr_eq(watched, symbol) {
                self.class_symbol_tracked = true;
                return false;
            }
        }
        let result = self
            .resolver
            .is_symbol_accessible(symbol, enclosing_declaration, meaning, true);
        self.handle_symbol_accessibility_error(result)
    }

    pub fn handle_symbol_accessibility_error(
        &mut self,
        symbol_accessibility_result: SymbolAccessibilityResult,
    ) -> bool { ::tsox_core::fntrace::enter("handle_symbol_accessibility_error"); 
        if symbol_accessibility_result.accessibility == SymbolAccessibility::Accessible {
            if !symbol_accessibility_result.aliases_to_make_visible.is_empty() {
                for ref_node in &symbol_accessibility_result.aliases_to_make_visible {
                    if !self
                        .state
                        .late_marked_statements
                        .iter()
                        .any(|n| Arc::ptr_eq(n, ref_node))
                    {
                        self.state.late_marked_statements.push(Arc::clone(ref_node));
                    }
                }
            }
        } else if symbol_accessibility_result.accessibility != SymbolAccessibility::NotResolved {
            let error_info = self
                .state
                .get_symbol_accessibility_diagnostic
                .as_mut()
                .and_then(|get| get(&symbol_accessibility_result));
            if let Some(info) = error_info {
                let diag_node = symbol_accessibility_result
                    .error_node
                    .clone()
                    .or_else(|| info.error_node.clone());
                if let Some(diag_node) = diag_node {
                    if let Some(type_name) = &info.type_name {
                        let text = get_text_of_node(type_name);
                        let mut args = vec![text];
                        args.push(symbol_accessibility_result.error_symbol_name.clone());
                        args.push(symbol_accessibility_result.error_module_name.clone());
                        let diag = create_diagnostic_for_node(
                            &diag_node,
                            Some(info.diagnostic_message),
                            &args,
                        );
                        self.state.add_diagnostic(diag);
                    } else {
                        let mut args = vec![symbol_accessibility_result.error_symbol_name.clone()];
                        args.push(symbol_accessibility_result.error_module_name.clone());
                        let diag =
                            create_diagnostic_for_node(&diag_node, Some(info.diagnostic_message), &args);
                        self.state.add_diagnostic(diag);
                    }
                    return true;
                }
            }
        }
        false
    }
}

pub fn create_diagnostic_for_node(
    node: &Arc<Node>,
    message: Option<&'static Message>,
    args: &[String],
) -> Diagnostic { ::tsox_core::fntrace::enter("create_diagnostic_for_node"); 
    new_diagnostic_for_node(
        Some(node),
        message.copied().expect("diagnostic message must be present"),
        args.to_vec(),
    )
}

pub fn new_symbol_tracker(
    host: DeclarationEmitHost,
    resolver: EmitResolver,
    state: SymbolTrackerSharedState,
) -> SymbolTrackerImpl { ::tsox_core::fntrace::enter("new_symbol_tracker"); 
    let isolated_error_resolver = host.get_emit_resolver();
    SymbolTrackerImpl {
        host,
        resolver,
        state,
        fallback_stack: Vec::new(),
        watched_class_symbol: None,
        class_symbol_tracked: false,
        get_isolated_declaration_error: Box::new(create_get_isolated_declaration_errors(
            isolated_error_resolver,
        )),
    }
}
