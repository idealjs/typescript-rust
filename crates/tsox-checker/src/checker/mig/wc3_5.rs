#![allow(unused_imports)]

use crate::checker::checker_checker::*;
use crate::checker::mig::wc3::NodeAccessExt;
use crate::checker::mig::m3a_2::{new_diagnostic_for_node, new_diagnostic_chain_for_node};
use crate::checker::utilities_has_only_expression_initialization::{is_super_call, is_call_chain};
use crate::checker::utilities_is_optional_symbol::is_type_any;
use crate::checker::mig::m1d_2::get_base_type_node_of_class;
use crate::checker::mig::wc3::CallState;
use crate::checker::types_type_id::TYPE_FLAGS_PRIMITIVE;
use crate::checker::mig::wc3::r25k2_defs::create_instantiated_symbol_table_opt;
use crate::checker::mig::wc3::r26k2_defs::{
    set_interface_resolved_base_types, with_object_type_structured_mut,
};
use crate::checker::mig::wc3_6::r24k4_defs::{non_existent_properties_contains, non_existent_properties_insert};
use tsox_frontend::ast::{INTERNAL_SYMBOL_NAME_CALL, INTERNAL_SYMBOL_NAME_NEW, INTERNAL_SYMBOL_NAME_INDEX, INTERNAL_SYMBOL_NAME_CONSTRUCTOR, INTERNAL_SYMBOL_NAME_EXPORT_EQUALS};
use std::sync::Arc;
use tsox_core::diagnostics as msg;
use tsox_frontend::ast::{self, Node, Symbol, SyntaxKind};

impl Checker {
    pub fn report_nonexistent_property(
        &mut self,
        prop_node: &Arc<Node>,
        containing_type: &Arc<Type>,
        is_unchecked_js: bool,
    ) {
        let key = (
            Arc::as_ptr(prop_node) as usize,
            Arc::as_ptr(containing_type) as usize,
            is_unchecked_js,
        );
        if non_existent_properties_contains(&key) {
            return;
        }
        non_existent_properties_insert(key);
        let links = self.node_links.get_or_default(prop_node);
        if links.flags.intersects(NodeCheckFlags::TypeChecked) {
            return;
        }
        links.flags.insert(NodeCheckFlags::TypeChecked);
        if ast::mig::w7a::is_jsdoc_name_reference_context(prop_node) {
            return;
        }
        let mut diagnostic: Option<ast::Diagnostic> = None;
        if !ast::is_private_identifier(prop_node)
            && containing_type.flags.intersects(TypeFlags::Union)
            && !containing_type.flags.intersects(TYPE_FLAGS_PRIMITIVE)
        {
            for subtype in containing_type.types().into_iter().flatten() {
                if self.get_property_of_type(&subtype, &prop_node.text()).is_none()
                    && self
                        .get_applicable_index_info_for_name(&subtype, &prop_node.text())
                        .is_none()
                {
                    diagnostic = Some(new_diagnostic_chain_for_node(
                        diagnostic.as_ref(),
                        Some(prop_node),
                        msg::PROPERTY_0_DOES_NOT_EXIST_ON_TYPE_1,
                        vec![
                            tsox_frontend::scanner::mig::m3i::declaration_name_to_string(Some(prop_node)),
                            self.type_to_string(&subtype),
                        ],
                    ));
                    break;
                }
            }
        }
        if self.type_has_static_property(&prop_node.text(), containing_type) {
            let prop_name =
                tsox_frontend::scanner::mig::m3i::declaration_name_to_string(Some(prop_node));
            let type_name = self.type_to_string(containing_type);
            diagnostic = Some(new_diagnostic_chain_for_node(
                diagnostic.as_ref(),
                Some(prop_node),
                msg::PROPERTY_0_DOES_NOT_EXIST_ON_TYPE_1_DID_YOU_MEAN_TO_ACCESS_THE_STATIC_MEMBER_2_INSTEAD,
                vec![
                    prop_name.clone(),
                    type_name.clone(),
                    format!("{}.{}", type_name, prop_name),
                ],
            ));
        } else {
            let promised_type = self.get_promised_type_of_promise(containing_type);
            if promised_type
                .as_ref()
                .is_some_and(|p| self.get_property_of_type(p, &prop_node.text()).is_some())
            {
                let diag = new_diagnostic_chain_for_node(
                    diagnostic.as_ref(),
                    Some(prop_node),
                    msg::PROPERTY_0_DOES_NOT_EXIST_ON_TYPE_1,
                    vec![
                        tsox_frontend::scanner::mig::m3i::declaration_name_to_string(Some(prop_node)),
                        self.type_to_string(containing_type),
                    ],
                );
                diagnostic = Some(diag);
                diagnostic
                    .as_mut()
                    .unwrap()
                    .add_related_info(new_diagnostic_for_node(
                        Some(prop_node),
                        msg::DID_YOU_FORGET_TO_USE_AWAIT,
                        vec![],
                    ));
            } else {
                let missing_property =
                    tsox_frontend::scanner::mig::m3i::declaration_name_to_string(Some(prop_node));
                let container = self.type_to_string(containing_type);
                let lib_suggestion =
                    self.get_suggested_lib_for_non_existent_property(&missing_property, containing_type);
                if !lib_suggestion.is_empty() {
                    diagnostic = Some(new_diagnostic_chain_for_node(
                        diagnostic.as_ref(),
                        Some(prop_node),
                        msg::PROPERTY_0_DOES_NOT_EXIST_ON_TYPE_1_DO_YOU_NEED_TO_CHANGE_YOUR_TARGET_LIBRARY_TRY_CHANGING_THE_LIB_COMPILER_OPTION_TO_2_OR_LATER,
                        vec![missing_property, container, lib_suggestion],
                    ));
                } else {
                    let suggestion =
                        self.get_suggested_symbol_for_nonexistent_property(prop_node, containing_type);
                    if let Some(suggestion) = suggestion {
                        let suggested_name = suggestion.name.clone();
                        let message = if is_unchecked_js {
                            msg::PROPERTY_0_MAY_NOT_EXIST_ON_TYPE_1_DID_YOU_MEAN_2
                        } else {
                            msg::PROPERTY_0_DOES_NOT_EXIST_ON_TYPE_1_DID_YOU_MEAN_2
                        };
                        diagnostic = Some(new_diagnostic_chain_for_node(
                            diagnostic.as_ref(),
                            Some(prop_node),
                            message,
                            vec![missing_property, container, suggested_name.clone()],
                        ));
                        if let Some(value_declaration) = suggestion.value_declaration.as_ref() {
                            diagnostic
                                .as_mut()
                                .unwrap()
                                .add_related_info(new_diagnostic_for_node(
                                    Some(value_declaration),
                                    msg::X_0_IS_DECLARED_HERE,
                                    vec![suggested_name],
                                ));
                        }
                    } else {
                        let elaborate_chain = diagnostic
                            .as_ref()
                            .map(|d| Arc::new(d.clone()))
                            .unwrap_or_else(|| {
                                Arc::new(new_diagnostic_for_node(
                                    Some(prop_node),
                                    msg::PROPERTY_0_DOES_NOT_EXIST_ON_TYPE_1,
                                    vec![missing_property.clone(), container.clone()],
                                ))
                            });
                        diagnostic = Some((*self.elaborate_never_intersection(
                            elaborate_chain,
                            prop_node,
                            containing_type,
                        )).clone());
                        let message = if self.container_seems_to_be_empty_dom_element(containing_type) {
                            msg::PROPERTY_0_DOES_NOT_EXIST_ON_TYPE_1_TRY_CHANGING_THE_LIB_COMPILER_OPTION_TO_INCLUDE_DOM
                        } else {
                            msg::PROPERTY_0_DOES_NOT_EXIST_ON_TYPE_1
                        };
                        diagnostic = Some(new_diagnostic_chain_for_node(
                            diagnostic.as_ref(),
                            Some(prop_node),
                            message,
                            vec![missing_property, container],
                        ));
                    }
                }
            }
        }
        let diagnostic = diagnostic.unwrap();
        let is_error = !is_unchecked_js
            || diagnostic.code() != msg::PROPERTY_0_MAY_NOT_EXIST_ON_TYPE_1_DID_YOU_MEAN_2.code();
        self.add_error_or_suggestion(is_error, diagnostic);
    }

    pub fn resolve_anonymous_type_members(&mut self, t: &Arc<Type>) {
        let d = t.as_object_type().expect("object type data");
        if let Some(target) = d.target.as_ref() {
            self.set_structured_type_members(t, None, Vec::new(), Vec::new(), Vec::new());
            let properties = self.get_properties_of_object_type(target);
            let members = create_instantiated_symbol_table_opt(self, &properties, d.mapper.as_ref());
            let call_signatures = self.instantiate_signatures(
                &self.get_signatures_of_type(target, SignatureKind::Call),
                d.mapper.as_ref(),
            );
            let construct_signatures = self.instantiate_signatures(
                &self.get_signatures_of_type(target, SignatureKind::Construct),
                d.mapper.as_ref(),
            );
            let target_index_infos = self.get_index_infos_of_type(target);
            let index_infos =
                self.instantiate_index_infos(&target_index_infos, d.mapper.as_ref());
            self.set_structured_type_members(
                t,
                Some(members),
                call_signatures,
                construct_signatures,
                index_infos,
            );
            return;
        }
        let symbol = self.get_merged_symbol(t.symbol().as_ref().expect("type symbol"));
        if symbol.flags.intersects(SymbolFlags::TypeLiteral) {
            self.set_structured_type_members(t, None, Vec::new(), Vec::new(), Vec::new());
            let members = self.get_members_of_symbol(&symbol);
            let call_signatures =
                self.get_signatures_of_symbol(members.get(INTERNAL_SYMBOL_NAME_CALL));
            let construct_signatures =
                self.get_signatures_of_symbol(members.get(INTERNAL_SYMBOL_NAME_NEW));
            let index_infos = self.get_index_infos_of_symbol(&symbol);
            self.set_structured_type_members(
                t,
                Some(members),
                call_signatures,
                construct_signatures,
                index_infos,
            );
            return;
        }
        let mut members = self.get_exports_of_symbol(&symbol);
        let mut index_infos: Vec<Arc<IndexInfo>> = Vec::new();
        if self.global_this_symbol.as_ref().is_some_and(|g| Arc::ptr_eq(&symbol, g)) {
            let mut vars_only = SymbolTable::default();
            for p in members.entries.values() {
                let all_ambient = !p.declarations.is_empty()
                    && p.declarations.iter().all(|d| ast::is_ambient_module(d));
                if !p.flags.intersects(SymbolFlags::BLOCK_SCOPED)
                    && !(p.flags.intersects(SymbolFlags::ValueModule) && all_ambient)
                {
                    vars_only.insert(p.name.clone(), Arc::clone(p));
                }
            }
            members = vars_only;
        }
        let mut base_constructor_index_info: Option<Arc<IndexInfo>> = None;
        self.set_structured_type_members(t, Some(members.clone()), Vec::new(), Vec::new(), Vec::new());
        if symbol.flags.intersects(SymbolFlags::Class) {
            let class_type = self.get_declared_type_of_class_or_interface(&symbol);
            if let Some(base_constructor_type) = self.get_base_constructor_type_of_class(&class_type) {
                if base_constructor_type
                    .flags
                    .intersects(TypeFlags::Object | TypeFlags::Intersection | crate::checker::types_type_id::TYPE_FLAGS_TYPE_VARIABLE)
                {
                    let props = self.get_properties_of_type(&base_constructor_type);
                    members = self.add_inherited_members(members, &props);
                    self.set_structured_type_members(t, Some(members.clone()), Vec::new(), Vec::new(), Vec::new());
                } else if Arc::ptr_eq(&base_constructor_type, &self.any_type()) {
                    base_constructor_index_info = Some(self.any_base_type_index_info());
                }
            }
        }
        let index_symbol = members.get(INTERNAL_SYMBOL_NAME_INDEX).cloned();
        if let Some(index_symbol) = index_symbol {
            let values: Vec<Arc<Symbol>> = members.entries.values().cloned().collect();
            index_infos = self.get_index_infos_of_index_symbol(&index_symbol, &values);
        } else {
            if let Some(info) = base_constructor_index_info {
                index_infos.push(info);
            }
            let declared_type = self.get_declared_type_of_symbol(&symbol);
            let props_number_like = d
                .structured
                .properties
                .iter()
                .any(|prop| {
                    self.get_type_of_symbol(prop)
                        .flags
                        .intersects(crate::checker::types_type_id::TYPE_FLAGS_NUMBER_LIKE)
                });
            if symbol.flags.intersects(SymbolFlags::ENUM)
                && (declared_type.flags.contains(TypeFlags::Enum) || props_number_like)
            {
                index_infos.push(self.enum_number_index_info());
            }
        }
        with_object_type_structured_mut(t, |s| s.index_infos = index_infos);
        if symbol
            .flags
            .intersects(SymbolFlags::Function | SymbolFlags::Method)
        {
            let signatures = self.get_signatures_of_symbol(Some(&symbol));
            with_object_type_structured_mut(t, |s| {
                s.call_signature_count = signatures.len();
                s.signatures = signatures;
            });
        }
        if symbol.flags.intersects(SymbolFlags::Class) {
            let class_type = self.get_declared_type_of_class_or_interface(&symbol);
            let mut construct_signatures = self.get_signatures_of_symbol(
                symbol
                    .members
                    .get(INTERNAL_SYMBOL_NAME_CONSTRUCTOR),
            );
            if construct_signatures.is_empty() {
                construct_signatures = self.get_default_construct_signatures(&class_type);
            }
            with_object_type_structured_mut(t, |s| s.signatures.extend(construct_signatures));
        }
    }

    pub fn resolve_base_types_of_class(&mut self, t: &Arc<Type>) {
        let resolved_base = self
            .get_base_constructor_type_of_class(t)
            .unwrap_or_else(|| self.undefined_type());
        let base_constructor_type = self.get_apparent_type(&resolved_base);
        if !base_constructor_type
            .flags
            .intersects(TypeFlags::Object | TypeFlags::Intersection | TypeFlags::Any)
        {
            return;
        }
        let base_type_node = get_base_type_node_of_class(t);
        let base_type: Arc<Type>;
        let mut original_base_type: Option<Arc<Type>> = None;
        if let Some(symbol) = base_constructor_type.symbol().as_ref() {
            original_base_type = Some(self.get_declared_type_of_symbol(symbol));
        }
        if base_constructor_type
            .symbol()
            .as_ref()
            .is_some_and(|s| s.flags.intersects(SymbolFlags::Class))
            && self.are_all_outer_type_parameters_applied(original_base_type.as_ref().unwrap())
        {
            base_type = self.get_type_from_class_or_interface_reference(
                base_type_node.as_ref().unwrap(),
                base_constructor_type.symbol().as_ref().unwrap(),
            );
        } else if base_constructor_type.flags.contains(TypeFlags::Any) {
            base_type = Arc::clone(&base_constructor_type);
        } else {
            let base_type_node = base_type_node.as_ref().unwrap();
            let type_arg_nodes: Vec<Arc<Node>> = base_type_node
                .type_arguments()
                .map(|tl| tl.nodes.clone())
                .unwrap_or_default();
            let constructors = self.get_instantiated_constructors_for_type_arguments(
                &base_constructor_type,
                &type_arg_nodes,
                Some(base_type_node),
            );
            if constructors.is_empty() {
                self.error_message(
                    &base_type_node.expression().unwrap(),msg::NO_BASE_CONSTRUCTOR_HAS_THE_SPECIFIED_NUMBER_OF_TYPE_ARGUMENTS,
                    &[],
                );
                return;
            }
            base_type = self.get_return_type_of_signature(&constructors[0])
                    .unwrap_or_else(|| Arc::clone(&self.error_type()));
        }
        if self.is_error_type(&base_type) {
            return;
        }
        let reduced_base_type = self.get_reduced_type(&base_type);
        if !self.is_valid_base_type(&reduced_base_type) {
            let error_node = base_type_node.as_ref().unwrap().expression().unwrap();
            let base_diag = new_diagnostic_for_node(
                Some(&error_node),
                msg::BASE_CONSTRUCTOR_RETURN_TYPE_0_IS_NOT_AN_OBJECT_TYPE_OR_INTERSECTION_OF_OBJECT_TYPES_WITH_STATICALLY_KNOWN_MEMBERS,
                vec![self.type_to_string(&reduced_base_type)],
            );
            let elaborated = (*self.elaborate_never_intersection(
                Arc::new(base_diag.clone()),
                &error_node,
                &base_type,
            ))
            .clone();
            let diagnostic = new_diagnostic_chain_for_node(
                Some(&elaborated),
                Some(&error_node),
                msg::BASE_CONSTRUCTOR_RETURN_TYPE_0_IS_NOT_AN_OBJECT_TYPE_OR_INTERSECTION_OF_OBJECT_TYPES_WITH_STATICALLY_KNOWN_MEMBERS,
                vec![self.type_to_string(&reduced_base_type)],
            );
            self.add_diagnostic(diagnostic);
            return;
        }
        if Arc::ptr_eq(t, &reduced_base_type) || self.has_base_type(&reduced_base_type, t) {
            let t_str = self.type_to_string(t);
            self.error_message(
                t.symbol().unwrap().value_declaration.as_ref().unwrap(),msg::TYPE_0_RECURSIVELY_REFERENCES_ITSELF_AS_A_BASE_TYPE,
                &[t_str],
            );
            return;
        }
        set_interface_resolved_base_types(t, vec![reduced_base_type]);
    }

    pub fn resolve_call(
        &mut self,
        node: &Arc<Node>,
        signatures: &[Arc<Signature>],
        candidates_out_array: Option<&mut Vec<Arc<Signature>>>,
        check_mode: CheckMode,
        call_chain_flags: SignatureFlags,
        head_message: Option<&'static msg::Message>,
    ) -> Option<Arc<Signature>> {
        let is_tagged_template = node.kind == SyntaxKind::TaggedTemplateExpression;
        let is_decorator = node.kind == SyntaxKind::Decorator;
        let is_jsx_opening_or_self_closing_element = ast::mig::m3g::is_jsx_opening_like_element(node);
        let is_instanceof = node.kind == SyntaxKind::BinaryExpression;
        let report_errors =
            !crate::checker::mig::m2h::r21k10_defs::inference_partially_blocked_get()
                && candidates_out_array.is_none();
        let mut s = CallState::default();
        s.node = Arc::clone(node);
        if !is_decorator && !is_instanceof && !is_super_call(node) && !ast::is_jsx_opening_fragment(node) {
            s.type_arguments = node
                .type_arguments()
                .map(|tl| tl.nodes.clone())
                .unwrap_or_default();
            if is_tagged_template
                || is_jsx_opening_or_self_closing_element
                || node.expression().unwrap().kind != SyntaxKind::SuperKeyword
            {
                self.check_source_elements(&s.type_arguments);
            }
        }
        s.candidates = self.reorder_candidates(signatures);
        if let Some(out) = candidates_out_array {
            *out = s.candidates.clone();
        }
        if s.candidates.is_empty() {
            return self.get_unknown_signature();
        }
        s.args = self.get_effective_call_arguments(node);
        s.is_single_non_generic_candidate =
            s.candidates.len() == 1 && s.candidates[0].type_parameters.is_empty();
        if !is_decorator
            && !s.is_single_non_generic_candidate
            && s.args.iter().any(|arg| self.is_context_sensitive(arg))
        {
            s.arg_check_mode = CheckMode::SkipContextSensitive;
        } else {
            s.arg_check_mode = CheckMode::Normal;
        }
        s.signature_help_trailing_comma = check_mode.intersects(CheckMode::IsForSignatureHelp)
            && ast::is_call_expression(node)
            && node
                .argument_list()
                .as_ref()
                .is_some_and(|al| al.has_trailing_comma());
        let mut result: Option<Arc<Signature>> = None;
        if s.candidates.len() > 1 {
            result = self.choose_overload(&mut s, &self.subtype_relation());
        }
        if result.is_none() {
            result = self.choose_overload(&mut s, &self.assignable_relation());
        }
        if let Some(result) = result {
            return Some(result);
        }
        let result = node
            .arguments()
            .and_then(|args_list| {
                self.candidate_for_overload_failure(&s.node, &s.candidates, args_list)
            });
        self.signature_links
            .get_or_default(node)
            .resolved_signature = result.clone();
        if report_errors {
            let mut head_message = head_message;
            if head_message.is_none() && is_instanceof {
                head_message = Some(
                    &msg::THE_LEFT_HAND_SIDE_OF_AN_INSTANCEOF_EXPRESSION_MUST_BE_ASSIGNABLE_TO_THE_FIRST_ARGUMENT_OF_THE_RIGHT_HAND_SIDE_S_SYMBOL_HASINSTANCE_METHOD,
                );
            }
            self.report_call_resolution_errors(node, &mut s, signatures, head_message);
        }
        result
    }

    pub fn resolve_call_expression(
        &mut self,
        node: &Arc<Node>,
        candidates_out_array: Option<&mut Vec<Arc<Signature>>>,
        check_mode: CheckMode,
    ) -> Option<Arc<Signature>> {
        if node.expression().unwrap().kind == SyntaxKind::SuperKeyword {
            let expr = node.expression().unwrap();
            self.check_super_expression(&expr);
            let super_type = self.get_type_of_node(&expr);
            if is_type_any(&super_type) {
                if let Some(args_list) = node.arguments() {
                    for arg in &args_list.nodes {
                        self.check_expression(arg);
                    }
                }
                return Some(self.any_signature());
            }
            if !self.is_error_type(&super_type) {
                let base_type_node = ast::get_containing_class(node)
                    .as_ref()
                    .and_then(|cc| ast::get_class_extends_heritage_element(cc));
                if let Some(base_type_node) = base_type_node {
                    let type_arg_nodes: Vec<Arc<Node>> = base_type_node
                        .type_arguments()
                        .map(|tl| tl.nodes.clone())
                        .unwrap_or_default();
                    let base_constructors = self.get_instantiated_constructors_for_type_arguments(
                        &super_type,
                        &type_arg_nodes,
                        Some(&base_type_node),
                    );
                    return self.resolve_call(
                        node,
                        &base_constructors,
                        candidates_out_array,
                        check_mode,
                        SignatureFlags::empty(),
                        None,
                    );
                }
            }
            return Some(self.resolve_untyped_call(node));
        }
        if ast::is_import_call(node) {
            return Some(self.resolve_untyped_call(node));
        }
        let func_type_initial = self.check_expression_ex(&node.expression().unwrap(), CheckMode::Normal);
        let mut call_chain_flags = SignatureFlags::empty();
        let mut func_type = func_type_initial;
        if is_call_chain(node) {
            let non_optional_type =
                self.get_optional_expression_type(&func_type, &node.expression().unwrap());
            if Arc::ptr_eq(&non_optional_type, &func_type) {
                call_chain_flags = SignatureFlags::empty();
            } else if ast::mig::m3g_2::is_outermost_optional_chain(node) {
                call_chain_flags = SignatureFlags::IsOuterCallChain;
            } else {
                call_chain_flags = SignatureFlags::IsInnerCallChain;
            }
            func_type = non_optional_type;
        }
        let func_type = self.check_non_null_type_with_reporter(
            &func_type,
            &node.expression().unwrap(),
            Checker::report_cannot_invoke_possibly_null_or_undefined_error,
        );
        if Arc::ptr_eq(&func_type, &self.silent_never_type()) {
            return Some(self.silent_never_signature());
        }
        let apparent_type = self.get_apparent_type(&func_type);
        if self.is_error_type(&apparent_type) {
            return Some(self.resolve_error_call(node));
        }
        let call_signatures = self.get_signatures_of_type(&apparent_type, SignatureKind::Call);
        let num_construct_signatures =
            self.get_signatures_of_type(&apparent_type, SignatureKind::Construct).len();
        if self.is_untyped_function_call(&func_type, &apparent_type, call_signatures.len(), num_construct_signatures)
        {
            if !self.is_error_type(&func_type) && node.type_arguments().is_some() {
                self.error_message(
                    node,msg::UNTYPED_FUNCTION_CALLS_MAY_NOT_ACCEPT_TYPE_ARGUMENTS,
                    &[],
                );
            }
            return Some(self.resolve_untyped_call(node));
        }
        if call_signatures.is_empty() {
            if num_construct_signatures != 0 {
                let func_type_str = self.type_to_string(&func_type);
                self.error_message(
                    node,msg::VALUE_OF_TYPE_0_IS_NOT_CALLABLE_DID_YOU_MEAN_TO_INCLUDE_NEW,
                    &[func_type_str],
                );
            } else {
                let mut related_information: Option<ast::Diagnostic> = None;
                if let Some(args_list) = node.arguments() {
                    let has_single_arg = args_list.nodes.len() == 1;
                    if has_single_arg {
                    let file = self.get_source_file_of_node(node).unwrap();
                    let text = file.text.clone();
                    let index = tsox_frontend::scanner::skip_trivia_ex(
                        &text,
                        node.expression().unwrap().end(),
                        &tsox_frontend::scanner::SkipTriviaOptions::default(),
                        None,
                    );
                    if index > 0
                        && tsox_core::stringutil::is_line_break(
                            text.as_bytes()[index - 1] as char,
                        )
                    {
                        related_information = Some(new_diagnostic_for_node(
                            Some(&node.expression().unwrap()),
                            msg::ARE_YOU_MISSING_A_SEMICOLON,
                            vec![],
                        ));
                    }
                    }
                }
                self.invocation_error(&node.expression().unwrap(), &apparent_type, SignatureKind::Call, related_information);
            }
            return Some(self.resolve_error_call(node));
        }
        if check_mode.intersects(CheckMode::SkipGenericFunctions)
            && node.type_arguments().is_none()
            && call_signatures
                .iter()
                .any(|sig| self.is_generic_function_returning_function(sig))
        {
            self.skipped_generic_function(node, check_mode);
            return Some(self.resolving_signature());
        }
        self.resolve_call(
            node,
            &call_signatures,
            candidates_out_array,
            check_mode,
            call_chain_flags,
            None,
        )
    }

    pub fn resolve_decorator(
        &mut self,
        node: &Arc<Node>,
        candidates_out_array: Option<&mut Vec<Arc<Signature>>>,
        check_mode: CheckMode,
    ) -> Option<Arc<Signature>> {
        if !ast::can_have_decorators(node.parent().as_ref().unwrap()) {
            return Some(self.resolve_error_call(node));
        }
        let expr = node.expression().unwrap();
        let func_type = self.check_expression_ex(&expr, CheckMode::Normal);
        let apparent_type = self.get_apparent_type(&func_type);
        if self.is_error_type(&apparent_type) {
            return Some(self.resolve_error_call(node));
        }
        let call_signatures = self.get_signatures_of_type(&apparent_type, SignatureKind::Call);
        let num_construct_signatures =
            self.get_signatures_of_type(&apparent_type, SignatureKind::Construct).len();
        if self.is_untyped_function_call(&func_type, &apparent_type, call_signatures.len(), num_construct_signatures)
        {
            return Some(self.resolve_untyped_call(node));
        }
        if self.is_potentially_uncalled_decorator(node, &call_signatures)
            && !ast::is_parenthesized_expression(&expr)
        {
            let node_str = tsox_frontend::scanner::mig::m3i::get_text_of_node(&expr);
            self.error_message(
                node,msg::X_0_ACCEPTS_TOO_FEW_ARGUMENTS_TO_BE_USED_AS_A_DECORATOR_HERE_DID_YOU_MEAN_TO_CALL_IT_FIRST_AND_WRITE_0,
                &[node_str],
            );
            return Some(self.resolve_error_call(node));
        }
        let head_message = self.get_diagnostic_head_message_for_decorator_resolution(node);
        if call_signatures.is_empty() {
            let details = self.invocation_error_details(&expr, &apparent_type, SignatureKind::Call);
            let diag = new_diagnostic_chain_for_node(
                Some(&details.unwrap()),
                None,
                *head_message,
                vec![],
            );
            let diag = self.add_diagnostic(diag);
            self.invocation_error_recovery(&apparent_type, SignatureKind::Call, &diag);
            return Some(self.resolve_error_call(node));
        }
        let decorator_signature = self.get_decorator_call_signature(node);
        let Some(decorator_signature) = decorator_signature else {
            return Some(self.resolve_error_call(node));
        };
        let _ = decorator_signature;
        self.resolve_call(
            node,
            &call_signatures,
            candidates_out_array,
            check_mode,
            SignatureFlags::empty(),
            Some(head_message),
        )
    }

    pub fn resolve_entity_name(
        &mut self,
        name: &Arc<Node>,
        meaning: SymbolFlags,
        ignore_errors: bool,
        dont_resolve_alias: bool,
        location: Option<&Arc<Node>>,
    ) -> Option<Arc<Symbol>> {
        if ast::node_is_missing(Some(name)) {
            return None;
        }
        let symbol: Option<Arc<Symbol>>;
        match name.kind {
            SyntaxKind::Identifier => {
                let mut message: Option<&'static msg::Message> = None;
                if !ignore_errors {
                    if meaning == SymbolFlags::NAMESPACE || ast::node_is_synthesized(name) {
                        message = Some(&msg::CANNOT_FIND_NAMESPACE_0);
                    } else {
                        message = Some(self.get_cannot_find_name_diagnostic_for_name(
                            &ast::mig::m3e_4::get_first_identifier(name),
                        ));
                    }
                }
                let resolve_location = location.unwrap_or(name);
                if meaning == SymbolFlags::NAMESPACE {
                    let mut resolved =
                        self.resolve_name(&name.text(), resolve_location, meaning, false);
                    let mut resolved_symbol = resolved.take().map(|s| self.get_merged_symbol(&s));
                    if resolved_symbol.is_none() {
                        let alias = self.resolve_name(
                            &name.text(),
                            resolve_location,
                            SymbolFlags::Alias,
                            false,
                        );
                        let alias = alias.map(|s| self.get_merged_symbol(&s));
                        if let Some(alias) = alias {
                            if alias.name == INTERNAL_SYMBOL_NAME_EXPORT_EQUALS {
                                resolved_symbol = alias.parent();
                            }
                        }
                    }
                    if resolved_symbol.is_none() && message.is_some() {
                        self.resolve_name(&name.text(), resolve_location, meaning, false);
                    }
                    symbol = resolved_symbol;
                } else {
                    symbol = self
                        .resolve_name(&name.text(), resolve_location, meaning, false)
                        .map(|s| self.get_merged_symbol(&s));
                }
            }
            SyntaxKind::QualifiedName => {
                let qualified = name.as_qualified_name();
                symbol = self.resolve_qualified_name(
                    name,
                    &qualified.left,
                    &qualified.right,
                    meaning,
                    ignore_errors,
                    location,
                );
            }
            SyntaxKind::PropertyAccessExpression => {
                let access = name.as_property_access_expression();
                symbol = self.resolve_qualified_name(
                    name,
                    &access.expression,
                    &access.name,
                    meaning,
                    ignore_errors,
                    location,
                );
            }
            _ => panic!("Unknown entity name kind"),
        }
        if let Some(sym) = &symbol {
            if !self
                .unknown_symbol
                .as_ref()
                .is_some_and(|u| Arc::ptr_eq(u, sym))
            {
                if !ast::node_is_synthesized(name)
                    && ast::is_entity_name(name)
                    && (sym.flags.intersects(SymbolFlags::Alias)
                        || name
                            .parent()
                            .as_ref()
                            .is_some_and(|p| p.kind == SyntaxKind::ExportAssignment))
                {
                    if let Some(alias_declaration) =
                        crate::checker::utilities_is_private_within_ambient::get_alias_declaration_from_name(name).as_ref()
                    {
                        self.mark_symbol_of_alias_declaration_if_type_only(Some(alias_declaration), None);
                    }
                }
                let mut current = Arc::clone(sym);
                while !current.flags.intersects(meaning)
                    && !dont_resolve_alias
                    && current.flags.intersects(SymbolFlags::Alias)
                {
                    let next = self.resolve_alias_base(Arc::clone(&current));
                    if Arc::ptr_eq(&next, &current) {
                        break;
                    }
                    current = next;
                }
                return Some(current);
            }
        }
        symbol
    }
}
