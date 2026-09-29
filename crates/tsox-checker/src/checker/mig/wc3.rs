#![allow(unused_imports)]

#[path = "r18k4_node_ext.rs"]
pub mod r18k4_node_ext;
#[path = "r18k4_checker_ext.rs"]
pub mod r18k4_checker_ext;
#[path = "r24k3_defs.rs"]
pub mod r24k3_defs;
#[path = "r25k2_defs.rs"]
pub mod r25k2_defs;
#[path = "r26k2_defs.rs"]
pub mod r26k2_defs;
pub use r18k4_checker_ext::{any_yield_expression, get_actual_type_variable_r18k4};
pub use r18k4_node_ext::NodeAccessExt;

use crate::checker::checker_checker::*;
use crate::checker::utilities_is_private_within_ambient::for_each_yield_expression;
use crate::checker::mig::m1a::accepts_void;
use crate::checker::mig::m2c_3::{signature_has_rest_parameter, some_type};
use crate::checker::mig::wc2::r23k3_defs::some_type_self;
use crate::checker::utilities_has_only_expression_initialization::is_optional_declaration;
use crate::checker::inference_inference_key_2::InferencePriority;
use crate::checker::mig::m3a_2::{new_diagnostic_for_node, new_diagnostic_chain_for_node};
use crate::checker::utilities_is_optional_symbol::is_type_any;
use crate::checker::exports_union_reduction::get_declaration_modifier_flags_from_symbol_ex;
use tsox_frontend::ast::mig::m3e::FunctionFlags;
use tsox_frontend::ast::NodeData;
use std::collections::HashSet;
use std::sync::Arc;
use tsox_core::diagnostics::messages_generated as msg;
use tsox_frontend::ast::{self, Node, Symbol, SyntaxKind};

impl Checker {
    pub fn has_context_sensitive_yield_expression(&mut self, node: &Arc<Node>) -> bool {
        tsox_frontend::ast::mig::m3e::get_function_flags(Some(node))
            .contains(tsox_frontend::ast::mig::m3e::FunctionFlags::GENERATOR)
            && node.body().is_some()
            && {
                let body = node.body().unwrap();
                any_yield_expression(&body, &mut |yield_expr| {
                    self.is_context_sensitive(yield_expr)
                })
            }
    }

    pub fn has_correct_arity(
        &mut self,
        node: &Arc<Node>,
        args: &[Arc<Node>],
        signature: &Arc<Signature>,
        signature_help_trailing_comma: bool,
    ) -> bool {
        if ast::is_jsx_opening_fragment(node) {
            return true;
        }
        let mut arg_count;
        let mut call_is_incomplete = false;
        let mut effective_parameter_count = self.get_parameter_count(signature);
        let mut effective_minimum_arguments = self.get_min_argument_count(signature);
        if ast::is_tagged_template_expression(node) {
            arg_count = args.len();
            let template = node.as_tagged_template_expression().template.clone();
            if ast::is_template_expression(&template) {
                let spans = &template.as_template_expression().template_spans.nodes;
                if let Some(last_span) = spans.last() {
                    let literal = &last_span.as_template_span().literal;
                    call_is_incomplete = ast::node_is_missing(Some(literal))
                        || ast::mig::m3g_3::is_unterminated_literal(literal);
                }
            } else {
                call_is_incomplete = ast::mig::m3g_3::is_unterminated_literal(&template);
            }
        } else if ast::is_decorator(node) {
            arg_count = self.get_decorator_argument_count(node, signature);
        } else if ast::is_binary_expression(node) {
            arg_count = 1;
        } else if ast::mig::m3g::is_jsx_opening_like_element(node) {
            call_is_incomplete = node.attributes().end() == node.end();
            if call_is_incomplete {
                return true;
            }
            arg_count = if effective_minimum_arguments == 0 {
                args.len()
            } else {
                1
            };
            effective_parameter_count = if args.is_empty() {
                effective_parameter_count
            } else {
                1
            };
            effective_minimum_arguments = effective_minimum_arguments.min(1);
        } else if ast::is_new_expression(node) && node.argument_list().is_none() {
            return self.get_min_argument_count(signature) == 0;
        } else {
            if signature_help_trailing_comma {
                arg_count = args.len() + 1;
            } else {
                arg_count = args.len();
            }
            call_is_incomplete = node
                .argument_list()
                .as_ref()
                .is_some_and(|al| al.end() == node.end());
            let spread_arg_index = self.get_spread_argument_index(args);
            if spread_arg_index >= 0 {
                return spread_arg_index >= self.get_min_argument_count(signature) as i64
                    && (self.has_effective_rest_parameter(signature)
                        || spread_arg_index < self.get_parameter_count(signature) as i64);
            }
        }
        if !self.has_effective_rest_parameter(signature) && arg_count > effective_parameter_count {
            return false;
        }
        if call_is_incomplete || arg_count >= effective_minimum_arguments {
            return true;
        }
        for i in arg_count..effective_minimum_arguments {
            let t = self.get_type_at_position(signature, i);
            if self.filter_type(&t, &mut |t: &Arc<Type>| t.flags.contains(TypeFlags::Void))
                .flags
                .contains(TypeFlags::Never)
            {
                return false;
            }
        }
        true
    }

    pub fn has_correct_type_argument_arity(
        &mut self,
        signature: &Arc<Signature>,
        type_arguments: &[Arc<Node>],
    ) -> bool {
        let num_type_parameters = signature.type_parameters.len();
        let min_type_argument_count =
            self.get_min_type_argument_count(&signature.type_parameters);
        type_arguments.is_empty()
            || (type_arguments.len() >= min_type_argument_count
                && type_arguments.len() <= num_type_parameters)
    }

    pub fn has_non_circular_base_constraint(&mut self, t: &Arc<Type>) -> bool {
        let constraint = self.get_resolved_base_constraint(t, &[]);
        !Arc::ptr_eq(&constraint, self.circular_constraint_type.get().unwrap())
    }

    pub fn has_numeric_property_names(&mut self, t: &Arc<Type>) -> bool {
        let number_type = Arc::clone(self.number_type.get().unwrap());
        self.get_index_infos_of_type(t).len() == 1
            && self.get_index_info_of_type(t, &number_type).is_some()
    }

    pub fn has_parent_with_type_annotation(&mut self, symbol: &Arc<Symbol>) -> bool {
        if let Some(parent) = symbol.parent().as_ref() {
            if let Some(value_declaration) = parent.value_declaration.as_ref() {
                if ast::is_function_expression_or_arrow_function(value_declaration) {
                    if let Some(declaration_parent) = value_declaration.parent() {
                        if let Some(possibly_annotated_symbol) =
                            self.get_symbol_of_node(&declaration_parent)
                        {
                            if possibly_annotated_symbol.value_declaration.is_some() {
                                return possibly_annotated_symbol
                                    .value_declaration
                                    .as_ref()
                                    .unwrap()
                                    .typ()
                                    .is_some();
                            }
                        }
                    }
                }
            }
        }
        false
    }

    pub fn infer_from_annotated_parameters_and_return(
        &mut self,
        sig: &Arc<Signature>,
        context: &Arc<Signature>,
        inference_context: &Arc<InferenceContext>,
    ) {
        let length = sig.parameters.len() - if signature_has_rest_parameter(sig) { 1 } else { 0 };
        let inference_context =
            unsafe { &mut *(Arc::as_ptr(inference_context) as *mut InferenceContext) };
        for i in 0..length {
            let declaration = sig.parameters[i].value_declaration.clone().unwrap();
            let type_node = declaration.typ();
            if let Some(type_node) = type_node {
                let annotated = self.get_type_from_type_node(&type_node);
                let source = self.add_optionality_ex(
                    &annotated,
                    false,
                    is_optional_declaration(&declaration),
                );
                let target = self.get_type_at_position(context, i);
                self.infer_types(
                    &mut inference_context.inferences,
                    Some(source),
                    Some(target),
                    InferencePriority::None,
                    false,
                );
            }
        }
        if let Some(declaration) = sig.declaration.as_ref() {
            if let Some(return_type_node) = declaration.typ() {
                let source = self.get_type_from_type_node(&return_type_node);
                let target = self.get_return_type_of_signature(context);
                self.infer_types(
                    &mut inference_context.inferences,
                    Some(source),
                    target,
                    InferencePriority::None,
                    false,
                );
            }
        }
    }

    fn initialize_global_symbols(&mut self) -> (Vec<Arc<Symbol>>, Vec<Vec<Arc<Node>>>) {
        let mut ambient_module_symbols: Vec<Arc<Symbol>> = Vec::new();
        let mut augmentations: Vec<Vec<Arc<Node>>> = Vec::with_capacity(self.files.len());
        let files: Vec<Arc<ast::SourceFile>> = self.files.clone();
        for file in &files {
            if !ast::utilities::is_external_or_common_js_module(file) {
                let file_locals = self
                    .program
                    .symbol_map()
                    .locals_of(&file.node)
                    .cloned();
                if let Some(file_locals) = file_locals {
                    if let Some(file_global_this_symbol) = file_locals.get("globalThis") {
                        for d in &file_global_this_symbol.declarations {
                            self.add_diagnostic(new_diagnostic_for_node(
                                Some(d),
                                msg::DECLARATION_NAME_CONFLICTS_WITH_BUILT_IN_GLOBAL_IDENTIFIER_0,
                                vec!["globalThis".to_string()],
                            ));
                        }
                    }
                    let locals: Vec<Arc<Symbol>> = file_locals.entries.values().cloned().collect();
                    for symbol in locals {
                        if symbol
                            .flags
                            .intersects(SymbolFlags::MODULE)
                            && ast::is_ambient_module_symbol_name(&symbol.name)
                        {
                            ambient_module_symbols.push(symbol);
                        } else {
                            r24k3_defs::merge_global_symbol(self, &symbol);
                        }
                    }
                }
            }
            augmentations.push(file.module_augmentations.clone());
            if self.program.symbol_map().symbol_of(&file.node).is_some() {
                if let NodeData::SourceFile(source_file_data) = &file.node.data {
                    if let Some(global_exports) = &source_file_data.global_exports {
                        for (name, symbol) in &global_exports.entries {
                            if !self.globals.entries.contains_key(name) {
                                self.globals.insert(name.clone(), Arc::clone(symbol));
                            }
                        }
                    }
                }
            }
        }
        for list in &augmentations {
            for augmentation in list {
                if ast::is_global_scope_augmentation(augmentation.parent().as_ref().unwrap()) {
                    self.merge_module_augmentation(augmentation, &augmentation.parent().unwrap());
                }
            }
        }
        self.add_undefined_to_globals_or_error_on_redeclaration();
        (ambient_module_symbols, augmentations)
    }

    pub fn initialize_checker(&mut self) {
        let (ambient_module_symbols, augmentations) = if self.globals_populated {
            (Vec::new(), Vec::new())
        } else {
            self.initialize_global_symbols()
        };
        self.globals_populated = true;
        if let Some(undefined_symbol) = self.undefined_symbol.as_ref() {
            if let Some(links) = self.value_symbol_links.get_mut(undefined_symbol) {
                links.resolved_type = Some(Arc::clone(&self.undefined_widening_type));
            }
        }
        if let Some(arguments_symbol) = self.arguments_symbol.clone() {
            let arguments_type = self.get_global_type("IArguments", 0, true);
            if let Some(links) = self.value_symbol_links.get_mut(&arguments_symbol) {
                links.resolved_type = Some(arguments_type);
            }
        }
        if let Some(unknown_symbol) = self.unknown_symbol.clone() {
            let error_type = self.error_type();
            if let Some(links) = self.value_symbol_links.get_mut(&unknown_symbol) {
                links.resolved_type = Some(error_type);
            }
        }
        if let Some(global_this_symbol) = self.global_this_symbol.clone() {
            let global_this_type =
                self.new_object_type(ObjectFlags::Anonymous, Some(Arc::clone(&global_this_symbol)));
            if let Some(links) = self.value_symbol_links.get_mut(&global_this_symbol) {
                links.resolved_type = Some(global_this_type);
            }
        }
        let global_array = self.get_global_type("Array", 1, true);
        let _ = self.global_array_type.set(global_array);
        let global_object = self.get_global_type("Object", 0, true);
        let _ = self.global_object_type.set(global_object);
        let global_function = self.get_global_type("Function", 0, true);
        let _ = self.global_function_type.set(global_function);
        let global_string = self.get_global_type("String", 0, true);
        let _ = self.global_string_type.set(global_string);
        let global_number = self.get_global_type("Number", 0, true);
        let _ = self.global_number_type.set(global_number);
        let global_boolean = self.get_global_type("Boolean", 0, true);
        let _ = self.global_boolean_type.set(global_boolean);
        let global_reg_exp = self.get_global_type("RegExp", 0, true);
        let _ = self.global_reg_exp_type.set(global_reg_exp);
        let any_type = self.any_type();
        let any_array = self.create_array_type(any_type);
        let _ = self.any_array_type.set(any_array);
        let auto_type = self.auto_type();
        let auto_array = self.create_array_type(auto_type);
        let _ = self.auto_array_type.set(auto_array);
        let empty_object = self.empty_object_type();
        if Arc::ptr_eq(
            self.auto_array_type.get().unwrap(),
            &empty_object,
        ) {
            let auto_array_type = self.new_object_type(ObjectFlags::Anonymous, None);
            self.set_structured_type_members(
                &auto_array_type,
                Some(tsox_frontend::ast::SymbolTable::new()),
                Vec::new(),
                Vec::new(),
                Vec::new(),
            );
            let _ = self.auto_array_type.set(auto_array_type);
        }
        let global_readonly_array = self.get_global_type("ReadonlyArray", 1, false);
        let _ = self.global_readonly_array_type.set(global_readonly_array);
        let empty_generic = self.empty_generic_type();
        if Arc::ptr_eq(
            self.global_readonly_array_type.get().unwrap(),
            &empty_generic,
        ) {
            let _ = self
                .global_readonly_array_type
                .set(Arc::clone(self.global_array_type.get().unwrap()));
        }
        let global_readonly_array_type = Arc::clone(self.global_readonly_array_type.get().unwrap());
        let any_type = self.any_type();
        let any_readonly_array = self.create_type_from_generic_global_type(
            &global_readonly_array_type,
            &[any_type],
        );
        let _ = self.any_readonly_array_type.set(any_readonly_array);
        let global_this = self.get_global_type("ThisType", 1, false);
        let _ = self.global_this_type.set(global_this);
        for symbol in ambient_module_symbols {
            r24k3_defs::merge_global_symbol(self, &symbol);
        }
        self.merge_pattern_ambient_modules();
        for list in &augmentations {
            for augmentation in list {
                if !ast::is_global_scope_augmentation(augmentation.parent().as_ref().unwrap()) {
                    self.merge_module_augmentation(augmentation, &augmentation.parent().unwrap());
                }
            }
        }
    }

    pub fn invocation_error_details(
        &mut self,
        error_target: &Arc<Node>,
        apparent_type: &Arc<Type>,
        kind: SignatureKind,
    ) -> Option<ast::Diagnostic> {
        let mut diagnostic: Option<ast::Diagnostic> = None;
        let is_call = kind == SignatureKind::Call;
        let awaited_type = self.get_awaited_type(apparent_type);
        let maybe_missing_await = awaited_type
            .as_ref()
            .is_some_and(|at| !self.get_signatures_of_type(at, kind).is_empty());
        let mut target = Arc::clone(error_target);
        if ast::is_property_access_expression(error_target)
            && error_target
                .parent()
                .as_ref()
                .is_some_and(|p| ast::is_call_expression(p))
        {
            target = error_target.name().unwrap().clone();
        }
        if apparent_type.flags.contains(TypeFlags::Union) {
            let types = apparent_type.types().unwrap();
            let mut has_signatures = false;
            for constituent in types {
                let signatures = self.get_signatures_of_type(constituent, kind);
                if !signatures.is_empty() {
                    has_signatures = true;
                    if diagnostic.is_some() {
                        break;
                    }
                } else {
                    if diagnostic.is_none() {
                        let head = new_diagnostic_for_node(
                            Some(&target),
                            if is_call {
                                msg::TYPE_0_HAS_NO_CALL_SIGNATURES
                            } else {
                                msg::TYPE_0_HAS_NO_CONSTRUCT_SIGNATURES
                            },
                            vec![self.type_to_string(constituent)],
                        );
                        diagnostic = Some(new_diagnostic_chain_for_node(
                            Some(&head),
                            Some(&target),
                            if is_call {
                                msg::NOT_ALL_CONSTITUENTS_OF_TYPE_0_ARE_CALLABLE
                            } else {
                                msg::NOT_ALL_CONSTITUENTS_OF_TYPE_0_ARE_CONSTRUCTABLE
                            },
                            vec![self.type_to_string(apparent_type)],
                        ));
                    }
                    if has_signatures {
                        break;
                    }
                }
            }
            if !has_signatures {
                diagnostic = Some(new_diagnostic_for_node(
                    Some(&target),
                    if is_call {
                        msg::NO_CONSTITUENT_OF_TYPE_0_IS_CALLABLE
                    } else {
                        msg::NO_CONSTITUENT_OF_TYPE_0_IS_CONSTRUCTABLE
                    },
                    vec![self.type_to_string(apparent_type)],
                ));
            }
            if diagnostic.is_none() {
                diagnostic = Some(new_diagnostic_for_node(
                    Some(&target),
                    if is_call {
                        msg::EACH_MEMBER_OF_THE_UNION_TYPE_0_HAS_SIGNATURES_BUT_NONE_OF_THOSE_SIGNATURES_ARE_COMPATIBLE_WITH_EACH_OTHER
                    } else {
                        msg::EACH_MEMBER_OF_THE_UNION_TYPE_0_HAS_CONSTRUCT_SIGNATURES_BUT_NONE_OF_THOSE_SIGNATURES_ARE_COMPATIBLE_WITH_EACH_OTHER
                    },
                    vec![self.type_to_string(apparent_type)],
                ));
            }
        } else {
            diagnostic = Some(new_diagnostic_for_node(
                Some(&target),
                if is_call {
                    msg::TYPE_0_HAS_NO_CALL_SIGNATURES
                } else {
                    msg::TYPE_0_HAS_NO_CONSTRUCT_SIGNATURES
                },
                vec![self.type_to_string(apparent_type)],
            ));
        }
        let mut head_message = if is_call {
            msg::THIS_EXPRESSION_IS_NOT_CALLABLE
        } else {
            msg::THIS_EXPRESSION_IS_NOT_CONSTRUCTABLE
        };
        if error_target
            .parent()
            .as_ref()
            .is_some_and(|p| {
                ast::is_call_expression(p)
                    && p.arguments().as_ref().is_some_and(|a| a.nodes.is_empty())
            })
        {
            let resolved_symbol = self.get_resolved_symbol_or_nil(error_target);
            if resolved_symbol
                .as_ref()
                .is_some_and(|s| s.flags.intersects(SymbolFlags::GetAccessor))
            {
                head_message = msg::THIS_EXPRESSION_IS_NOT_CALLABLE_BECAUSE_IT_IS_A_GET_ACCESSOR_DID_YOU_MEAN_TO_USE_IT_WITHOUT;
            }
        }
        let mut diagnostic = new_diagnostic_chain_for_node(
            diagnostic.as_ref(),
            Some(&target),
            head_message,
            vec![],
        );
        if maybe_missing_await {
            diagnostic.add_related_info(new_diagnostic_for_node(
                Some(error_target),
                msg::DID_YOU_FORGET_TO_USE_AWAIT,
                vec![],
            ));
        }
        Some(diagnostic)
    }

    pub fn is_array_or_tuple_like_type(&mut self, t: &Arc<Type>) -> bool {
        self.is_array_like_type(t) || self.is_tuple_like_type(t)
    }

    pub fn is_array_or_tuple_or_intersection(&mut self, t: &Arc<Type>) -> bool {
        t.flags.intersects(TypeFlags::Intersection)
            && t.types()
                .unwrap()
                .iter()
                .all(|x| self.is_array_or_tuple_type(x))
    }

    pub fn is_array_or_tuple_symbol(
        &mut self,
        symbol: Option<&Arc<Symbol>>,
    ) -> bool {
        let Some(symbol) = symbol else {
            return false;
        };
        let (Some(global_array_type), Some(global_readonly_array_type)) = (
            self.global_array_type.get(),
            self.global_readonly_array_type.get(),
        ) else {
            return false;
        };
        let (Some(global_array_symbol), Some(global_readonly_array_symbol)) =
            (global_array_type.symbol(), global_readonly_array_type.symbol())
        else {
            return false;
        };
        self.get_symbol_if_same_reference(symbol, &global_array_symbol).is_some()
            || self
                .get_symbol_if_same_reference(symbol, &global_readonly_array_symbol)
                .is_some()
    }

    pub fn is_assignment_to_readonly_entity(
        &mut self,
        expr: &Arc<Node>,
        symbol: &Arc<Symbol>,
        assignment_kind: AssignmentKind,
    ) -> bool {
        if assignment_kind == AssignmentKind::None {
            return false;
        }
        if ast::is_access_expression(expr) {
            let node = ast::skip_parentheses(&expr.expression().unwrap());
            if ast::is_identifier(&node) {
                let expression_symbol = self.get_resolved_symbol(&node);
                if expression_symbol
                    .as_ref()
                    .is_some_and(|s| s.flags.intersects(SymbolFlags::ModuleExports))
                {
                    return false;
                }
            }
        }
        if self.is_readonly_symbol(symbol) {
            if symbol.flags.intersects(SymbolFlags::Property)
                && ast::is_access_expression(expr)
                && expr.expression().unwrap().kind == SyntaxKind::ThisKeyword
            {
                let ctor = self.get_control_flow_container(expr);
                if !ast::is_constructor_declaration(&ctor) {
                    return true;
                }
                if let Some(value_declaration) = symbol.value_declaration.as_ref() {
                    let is_assignment_declaration = ast::is_binary_expression(value_declaration);
                    let is_local_property_declaration = ctor.parent().as_ref().map(Arc::as_ptr)
                        == value_declaration.parent().as_ref().map(Arc::as_ptr);
                    let is_local_parameter_property =
                        Some(Arc::as_ptr(&ctor)) == value_declaration.parent().as_ref().map(Arc::as_ptr);
                    let is_local_this_property_assignment = is_assignment_declaration
                        && symbol
                            .parent()
                            .as_ref()
                            .and_then(|p| p.value_declaration.as_ref())
                            .map(Arc::as_ptr)
                            == ctor.parent().as_ref().map(Arc::as_ptr);
                    let is_local_this_property_assignment_constructor_function =
                        is_assignment_declaration
                            && symbol
                                .parent()
                                .as_ref()
                                .and_then(|p| p.value_declaration.as_ref())
                                .map(Arc::as_ptr)
                                == Some(Arc::as_ptr(&ctor));
                    let is_writeable_symbol = is_local_property_declaration
                        || is_local_parameter_property
                        || is_local_this_property_assignment
                        || is_local_this_property_assignment_constructor_function;
                    return !is_writeable_symbol;
                }
            }
            return true;
        }
        if ast::is_access_expression(expr) {
            let node = ast::skip_parentheses(&expr.expression().unwrap());
            if ast::is_identifier(&node) {
                let expression_symbol = self.get_resolved_symbol(&node);
                if let Some(expression_symbol) = expression_symbol.as_ref() {
                    if expression_symbol.flags.intersects(SymbolFlags::Alias) {
                        let declaration =
                            self.get_declaration_of_alias_symbol(expression_symbol);
                        return declaration
                            .as_ref()
                            .is_some_and(|d| ast::is_namespace_import(d));
                    }
                }
            }
        }
        false
    }

    pub fn is_auto_typed_property(&mut self, symbol: &Arc<Symbol>) -> bool {
        symbol
            .value_declaration
            .as_ref()
            .is_some_and(|declaration| {
                ast::is_property_declaration(declaration)
                    && declaration.typ().is_none()
                    && declaration.initializer().is_none()
                    && self.no_implicit_any
            })
    }

    pub fn is_awaited_type_instantiation(&mut self, t: &Arc<Type>) -> bool {
        if t.flags.contains(TypeFlags::Conditional) {
            let awaited_symbol = self.get_global_awaited_symbol_or_nil();
            return awaited_symbol.is_some()
                && t.alias.is_some()
                && Arc::ptr_eq(t.alias.as_ref().unwrap().symbol.as_ref().unwrap(), awaited_symbol.as_ref().unwrap())
                && t.alias.as_ref().unwrap().type_arguments.len() == 1;
        }
        false
    }

    pub fn is_awaited_type_needed(&mut self, t: &Arc<Type>) -> bool {
        if is_type_any(t) || self.is_awaited_type_instantiation(t) {
            return false;
        }
        if self.is_generic_object_type(t) {
            let base_constraint = self.get_base_constraint_of_type(t);
            if let Some(base_constraint) = base_constraint {
                return base_constraint.flags.intersects(crate::checker::types_type_id::TYPE_FLAGS_ANY_OR_UNKNOWN)
                    || self.is_empty_object_type(&base_constraint)
                    || some_type_self(self, &base_constraint, |c, x| c.is_thenable_type(x));
            }
            return self.maybe_type_of_kind(t, crate::checker::types_type_id::TYPE_FLAGS_TYPE_VARIABLE);
        }
        false
    }

    pub fn is_circular_mapped_property(&mut self, symbol: &Arc<Symbol>) -> bool {
        if symbol.check_flags.intersects(CheckFlags::Mapped) {
            let links = self.value_symbol_links.get(symbol);
            return links.is_some_and(|l| l.resolved_type.is_none())
                && self.find_resolution_cycle_start_index(
                    TypeSystemEntity::Symbol(Arc::clone(symbol)),
                    TypeSystemPropertyName::Type,
                ) >= 0;
        }
        false
    }

    pub fn is_class_derived_from_declaring_classes(
        &mut self,
        check_class: &Arc<Type>,
        prop: &Arc<Symbol>,
        writing: bool,
    ) -> bool {
        !self.for_each_property(prop, &mut |c: &mut Checker, p: &Arc<Symbol>| {
            if get_declaration_modifier_flags_from_symbol_ex(p, writing)
                .intersects(ModifierFlags::Protected)
            {
                let declaring_class = c.get_declaring_class(p);
                return !c.has_base_type(check_class, declaring_class.as_ref().unwrap());
            }
            false
        })
    }

    pub fn is_commonjs_require(&mut self, node: &Arc<Node>) -> bool {
        if !ast::is_require_call(node, true) {
            return false;
        }
        let expression = node.expression().unwrap();
        if !ast::is_identifier(&expression) {
            panic!("Expected identifier for require call");
        }
        let resolved_require = self.resolve_name(
            &expression.text(),
            &expression,
            SymbolFlags::VALUE,
            true,
        );
        if let Some(resolved_require) = &resolved_require {
            if self.require_symbol.is_some()
                && Arc::ptr_eq(resolved_require, self.require_symbol.as_ref().unwrap())
            {
                return true;
            }
        } else {
            return false;
        }
        let resolved_require = resolved_require.unwrap();
        if resolved_require.flags.intersects(SymbolFlags::Alias) {
            return false;
        }
        let target_declaration_kind = if resolved_require.flags.intersects(SymbolFlags::Function) {
            Some(SyntaxKind::FunctionDeclaration)
        } else if resolved_require.flags.intersects(SymbolFlags::VARIABLE) {
            Some(SyntaxKind::VariableDeclaration)
        } else {
            None
        };
        if let Some(kind) = target_declaration_kind {
            let decl = tsox_frontend::ast::mig::m3e_4::get_declaration_of_kind(&resolved_require, kind);
            return decl
                .as_ref()
                .is_some_and(|d| d.flags.contains(NodeFlags::Ambient));
        }
        false
    }
}

pub fn is_conflicting_private_property(prop: &Arc<Symbol>) -> bool {
    prop.value_declaration.is_none() && prop.check_flags.intersects(CheckFlags::ContainsPrivate)
}

bitflags::bitflags! {
    #[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
    pub struct MappedTypeModifiers: u32 {
        const ExcludeReadonly = 1 << 0;
        const IncludeReadonly = 1 << 1;
        const ExcludeOptional = 1 << 2;
        const IncludeOptional = 1 << 3;
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ReferenceHint {
    #[default]
    Unspecified,
    Identifier,
    Property,
    ExportAssignment,
    Jsx,
    ExportImportEquals,
    ExportSpecifier,
    Decorator,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ThisAssignmentDeclarationKind {
    #[default]
    None,
    Typed,
    Method,
    Constructor,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum JSDeclarationKind {
    #[default]
    None,
    ThisProperty,
    ModuleExports,
    ExportEquals,
}

pub struct CallState {
    pub node: Arc<Node>,
    pub args: Vec<Arc<Node>>,
    pub type_arguments: Vec<Arc<Node>>,
    pub candidates: Vec<Arc<Signature>>,
    pub is_single_non_generic_candidate: bool,
    pub arg_check_mode: CheckMode,
    pub signature_help_trailing_comma: bool,
    pub candidates_for_argument_error: Option<Vec<Arc<Signature>>>,
    pub candidate_for_argument_arity_error: Option<Arc<Signature>>,
    pub candidate_for_type_argument_error: Option<Arc<Signature>>,
}

impl Default for CallState {
    fn default() -> Self {
        Self {
            node: Arc::new(Node::new(SyntaxKind::Unknown, NodeData::Token)),
            args: Vec::new(),
            type_arguments: Vec::new(),
            candidates: Vec::new(),
            is_single_non_generic_candidate: false,
            arg_check_mode: CheckMode::Normal,
            signature_help_trailing_comma: false,
            candidates_for_argument_error: None,
            candidate_for_argument_arity_error: None,
            candidate_for_type_argument_error: None,
        }
    }
}

pub fn symbol_ptr_key(symbol: &Arc<Symbol>) -> u64 {
    Arc::as_ptr(symbol) as *const () as u64
}

pub fn is_node_descendant_of(node: &Arc<Node>, other: &Arc<Node>) -> bool {
    let mut current = Some(Arc::clone(node));
    while let Some(current_node) = current {
        if Arc::ptr_eq(&current_node, other) {
            return true;
        }
        current = current_node.parent();
    }
    false
}

pub fn is_invalid_computed_property_name(name: &Node) -> bool {
    !tsox_frontend::ast::utilities::is_property_name_literal(name)
        && !tsox_frontend::ast::utilities::is_entity_name_expression(name)
}

pub fn is_internal_module_import_equals_declaration(node: &Node) -> bool {
    ast::is_import_equals_declaration(node)
        && tsox_frontend::ast::utilities::is_entity_name(
            &node.as_import_equals_declaration().module_reference,
        )
}

pub const TYPE_FLAGS_INSTANTIABLE: TypeFlags = TypeFlags::TypeParameter
    .union(TypeFlags::IndexedAccess)
    .union(TypeFlags::Conditional)
    .union(TypeFlags::Substitution);

pub const CHECK_FLAGS_NON_UNIFORM_AND_LITERAL: tsox_frontend::ast::CheckFlags =
    tsox_frontend::ast::CheckFlags::HasNonUniformType
        .union(tsox_frontend::ast::CheckFlags::HasLiteralType);
