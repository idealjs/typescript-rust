use crate::ast::{Node, SourceFile, Symbol, SymbolFlags};
use std::cell::RefCell;
use std::rc::Rc;
use std::sync::Arc;

use crate::checker::symboltracker::{SymbolTracker, TrackedSymbolArgs};

pub struct RecoveryBoundary {
    pub ctx: crate::checker::symboltracker::SharedNodeBuilderContext,
    pub had_error: bool,
    pub deferred_reports: Vec<Box<dyn FnOnce()>>,
    pub old_tracker: Option<Box<dyn SymbolTracker>>,
    pub old_tracked_symbols: Vec<TrackedSymbolArgs>,
    pub tracked_symbols: Vec<TrackedSymbolArgs>,
    pub old_encountered_error: bool,
    pub old_approximate_length: usize,
}

impl RecoveryBoundary {
    pub fn mark_error(&mut self, report: Box<dyn FnOnce()>) {
        self.had_error = true;
        self.deferred_reports.push(report);
    }

    pub fn start_recovery_scope(&self) -> OriginalRecoveryScopeState {
        let tracked_symbols_top = self.ctx.borrow().tracked_symbols.len();
        let unreported_errors_top = self.deferred_reports.len();
        OriginalRecoveryScopeState {
            tracked_symbols_top,
            unreported_errors_top,
            had_error: self.had_error,
        }
    }

    pub fn end_recovery_scope(&mut self, state: OriginalRecoveryScopeState) {
        self.had_error = state.had_error;
        let mut ctx = self.ctx.borrow_mut();
        ctx.tracked_symbols.truncate(state.tracked_symbols_top);
        drop(ctx);
        self.deferred_reports.truncate(state.unreported_errors_top);
    }
}

pub struct OriginalRecoveryScopeState {
    pub tracked_symbols_top: usize,
    pub unreported_errors_top: usize,
    pub had_error: bool,
}

pub struct WrappingTracker {
    pub wrapped: Rc<RefCell<Box<dyn SymbolTracker>>>,
    pub bound: Rc<RefCell<RecoveryBoundary>>,
}

impl WrappingTracker {
    pub fn new(inner: Box<dyn SymbolTracker>, bound: Rc<RefCell<RecoveryBoundary>>) -> Self {
        WrappingTracker {
            wrapped: Rc::new(RefCell::new(inner)),
            bound,
        }
    }
}

impl SymbolTracker for WrappingTracker {
    fn track_symbol(
        &mut self,
        symbol: &Arc<Symbol>,
        enclosing_declaration: Option<&Arc<Node>>,
        meaning: SymbolFlags,
    ) -> bool {
        self.bound
            .borrow_mut()
            .tracked_symbols
            .push(TrackedSymbolArgs {
                symbol: Arc::clone(symbol),
                enclosing_declaration: enclosing_declaration.cloned(),
                meaning,
            });
        false
    }

    fn report_inaccessible_this_error(&mut self) {
        let wrapped = Rc::clone(&self.wrapped);
        self.bound.borrow_mut().mark_error(Box::new(move || {
            wrapped.borrow_mut().report_inaccessible_this_error();
        }));
    }

    fn report_private_in_base_of_class_expression(&mut self, property_name: &str) {
        let pn = property_name.to_string();
        let wrapped = Rc::clone(&self.wrapped);
        self.bound.borrow_mut().mark_error(Box::new(move || {
            wrapped
                .borrow_mut()
                .report_private_in_base_of_class_expression(&pn);
        }));
    }

    fn report_inaccessible_unique_symbol_error(&mut self) {
        let wrapped = Rc::clone(&self.wrapped);
        self.bound.borrow_mut().mark_error(Box::new(move || {
            wrapped
                .borrow_mut()
                .report_inaccessible_unique_symbol_error();
        }));
    }

    fn report_cyclic_structure_error(&mut self) {
        let wrapped = Rc::clone(&self.wrapped);
        self.bound.borrow_mut().mark_error(Box::new(move || {
            wrapped.borrow_mut().report_cyclic_structure_error();
        }));
    }

    fn report_likely_unsafe_import_required_error(&mut self, specifier: &str, symbol_name: &str) {
        let sp = specifier.to_string();
        let sn = symbol_name.to_string();
        let wrapped = Rc::clone(&self.wrapped);
        self.bound.borrow_mut().mark_error(Box::new(move || {
            wrapped
                .borrow_mut()
                .report_likely_unsafe_import_required_error(&sp, &sn);
        }));
    }

    fn report_truncation_error(&mut self) {
        self.wrapped.borrow_mut().report_truncation_error();
    }

    fn report_nonlocal_augmentation(
        &mut self,
        containing_file: Option<&Arc<SourceFile>>,
        parent_symbol: &Arc<Symbol>,
        augmenting_symbol: &Arc<Symbol>,
    ) {
        self.wrapped.borrow_mut().report_nonlocal_augmentation(
            containing_file,
            parent_symbol,
            augmenting_symbol,
        );
    }

    fn report_non_serializable_property(&mut self, property_name: &str) {
        let pn = property_name.to_string();
        let wrapped = Rc::clone(&self.wrapped);
        self.bound.borrow_mut().mark_error(Box::new(move || {
            wrapped.borrow_mut().report_non_serializable_property(&pn);
        }));
    }

    fn report_inference_fallback(&mut self, node: &Arc<Node>) {
        self.wrapped.borrow_mut().report_inference_fallback(node);
    }

    fn push_error_fallback_node(&mut self, node: &Arc<Node>) {
        self.wrapped.borrow_mut().push_error_fallback_node(node);
    }

    fn pop_error_fallback_node(&mut self) {
        self.wrapped.borrow_mut().pop_error_fallback_node();
    }
}
