#![allow(unused_imports)]

#[allow(unused_imports, ambiguous_glob_reexports)]
use crate::checker::*;
#[allow(unused_imports)]
use tsox_frontend::ast::*;
#[allow(unused_imports)]
use tsox_core::diagnostics::messages_generated::*;
use tsox_core::diagnostics::new_ad_hoc_message;

pub(crate) use crate::checker::checker::*;
pub(crate) use crate::checker::mig::m1c::*;
#[allow(unused_imports)]
use tsox_frontend::ast::mig::m3e::{FunctionFlags, get_function_flags};
#[allow(unused_imports)]
use tsox_frontend::ast::mig::x4ast::get_containing_function;
use std::sync::Arc;

use crate::checker::types::*;
use tsox_frontend::ast::{Diagnostic, Node, NodeData, SyntaxKind};

impl Checker {
    pub fn check_yield_expression(&mut self, node: &Arc<Node>) -> Arc<Type> {
        self.check_grammar_yield_expression(node);
        let yield_expression_type = match node_expression(node).cloned() {
            Some(expr) => self.check_expression_ex(&expr, CheckMode::Normal),
            None => self.undefined_widening_type.clone(),
        };
        let fn_node = match get_containing_function(node) {
            Some(f) => f,
            None => return self.any_type(),
        };
        let function_flags = get_function_flags(Some(&fn_node));
        if !function_flags.contains(FunctionFlags::GENERATOR) {
            return self.any_type();
        }
        let is_async = function_flags.contains(FunctionFlags::ASYNC);
        let has_asterisk = matches!(&node.data, NodeData::YieldExpression(data) if data.asterisk_token.is_some());
        if has_asterisk
            && is_async
            && self.language_version
                < crate::checker::mig::m2c::r18k3_defs::LanguageFeatureMinimumTarget::AsyncGenerators
        {
            self.check_external_emit_helpers(
                node,
                (ExternalEmitHelpers::AsyncDelegator | ExternalEmitHelpers::AsyncValues).bits(),
            );
        }
        let mut return_type = Some(self.get_return_type_from_annotation(&fn_node));
        if let Some(rt) = &return_type {
            if rt.flags.contains(TypeFlags::Union) {
                let types = rt.types().unwrap_or(&[]).to_vec();
                let kept: Vec<Arc<Type>> = types
                    .iter()
                    .filter(|t| {
                        self.check_generator_instantiation_assignability_to_return_type_no_error(
                            t, function_flags,
                        )
                    })
                    .cloned()
                    .collect();
                if kept.len() < types.len() {
                    return_type = Some(self.get_union_type_ex(kept, UnionReduction::Subtype));
                }
            }
        }
        let iteration_types = match &return_type {
            Some(rt) => self.iteration_types_of_generator_function_return_type(rt, is_async),
            None => crate::checker::checker_iteration::IterationTypes::default(),
        };
        let signature_yield_type = iteration_types
            .yield_type
            .clone()
            .unwrap_or_else(|| self.any_type());
        let signature_next_type = iteration_types
            .next_type
            .clone()
            .unwrap_or_else(|| self.any_type());
        let yielded_type = self.get_yielded_type_of_yield_expression(
            node,
            &yield_expression_type,
            &signature_next_type,
            is_async,
        );
        if return_type.is_some() {
            let error_node = node_expression(node).cloned().unwrap_or_else(|| Arc::clone(node));
            self.check_type_assignable_to_and_optionally_elaborate(
                &yielded_type,
                &signature_yield_type,
                Some(&error_node),
                Some(&error_node),
                None,
                None,
            );
        }
        if has_asterisk {
            let use_ = if is_async { IterationUse::YieldStar { is_async: true } } else { IterationUse::YieldStar { is_async: false } };
            let expression = node_expression(node).cloned();
            return self
                .get_iteration_type_of_iterable(use_, IterationTypeKind::RETURN, &yield_expression_type, expression.as_deref())
                .unwrap_or_else(|| self.any_type());
        }
        if let Some(return_type) = &return_type {
            return self
                .get_iteration_type_of_generator_function_return_type(IterationTypeKind::NEXT, return_type, is_async)
                .unwrap_or_else(|| self.any_type());
        }
        match self.get_contextual_iteration_type(IterationTypeKind::NEXT, Some(&fn_node)) {
            Some(t) => t,
            None => self.any_type(),
        }
    }

    pub fn container_seems_to_be_empty_dom_element(&self, containing_type: &Arc<Type>) -> bool {
        !self.compiler_options.lib.iter().any(|l| l == "lib.dom.d.ts")
            && every_contained_type(containing_type, &has_common_dom_type_name)
            && self.is_empty_object_type(containing_type)
    }

    pub fn error(
        &mut self,
        location: &Arc<Node>,
        message: &'static str,
        args: &[String],
    ) -> Option<Arc<Diagnostic>> {
        let source_file = self.get_source_file_of_node(location);
        let diagnostic = Diagnostic::new(
            source_file,
            location.loc,
            new_ad_hoc_message(message),
            args.to_vec(),
        );
        self.diagnostics.add(diagnostic);
        None
    }

    pub fn error_message(
        &mut self,
        location: &Arc<Node>,
        message: tsox_core::diagnostics::Message,
        args: &[String],
    ) -> Option<Arc<Diagnostic>> {
        let source_file = self.get_source_file_of_node(location);
        let diagnostic = Diagnostic::new(source_file, location.loc, message, args.to_vec());
        self.diagnostics.add(diagnostic);
        None
    }

    pub fn add_suggestion(
        &mut self,
        location: &Arc<Node>,
        message: &'static str,
        args: &[String],
    ) -> Option<Arc<Diagnostic>> {
        let source_file = self.get_source_file_of_node(location);
        let diagnostic = Diagnostic::new(
            source_file,
            location.loc,
            new_ad_hoc_message(message),
            args.to_vec(),
        );
        self.suggestion_diagnostics.add(diagnostic);
        None
    }

    pub fn add_suggestion_message(
        &mut self,
        location: &Arc<Node>,
        message: tsox_core::diagnostics::Message,
        args: &[String],
    ) -> Option<Arc<Diagnostic>> {
        let source_file = self.get_source_file_of_node(location);
        let diagnostic = Diagnostic::new(source_file, location.loc, message, args.to_vec());
        self.suggestion_diagnostics.add(diagnostic);
        None
    }

    pub fn add_suggestion_for_async(
        &mut self,
        location: &Arc<Node>,
        source_file: &Option<Arc<Node>>,
        diagnostic: &Arc<Diagnostic>,
    ) {
        let _ = location;
        let _ = source_file;
        self.suggestion_diagnostics.add((**diagnostic).clone());
    }

    pub fn error_skipped_on_no_emit(
        &mut self,
        location: &Arc<Node>,
        message: &'static str,
        args: &[String],
    ) -> Option<Arc<Diagnostic>> {
        if !self.compiler_options.no_emit.is_true() {
            return self.error(location, message, args);
        }
        None
    }

    pub fn error_skipped_on_no_emit_message(
        &mut self,
        location: &Arc<Node>,
        message: tsox_core::diagnostics::Message,
        args: &[String],
    ) -> Option<Arc<Diagnostic>> {
        if !self.compiler_options.no_emit.is_true() {
            return self.error_message(location, message, args);
        }
        None
    }

    pub fn error_or_suggestion(
        &mut self,
        is_error: bool,
        location: &Arc<Node>,
        message: &'static str,
        args: &[String],
    ) {
        let diagnostic = if is_error {
            self.error(location, message, args)
        } else {
            self.add_suggestion(location, message, args)
        };
        if let Some(diagnostic) = diagnostic {
            self.add_diagnostic((*diagnostic).clone());
        }
    }

    pub fn error_or_suggestion_message(
        &mut self,
        is_error: bool,
        location: &Arc<Node>,
        message: tsox_core::diagnostics::Message,
        args: &[String],
    ) {
        let diagnostic = if is_error {
            self.error_message(location, message, args)
        } else {
            self.add_suggestion_message(location, message, args)
        };
        if let Some(diagnostic) = diagnostic {
            self.add_diagnostic((*diagnostic).clone());
        }
    }

    pub fn error_and_maybe_suggest_await(
        &mut self,
        location: &Arc<Node>,
        maybe_missing_await: bool,
        message: &'static str,
        args: &[String],
    ) -> Option<Arc<Diagnostic>> {
        let diagnostic = self.error(location, message, args);
        if let (Some(diagnostic), true) = (&diagnostic, maybe_missing_await) {
            let source_file = get_source_file_of_node(location);
            self.add_suggestion_for_async(location, &source_file, diagnostic);
        }
        diagnostic
    }

    pub fn error_and_maybe_suggest_await_message(
        &mut self,
        location: &Arc<Node>,
        maybe_missing_await: bool,
        message: tsox_core::diagnostics::Message,
        args: &[String],
    ) -> Option<Arc<Diagnostic>> {
        let diagnostic = self.error_message(location, message, args);
        if let (Some(diagnostic), true) = (&diagnostic, maybe_missing_await) {
            let source_file = get_source_file_of_node(location);
            self.add_suggestion_for_async(location, &source_file, diagnostic);
        }
        diagnostic
    }
}

pub fn create_diagnostic_for_node(
    node: &Arc<Node>,
    message: &'static str,
    args: &[String],
) -> Arc<Diagnostic> {
    Arc::new(Diagnostic::new(
        None,
        node.loc,
        new_ad_hoc_message(message),
        args.to_vec(),
    ))
}

pub fn create_diagnostic_for_node_message(
    node: &Arc<Node>,
    message: tsox_core::diagnostics::Message,
    args: &[String],
) -> Arc<Diagnostic> {
    Arc::new(Diagnostic::new(None, node.loc, message, args.to_vec()))
}

pub fn has_common_dom_type_name(t: &Arc<Type>) -> bool {
    let symbol = match &t.symbol {
        Some(s) => s,
        None => return false,
    };
    let name = &symbol.name;
    name == "EventTarget"
        || name == "Node"
        || name == "Element"
        || (name.starts_with("HTML") && name.ends_with("Element"))
}

pub fn every_contained_type(t: &Arc<Type>, f: &dyn Fn(&Arc<Type>) -> bool) -> bool {
    match &t.data {
        TypeData::Union(data) => data
            .union_or_intersection
            .types
            .iter()
            .all(|t| every_contained_type(t, f)),
        _ => f(t),
    }
}

pub fn contains_type(types: &[Arc<Type>], t: &Arc<Type>) -> bool {
    types.iter().any(|x| x.id == t.id)
}

pub fn count_types(t: &Arc<Type>) -> usize {
    match &t.data {
        TypeData::Union(data) => data
            .union_or_intersection
            .types
            .iter()
            .map(count_types)
            .sum(),
        _ => 1,
    }
}

pub fn compare_type_ids(t1: &Arc<Type>, t2: &Arc<Type>) -> std::cmp::Ordering {
    t1.id.cmp(&t2.id)
}