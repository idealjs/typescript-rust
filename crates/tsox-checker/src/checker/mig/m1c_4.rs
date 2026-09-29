#![allow(unused_imports)]

#[allow(unused_imports, ambiguous_glob_reexports)]
use crate::checker::*;
use super::r27k_defs::NameResolver;
#[allow(unused_imports)]
use tsox_frontend::ast::*;
#[allow(unused_imports)]
use tsox_core::diagnostics::messages_generated::*;

pub(crate) use crate::checker::checker::*;
pub(crate) use crate::checker::mig::m1c::*;
pub(crate) use crate::checker::mig::m1c_2::*;
pub(crate) use crate::checker::mig::m1c_3::*;
#[allow(unused_imports)]
use crate::checker::mig::wc3::CallState;
#[allow(unused_imports)]
use crate::checker::inference_inference_key_2::InferenceFlags;
#[allow(unused_imports)]
use crate::checker::mig::w9a::new_type_mapper;
use std::sync::Arc;

use crate::checker::types::*;
use tsox_frontend::ast::{Diagnostic, Node, NodeData};
use crate::checker::mig::m1e::r20k2_defs::is_in_js_file;

impl Checker {
    pub fn create_name_resolver(&self) -> NameResolver {
        NameResolver {
            compiler_options: self.compiler_options.clone(),
            get_symbol_of_declaration: Self::get_symbol_of_declaration,
            error: Self::error,
            globals: self.globals.clone(),
            arguments_symbol: self.arguments_symbol.clone(),
            require_symbol: self.require_symbol.clone(),
            lookup: Self::get_symbol,
            symbol_referenced: Self::symbol_referenced,
            set_requires_scope_change_cache: Self::set_requires_scope_change_cache,
            get_requires_scope_change_cache: Self::get_requires_scope_change_cache,
            on_property_with_invalid_initializer: Some(Self::check_and_report_error_for_invalid_initializer),
            on_failed_to_resolve_symbol: Some(Self::on_failed_to_resolve_symbol),
            on_successfully_resolved_symbol: Some(Self::on_successfully_resolved_symbol),
        }
    }

    pub fn create_name_resolver_for_suggestion(&self) -> NameResolver {
        NameResolver {
            compiler_options: self.compiler_options.clone(),
            get_symbol_of_declaration: Self::get_symbol_of_declaration,
            error: Self::error,
            globals: self.globals.clone(),
            arguments_symbol: self.arguments_symbol.clone(),
            require_symbol: self.require_symbol.clone(),
            lookup: Self::get_suggestion_for_symbol_name_lookup,
            symbol_referenced: Self::symbol_referenced,
            set_requires_scope_change_cache: Self::set_requires_scope_change_cache,
            get_requires_scope_change_cache: Self::get_requires_scope_change_cache,
            on_property_with_invalid_initializer: None,
            on_failed_to_resolve_symbol: None,
            on_successfully_resolved_symbol: None,
        }
    }

    pub fn choose_overload(&mut self, s: &mut CallState, relation: &Relation) -> Option<Arc<Signature>> {
        s.candidates_for_argument_error = None;
        s.candidate_for_argument_arity_error = None;
        s.candidate_for_type_argument_error = None;
        if s.is_single_non_generic_candidate {
            let candidate = s.candidates[0].clone();
            if !s.type_arguments.is_empty()
                || !self.has_correct_arity(&s.node, &s.args, &candidate, s.signature_help_trailing_comma)
            {
                return None;
            }
            if !self.is_signature_applicable(
                &s.node,
                &s.args,
                &candidate,
                relation,
                CheckMode::Normal,
                false,
                &mut Vec::new(),
            ) {
                s.candidates_for_argument_error = Some(vec![candidate]);
                return None;
            }
            return Some(candidate);
        }
        for candidate_index in 0..s.candidates.len() {
            let candidate = s.candidates[candidate_index].clone();
            if !self.has_correct_type_argument_arity(&candidate, &s.type_arguments)
                || !self.has_correct_arity(&s.node, &s.args, &candidate, s.signature_help_trailing_comma)
            {
                continue;
            }
            let mut check_candidate: Arc<Signature>;
            let mut inference_context: Option<Box<InferenceContext>> = None;
            if !candidate.type_parameters.is_empty() {
                let type_argument_types: Vec<Arc<Type>>;
                if !s.type_arguments.is_empty() {
                    match self.check_type_arguments(&candidate, &s.type_arguments, false, None) {
                        Some(types) => type_argument_types = types,
                        None => {
                            s.candidate_for_type_argument_error = Some(candidate);
                            continue;
                        }
                    }
                } else {
                    let flags = if is_in_js_file(&s.node) {
                        InferenceFlags::AnyDefault
                    } else {
                        InferenceFlags::None
                    };
                    let mut context = self.new_inference_context(&candidate.type_parameters, Some(candidate.clone()), flags, None);
                    let inferred = self.infer_type_arguments(
                        &s.node,
                        &candidate,
                        &s.args,
                        &mut context,
                    );
                    if context.flags.contains(InferenceFlags::SkippedGenericFunction) {
                        s.arg_check_mode |= CheckMode::SkipGenericFunctions;
                    }
                    inference_context = Some(context);
                    type_argument_types = inferred;
                }
                let inferred_type_parameters = inference_context
                    .as_ref()
                    .map(|c| c.inferred_type_parameters.clone())
                    .unwrap_or_default();
                check_candidate = self.get_signature_instantiation(
                    &candidate,
                    &type_argument_types,
                );
                if self.get_non_array_rest_type(&candidate).is_some()
                    && !self.has_correct_arity(&s.node, &s.args, &check_candidate, s.signature_help_trailing_comma)
                {
                    s.candidate_for_argument_arity_error = Some(check_candidate);
                    continue;
                }
            } else {
                check_candidate = candidate.clone();
            }
            if !self.is_signature_applicable(
                &s.node,
                &s.args,
                &check_candidate,
                relation,
                s.arg_check_mode,
                false,
                &mut Vec::new(),
            ) {
                s.candidates_for_argument_error
                    .get_or_insert_with(Vec::new)
                    .push(check_candidate);
                continue;
            }
            if s.arg_check_mode != CheckMode::Normal {
                s.arg_check_mode = CheckMode::Normal;
                if let Some(inference_context) = inference_context.as_mut() {
                    let type_argument_types = self.infer_type_arguments(
                        &s.node,
                        &candidate,
                        &s.args,
                        inference_context,
                    );
                    check_candidate = self.get_signature_instantiation(
                        &candidate,
                        &type_argument_types,
                    );
                    if self.get_non_array_rest_type(&candidate).is_some()
                        && !self.has_correct_arity(&s.node, &s.args, &check_candidate, s.signature_help_trailing_comma)
                    {
                        s.candidate_for_argument_arity_error = Some(check_candidate);
                        continue;
                    }
                }
                if !self.is_signature_applicable(
                    &s.node,
                    &s.args,
                    &check_candidate,
                    relation,
                    s.arg_check_mode,
                    false,
                    &mut Vec::new(),
                ) {
                    s.candidates_for_argument_error
                        .get_or_insert_with(Vec::new)
                        .push(check_candidate);
                    continue;
                }
            }
            s.candidates[candidate_index] = check_candidate.clone();
            return Some(check_candidate);
        }
        None
    }

    pub fn check_type_arguments(
        &mut self,
        signature: &Arc<Signature>,
        type_argument_nodes: &[Arc<Node>],
        report_errors: bool,
        head_message: Option<&'static tsox_core::diagnostics::Message>,
    ) -> Option<Vec<Arc<Type>>> {
        let is_java_script = signature.declaration.as_deref().map(is_in_js_file).unwrap_or(false);
        let type_parameters = signature.type_parameters.clone();
        let mapped: Vec<Arc<Type>> = type_argument_nodes
            .iter()
            .map(|n| self.get_type_from_type_node(n))
            .collect();
        let type_argument_types = self.fill_missing_type_arguments(
            &mapped,
            &type_parameters,
            self.get_min_type_argument_count(&type_parameters),
            is_java_script,
        );
        let mut mapper: Option<Arc<TypeMapper>> = None;
        for i in 0..type_argument_nodes.len() {
            let constraint = self.get_constraint_of_type_parameter(&type_parameters[i]);
            if let Some(constraint) = constraint {
                let type_argument_head_message =
                    head_message.unwrap_or(&TYPE_0_DOES_NOT_SATISFY_THE_CONSTRAINT_1);
                if mapper.is_none() {
                    mapper = Some(Arc::new(new_type_mapper(type_parameters.clone(), type_argument_types.clone())));
                }
                let type_argument = type_argument_types[i].clone();
                let error_node = if report_errors {
                    Some(type_argument_nodes[i].clone())
                } else {
                    None
                };
                let instantiated = self.instantiate_type(&constraint, mapper.as_ref());
                let with_this = self.get_type_with_this_argument(&instantiated, Some(&type_argument), false);
                let mut diags: Vec<Diagnostic> = vec![];
                if !self.check_type_assignable_to_ex(
                    &type_argument,
                    &with_this,
                    error_node.as_ref(),
                    Some(type_argument_head_message),
                    Some(&mut diags),
                ) {
                    if !diags.is_empty() {
                        let mut diagnostic = diags[0].clone();
                        if head_message.is_some() {
                            let mut head = Diagnostic::new(
                                diagnostic.file.clone(),
                                diagnostic.loc,
                                TYPE_0_DOES_NOT_SATISFY_THE_CONSTRAINT_1.clone(),
                                Vec::new(),
                            );
                            head.message_chain = vec![diagnostic];
                            diagnostic = head;
                        }
                        self.add_diagnostic(diagnostic);
                    }
                    return None;
                }
            }
        }
        Some(type_argument_types)
    }

    pub fn create_combined_symbol_from_types(
        &mut self,
        sources: &[Arc<Symbol>],
        types: &[Arc<Type>],
    ) -> Arc<Symbol> {
        let union = self.get_union_type_ex(types.to_vec(), UnionReduction::Subtype);
        self.create_combined_symbol_for_overload_failure(sources, &union)
    }

    pub fn create_combined_symbol_for_overload_failure(
        &mut self,
        sources: &[Arc<Symbol>],
        t: &Arc<Type>,
    ) -> Arc<Symbol> {
        self.create_symbol_with_type(sources.first().unwrap(), t)
    }

}

