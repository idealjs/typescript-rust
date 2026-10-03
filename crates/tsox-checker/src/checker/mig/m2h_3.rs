#![allow(unused_imports)]

use crate::checker::checker::*;
use crate::checker::symboltracker::*;
use std::collections::HashMap;
use std::sync::Arc;
use crate::checker::jsx_impl_chunk::is_jsx_intrinsic_tag_name;
use tsox_frontend::ast::{self, is_function_expression_or_arrow_function, Node, Symbol, SyntaxKind};

use super::m2h::r21k10_defs::{
    inference_partially_blocked_get, inference_partially_blocked_set, is_call_like_expression,
    is_call_like_or_function_like_expression, skip_direct_inference_nodes_clear,
    skip_direct_inference_nodes_insert,
};
use crate::checker::mig::wc3::NodeAccessExt;

impl Checker {
    pub fn run_with_inference_blocked_from_source_node<T>(
        &mut self,
        node: &Arc<Node>,
        f: impl FnOnce(&mut Checker) -> T,
    ) -> T { ::tsox_core::fntrace::enter("run_with_inference_blocked_from_source_node"); 
        let containing_call = find_ancestor(node, is_call_like_expression);
        if let Some(containing_call) = containing_call {
            let mut to_mark_skip = Arc::clone(node);
            loop {
                skip_direct_inference_nodes_insert(Arc::as_ptr(&to_mark_skip) as *const () as usize);
                match to_mark_skip.parent() {
                    Some(parent) if !Arc::ptr_eq(&parent, &containing_call) => {
                        to_mark_skip = parent;
                    }
                    _ => break,
                }
            }
        }
        inference_partially_blocked_set(true);
        let result = self.run_without_resolved_signature_caching(node, f);
        inference_partially_blocked_set(false);
        skip_direct_inference_nodes_clear();
        result
    }

    pub fn run_without_resolved_signature_caching<T>(
        &mut self,
        node: &Arc<Node>,
        f: impl FnOnce(&mut Checker) -> T,
    ) -> T { ::tsox_core::fntrace::enter("run_without_resolved_signature_caching"); 
        let mut ancestor_node = find_ancestor(node, is_call_like_or_function_like_expression);
        if ancestor_node.is_some() {
            let mut cached_resolved_signatures: Vec<(Arc<Node>, Option<Arc<Signature>>)> =
                Vec::new();
            let mut cached_types: Vec<(Arc<Symbol>, Option<Arc<Type>>)> = Vec::new();
            while let Some(current) = ancestor_node {
                {
                    let signature_links = self.signature_links.get_or_default(&current);
                    cached_resolved_signatures
                        .push((Arc::clone(&current), signature_links.resolved_signature.take()));
                }
                if is_function_expression_or_arrow_function(&current) {
                    if let Some(symbol) = self.get_symbol_of_declaration(&current) {
                        let symbol_links = self.value_symbol_links.get_or_default(&symbol);
                        cached_types.push((symbol, symbol_links.resolved_type.take()));
                    }
                }
                let parent = current.parent();
                ancestor_node = parent
                    .as_ref()
                    .and_then(|p| find_ancestor(p, is_call_like_or_function_like_expression));
            }
            let result = f(self);
            for (cached_node, resolved_signature) in cached_resolved_signatures {
                if let Some(links) = self.signature_links.get_mut(&cached_node) {
                    links.resolved_signature = resolved_signature;
                }
            }
            for (cached_symbol, resolved_type) in cached_types {
                if let Some(links) = self.value_symbol_links.get_mut(&cached_symbol) {
                    links.resolved_type = resolved_type;
                }
            }
            return result;
        }
        f(self)
    }

    pub fn get_local_symbol_for_export_specifier(
        &mut self,
        reference_location: &Arc<Node>,
        reference_symbol: &Arc<Symbol>,
        export_specifier: &Arc<Node>,
    ) -> Arc<Symbol> { ::tsox_core::fntrace::enter("get_local_symbol_for_export_specifier"); 
        if is_export_specifier_alias(reference_location, export_specifier) {
            if let Some(symbol) = self.get_export_specifier_local_target_symbol(export_specifier) {
                return symbol;
            }
        }
        Arc::clone(reference_symbol)
    }

    pub fn get_uninstantiated_signatures(&mut self, node: &Arc<Node>) -> Vec<Arc<Signature>> { ::tsox_core::fntrace::enter("get_uninstantiated_signatures"); 
        match node.kind {
            SyntaxKind::CallExpression | SyntaxKind::Decorator => {
                let expression = node.expression().expect("call expression");
                let expression_type = self.get_type_of_expression(expression);
                self.get_signatures_of_type(&expression_type, SignatureKind::Call)
            }
            SyntaxKind::NewExpression => {
                let expression = node.expression().expect("new expression");
                let expression_type = self.get_type_of_expression(expression);
                self.get_signatures_of_type(&expression_type, SignatureKind::Construct)
            }
            SyntaxKind::JsxSelfClosingElement | SyntaxKind::JsxOpeningElement => {
                let tag_name = tsox_frontend::ast::mig::m3c::tag_name(node);
                if is_jsx_intrinsic_tag_name(tag_name) {
                    Vec::new()
                } else {
                    let tag_type = self.get_type_of_expression(tag_name);
                    self.get_signatures_of_type(&tag_type, SignatureKind::Call)
                }
            }
            SyntaxKind::TaggedTemplateExpression => {
                let tag = node.as_tagged_template_expression().tag.clone();
                let tag_type = self.get_type_of_expression(&tag);
                self.get_signatures_of_type(&tag_type, SignatureKind::Call)
            }
            _ => Vec::new(),
        }
    }

    pub fn get_type_parameter_constraint_for_position_across_signatures(
        &mut self,
        signatures: &[Arc<Signature>],
        position: usize,
    ) -> Arc<Type> { ::tsox_core::fntrace::enter("get_type_parameter_constraint_for_position_across_signatures"); 
        let mut relevant_constraints: Vec<Arc<Type>> = Vec::new();
        for signature in signatures {
            if position >= signature.type_parameters.len() {
                continue;
            }
            let relevant_type_parameter = &signature.type_parameters[position];
            if let Some(constraint) = self.get_constraint_of_type_parameter(relevant_type_parameter) {
                relevant_constraints.push(constraint);
            }
        }
        self.get_union_type(std::mem::take(&mut relevant_constraints))
    }
}

pub fn is_export_specifier_alias(
    reference_location: &Arc<Node>,
    export_specifier: &Arc<Node>,
) -> bool { ::tsox_core::fntrace::enter("is_export_specifier_alias"); 
    let property_name = export_specifier.as_export_specifier().property_name.clone();
    if let Some(property_name) = property_name {
        Arc::ptr_eq(&property_name, reference_location)
    } else {
        export_specifier
            .parent()
            .and_then(|parent| parent.parent())
            .and_then(|grandparent| tsox_frontend::ast::mig::m3b::module_specifier(&grandparent).cloned())
            .is_none()
    }
}

pub fn new_symbol_tracker_impl(
    context: SharedNodeBuilderContext,
    tracker: Option<Box<dyn SymbolTracker>>,
) -> SymbolTrackerImpl { ::tsox_core::fntrace::enter("new_symbol_tracker_impl"); 
    let mut tracker = tracker;
    while let Some(mut current) = tracker {
        if current.as_any().is::<SymbolTrackerImpl>() {
            tracker = current
                .as_any_mut()
                .downcast_mut::<SymbolTrackerImpl>()
                .unwrap()
                .inner
                .take();
        } else {
            tracker = Some(current);
            break;
        }
    }
    SymbolTrackerImpl::new(context, tracker)
}

fn find_ancestor(
    start: &Arc<Node>,
    predicate: impl Fn(&Arc<Node>) -> bool,
) -> Option<Arc<Node>> { ::tsox_core::fntrace::enter("find_ancestor"); 
    let mut current = Some(Arc::clone(start));
    while let Some(node) = current {
        if predicate(&node) {
            return Some(node);
        }
        current = node.parent();
    }
    None
}
