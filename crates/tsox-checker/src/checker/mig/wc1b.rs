#![allow(unused_imports)]
#[path = "r29k2_defs.rs"]
pub mod r29k2_defs;

use self::r29k2_defs::NodeFactoryExt29;
use tsox_core::diagnostics::{Message, messages_generated::*};

use crate::checker::checker_literals_array::{is_spread_into_call_or_new};
use crate::checker::utilities_is_optional_symbol::{
    is_type_any as is_type_any_ref, is_literal_expression_of_object, is_array_or_tuple_type as is_array_or_tuple_type_ref,
    is_late_bound_name,
};
use crate::checker::utilities_token_is_identifier_or_keyword::{
    is_tuple_type as is_tuple_type_ref, is_object_literal_type as is_object_literal_type_ref,
    is_literal_type as is_literal_type_ref,
};
use crate::checker::exports_union_reduction::{
    get_declaration_modifier_flags_from_symbol, get_declaration_modifier_flags_from_symbol_ex,
};
use crate::checker::grammarchecks_is_this_parameter_2::is_binding_pattern;
use crate::checker::mapper::prepend_type_mapping;
use crate::checker::utilities_get_assignment_target::{walk_up_outer_expressions, is_delete_target};
use crate::checker::inference_inference_key_2::{InferenceFlags, InferencePriority};
use crate::checker::relater_recursion_identity::RecursionIdentity as RecursionId;
use super::m2c_3::some_type;
use super::m2c_3::super_call_is_root_level_in_constructor;
use super::wc3_3::is_instance_property_with_initializer_or_private_identifier_property;
use crate::checker::utilities_has_only_expression_initialization::is_super_call;
use crate::checker::mig::m1e::r18k5_helpers::class_element_or_class_element_parameter_is_decorated;
use super::wc3_2::is_const_enum_object_type;
use super::m1c_3::create_diagnostic_for_node;
use super::m3a_2::new_diagnostic_for_node;
use super::m2a::{is_rest_parameter, is_prototype_property};
use super::wc2_2::get_mapped_type_optionality;
use super::wc3_3::is_generic_tuple_type;
use super::w9a::new_type_mapper;
use super::m1d_2::get_conditional_type_key;
use super::wc3::JSDeclarationKind;
use super::m2b_2::TupleNormalizer;
use tsox_frontend::ast::utilities::ModuleInstanceState;
use tsox_frontend::ast::utilities::is_in_js_file;
use tsox_frontend::ast::utilities::is_class_element;
use tsox_frontend::ast::utilities::is_statement;
use tsox_frontend::ast::utilities_r16a::is_for_in_statement;
use tsox_frontend::ast::utilities::find_ancestor;
use tsox_frontend::ast::utilities::is_string_literal_like;
use tsox_frontend::ast::utilities::get_containing_class;
use tsox_frontend::ast::node_data_generated::{
    is_binary_expression, is_call_expression, is_identifier, is_private_identifier, is_decorator,
    is_parameter_declaration, is_conditional_type_node, is_mapped_type_node, is_tuple_type_node,
    is_optional_type_node, is_rest_type_node, is_named_tuple_member, is_property_declaration,
};
use tsox_frontend::ast::mig::m3f_3::is_assignment_target;
use tsox_frontend::ast::mig::m3g::{is_logical_or_coalescing_binary_expression, is_named_evaluation_source};
use tsox_frontend::ast::mig::m3f_4::is_dotted_name;
use tsox_frontend::ast::mig::m3e_4::{get_enclosing_block_scope_container, get_assignment_declaration_kind};
use tsox_frontend::scanner::mig::m3i::{declaration_name_to_string, get_text_of_node};
use tsox_frontend::ast::mig::m3e::get_function_flags;
use tsox_frontend::ast::mig::m3g_3::{skip_outer_expressions, node_is_decorated, skip_type_parentheses, OuterExpressionKinds};
use tsox_frontend::ast::mig::m3g_2::is_private_identifier_class_element_declaration;
use tsox_frontend::ast::mig::m3f_2::{node_parameters, has_abstract_modifier};
use tsox_frontend::ast::mig::m3f::get_right_most_assigned_expression;
use tsox_frontend::ast::mig::m3b::initializer;
use tsox_frontend::ast::mig::m3d_2::new_diagnostic_chain;
use tsox_frontend::ast::mig::x4ast::get_first_constructor_with_body;
use tsox_frontend::scanner::token_to_string;

use crate::checker::checker_checker::*;
use crate::checker::mig::m2a::r18k8_flags::{TYPE_FLAGS_UNION, TYPE_FLAGS_ES_SYMBOL};
use crate::checker::services_checker_7::IndexKind;
use crate::checker::types_impl_chunk::LiteralValue;
#[path = "r22k4_defs.rs"]
pub mod r22k4_defs;
pub use r22k4_defs::*;
#[path = "r24k5_defs.rs"]
pub mod r24k5_defs;
pub use r24k5_defs::*;
#[path = "r25k3_defs.rs"]
pub mod r25k3_defs;
pub use r25k3_defs::*;
use crate::checker::mig::wc3::NodeAccessExt;
use crate::checker::mig::m2a::r19k11_defs::R19K11NodeExt;
use crate::checker::mig::m2e::r19k3_defs::NodeAccessExtR19k3;
use crate::checker::mig::m1e::r20k2_defs::R20k2NodeExt;
use std::sync::Arc;
use tsox_frontend::ast::{self, Node, Symbol, SyntaxKind};

impl Checker {
    pub fn check_array_literal(&mut self, node: &Arc<Node>, check_mode: CheckMode) -> Arc<Type> {
        let elements = node.elements().map(|l| l.nodes.as_slice()).unwrap_or(&[]);
        let mut element_types: Vec<Option<Arc<Type>>> = vec![None; elements.len()];
        let mut element_infos: Vec<TupleElementInfo> = (0..elements.len())
            .map(|_| TupleElementInfo {
                label: None,
                flags: ElementFlags::None,
                labeled_declaration: None,
                type_: None,
            })
            .collect();
        self.push_cached_contextual_type(node);
        let in_destructuring_pattern = is_assignment_target(node);
        let in_const_context = self.is_const_context(node);
        let contextual_type = self.get_apparent_type_of_contextual_type(node, ContextFlags::None);
        let checker_ptr: *mut Checker = self;
        let in_tuple_context = is_spread_into_call_or_new(node)
            || contextual_type.as_ref().is_some_and(|ct| {
                some_type(ct, &|t: &Arc<Type>| unsafe {
                    (*checker_ptr).is_tuple_like_type(t)
                        || ((*checker_ptr).is_generic_mapped_type(t)
                            && !t.as_mapped_type().is_some_and(|mt| mt.name_type.is_some())
                            && (*checker_ptr)
                                .get_homomorphic_type_variable(&core_OrElse(
                                    t.as_mapped_type().and_then(|mt| mt.object.target.clone()),
                                    || Arc::clone(t),
                                ))
                                .is_some())
                })
            });
        let mut has_omitted_expression = false;
        for (i, e) in elements.iter().enumerate() {
            if ast::is_spread_element(e) {
                let spread_type = self.check_expression_ex(e.expression().unwrap(), check_mode);
                if self.is_array_like_type(&spread_type) {
                    element_types[i] = Some(spread_type);
                    element_infos[i].flags = ElementFlags::Variadic;
                } else if in_destructuring_pattern {
                    let mut rest_element_type =
                        self.get_index_type_of_type(&spread_type, IndexKind::Number);
                    if rest_element_type.is_none() {
                        rest_element_type = self.get_iterated_type_or_element_type(
                            IterationUse::Destructuring,
                            &spread_type,
                            None,
                        );
                        if rest_element_type.is_none() {
                            rest_element_type = Some(self.unknown_type());
                        }
                    }
                    element_types[i] = rest_element_type;
                    element_infos[i].flags = ElementFlags::Rest;
                } else {
                    element_types[i] = Some(self.check_iterated_type_or_element_type(
                        IterationUse::Spread,
                        &spread_type,
                        Some(e.expression().unwrap()),
                    ));
                    element_infos[i].flags = ElementFlags::Rest;
                }
            } else if self.exact_optional_property_types && ast::is_omitted_expression(e) {
                has_omitted_expression = true;
                element_types[i] = Some(self.undefined_or_missing_type());
                element_infos[i].flags = ElementFlags::Optional;
            } else {
                let t = self.check_expression_for_mutable_location(e, check_mode);
                element_types[i] =
                    Some(self.add_optionality_ex(&t, true, has_omitted_expression));
                element_infos[i].flags = if has_omitted_expression {
                    ElementFlags::Optional
                } else {
                    ElementFlags::Required
                };
                if in_tuple_context
                    && check_mode.contains(CheckMode::Inferential)
                    && !check_mode.contains(CheckMode::SkipContextSensitive)
                    && self.is_context_sensitive(e)
                {
                    let inference_context = self.get_inference_context(node);
                    // In CheckMode.Inferential we should always have an inference context
                    if let Some(inference_context) = inference_context {
                        crate::checker::mig::m2e_2::r24k19_defs::add_intra_expression_inference_site_to(
                            inference_context,
                            Arc::clone(e),
                            Arc::clone(&t),
                        );
                    }
                }
            }
        }
        self.pop_contextual_type();
        if in_destructuring_pattern {
            let tys: Vec<Arc<Type>> = element_types.into_iter().flatten().collect();
            return self.create_tuple_type_ex(tys, element_infos, false);
        }
        if check_mode.contains(CheckMode::ForceTuple)
            || in_const_context
            || in_tuple_context
        {
            let readonly = in_const_context
                && !contextual_type.as_ref().is_some_and(|ct| {
                    some_type(ct, &|t: &Arc<Type>| unsafe {
                        (*checker_ptr).is_mutable_array_like_type(t)
                    })
                });
            let tys: Vec<Arc<Type>> = element_types.clone().into_iter().flatten().collect();
            let tuple_type = self.create_tuple_type_ex(tys, element_infos, readonly);
            return self.create_array_literal_type(tuple_type);
        }
        let mut tys: Vec<Arc<Type>> = element_types.into_iter().flatten().collect();
        let element_type = if !tys.is_empty() {
            for i in 0..tys.len() {
                if element_infos[i].flags.contains(ElementFlags::Variadic) {
                    let e = Arc::clone(&tys[i]);
                    tys[i] = self
                        .get_indexed_access_type_or_undefined(
                            &e,
                            &self.number_type(),
                            AccessFlags::None,
                            None,
                            None,
                        )
                        .unwrap_or_else(|| self.any_type());
                }
            }
            self.get_union_type_ex(tys, UnionReduction::Subtype)
        } else if self.strict_null_checks {
            self.implicit_never_type()
        } else {
            self.undefined_widening_type()
        };
        let array_type = self.create_array_type_ex(element_type, in_const_context);
        self.create_array_literal_type(array_type)
    }

    pub fn check_array_literal_destructuring_element_assignment(
        &mut self,
        node: &Arc<Node>,
        source_type: &Arc<Type>,
        element_index: usize,
        element_type: &Arc<Type>,
        check_mode: CheckMode,
    ) -> Option<Arc<Type>> {
        let elements_list = node.elements().cloned();
        let elements = elements_list
            .as_ref()
            .map(|l| l.nodes.clone())
            .unwrap_or_default();
        let element = &elements[element_index];
        if !ast::is_omitted_expression(element) {
            if !ast::is_spread_element(element) {
                let index_type =
                    self.get_number_literal_type(tsox_core::jsnum::Number::from(element_index as i32));
                if self.is_array_like_type(source_type) {
                    let access_flags = if self.has_default_value(element) {
                        AccessFlags::ExpressionPosition | AccessFlags::AllowMissing
                    } else {
                        AccessFlags::ExpressionPosition
                    };
                    let location =
                        self.create_synthetic_expression(element, &index_type, false, None);
                    let element_type = self
                        .get_indexed_access_type_or_undefined(
                            source_type,
                            &index_type,
                            access_flags,
                            Some(&location),
                            None,
                        )
                        .unwrap_or_else(|| self.error_type());
                    let mut assigned_type = Arc::clone(&element_type);
                    if self.has_default_value(element) {
                        assigned_type =
                            self.get_type_with_facts(&element_type, TypeFacts::NE_UNDEFINED);
                    }
                    let t = self.get_flow_type_of_destructuring(element, &assigned_type);
                    return Some(self.check_destructuring_assignment_for_binary(element, &t, check_mode, false));
                }
                self.check_destructuring_assignment_for_binary(element, element_type, check_mode, false);
                return None;
            }
            if element_index < elements.len() - 1 {
                self.error_message(element,A_REST_ELEMENT_MUST_BE_LAST_IN_A_DESTRUCTURING_PATTERN, &[]);
            } else {
                let Some(rest_expression) = element.expression() else { return None };
                if ast::is_binary_expression(rest_expression)
                    && rest_expression.as_binary_expression().operator_token.kind == SyntaxKind::EqualsToken
                {
                    self.error_message(
                        &rest_expression.as_binary_expression().operator_token,A_REST_ELEMENT_CANNOT_HAVE_AN_INITIALIZER,
                        &[],
                    );
                } else {
                    self.check_grammar_for_disallowed_trailing_comma(
                        elements_list.as_deref().unwrap(),
                        &A_REST_PARAMETER_OR_BINDING_PATTERN_MAY_NOT_HAVE_A_TRAILING_COMMA,
                    );
                    let t = if every_type(source_type, &|t: &Arc<Type>| is_tuple_type(t)) {
                        let source = Arc::clone(source_type);
                        let checker_ptr: *mut Checker = self;
                        self.map_type(&source, &mut |t: &Arc<Type>| {
                            Some(unsafe { (*checker_ptr).slice_tuple_type(t, element_index, 0) }?)
                        })
                        .unwrap_or_else(|| self.error_type())
                    } else {
                        self.create_array_type(Arc::clone(element_type))
                    };
                    return Some(self.check_destructuring_assignment_for_binary(rest_expression, &t, check_mode, false));
                }
            }
        }
        None
    }

    pub fn check_awaited_type(
        &mut self,
        t: &Arc<Type>,
        with_alias: bool,
        error_node: Option<&Arc<Node>>,
        diagnostic_message: Option<&'static Message>,
    ) -> Arc<Type> {
        let awaited_type = if with_alias {
            self.get_awaited_type_ex(t, error_node, diagnostic_message, &[])
        } else {
            self.get_awaited_type_no_alias_ex(t, error_node, diagnostic_message, &[])
        };
        if let Some(awaited_type) = awaited_type {
            return awaited_type;
        }
        self.error_type()
    }

    pub fn check_binary_like_expression(
        &mut self,
        left: &Arc<Node>,
        operator_token: &Arc<Node>,
        right: &Arc<Node>,
        check_mode: CheckMode,
        error_node: Option<&Arc<Node>>,
    ) -> Arc<Type> {
        let operator = operator_token.kind;
        if operator == SyntaxKind::EqualsToken
            && (left.kind == SyntaxKind::ObjectLiteralExpression
                || left.kind == SyntaxKind::ArrayLiteralExpression)
        {
            let right_type = self.check_expression_ex(right, check_mode);
            return self.check_destructuring_assignment_for_binary(
                left,
                &right_type,
                check_mode,
                right.kind == SyntaxKind::ThisKeyword,
            );
        }
        let mut left_type = self.check_expression_ex(left, check_mode);
        let mut right_type = self.check_expression_ex(right, check_mode);
        if ast::is_logical_or_coalescing_binary_operator(operator) {
                let mut parent = left.parent().unwrap().parent().unwrap();
            while ast::is_parenthesized_expression(&parent)
                || is_logical_or_coalescing_binary_expression(&parent)
            {
                parent = parent.parent().unwrap();
            }
            if operator == SyntaxKind::AmpersandAmpersandToken || ast::is_if_statement(&parent) {
                let body = if ast::is_if_statement(&parent) {
                    Some(parent.as_if_statement().then_statement.clone())
                } else {
                    None
                };
                self.check_testing_known_truthy_callable_or_awaitable_or_enum_member_type(
                    left, &left_type, body.as_ref(),
                );
            }
            if ast::is_logical_binary_operator(operator) {
                self.check_truthiness_of_type(left);
            }
        }
        match operator {
            SyntaxKind::AsteriskToken
            | SyntaxKind::AsteriskAsteriskToken
            | SyntaxKind::AsteriskEqualsToken
            | SyntaxKind::AsteriskAsteriskEqualsToken
            | SyntaxKind::SlashToken
            | SyntaxKind::SlashEqualsToken
            | SyntaxKind::PercentToken
            | SyntaxKind::PercentEqualsToken
            | SyntaxKind::MinusToken
            | SyntaxKind::MinusEqualsToken
            | SyntaxKind::LessThanLessThanToken
            | SyntaxKind::LessThanLessThanEqualsToken
            | SyntaxKind::GreaterThanGreaterThanToken
            | SyntaxKind::GreaterThanGreaterThanEqualsToken
            | SyntaxKind::GreaterThanGreaterThanGreaterThanToken
            | SyntaxKind::GreaterThanGreaterThanGreaterThanEqualsToken
            | SyntaxKind::BarToken
            | SyntaxKind::BarEqualsToken
            | SyntaxKind::CaretToken
            | SyntaxKind::CaretEqualsToken
            | SyntaxKind::AmpersandToken
            | SyntaxKind::AmpersandEqualsToken => {
                if self.types_identical_to_silent_never(&left_type, &right_type) {
                    return self.silent_never_type();
                }
                left_type = self.check_non_null_type(&left_type, left);
                right_type = self.check_non_null_type(&right_type, right);
                if left_type.flags.contains(TYPE_FLAGS_BOOLEAN_LIKE)
                    && right_type.flags.contains(TYPE_FLAGS_BOOLEAN_LIKE)
                {
                    let suggested_operator = self.get_suggested_boolean_operator(operator);
                    if suggested_operator != SyntaxKind::Unknown {
                        self.error_message(
                            operator_token,THE_0_OPERATOR_IS_NOT_ALLOWED_FOR_BOOLEAN_TYPES_CONSIDER_USING_1_INSTEAD,
                            &[
                                token_to_string(operator_token.kind).to_string(),
                                token_to_string(suggested_operator).to_string(),
                            ],
                        );
                        return self.number_type();
                    }
                }
                let left_ok = self.check_arithmetic_operand_type(
                    left,
                    &left_type,
                    THE_LEFT_HAND_SIDE_OF_AN_ARITHMETIC_OPERATION_MUST_BE_OF_TYPE_ANY_NUMBER_BIGINT_OR_AN_ENUM_TYPE,
                    true,
                );
                let right_ok = self.check_arithmetic_operand_type(
                    right,
                    &right_type,
                    THE_RIGHT_HAND_SIDE_OF_AN_ARITHMETIC_OPERATION_MUST_BE_OF_TYPE_ANY_NUMBER_BIGINT_OR_AN_ENUM_TYPE,
                    true,
                );
                let result_type;
                if (self.is_type_assignable_to_kind(&left_type, TYPE_FLAGS_ANY_OR_UNKNOWN)
                    && self.is_type_assignable_to_kind(&right_type, TYPE_FLAGS_ANY_OR_UNKNOWN))
                    || (!self.maybe_type_of_kind(&left_type, TYPE_FLAGS_BIG_INT_LIKE)
                        && !self.maybe_type_of_kind(&right_type, TYPE_FLAGS_BIG_INT_LIKE))
                {
                    result_type = self.number_type();
                } else if self.both_are_big_int_like(&left_type, &right_type) {
                    match operator {
                        SyntaxKind::GreaterThanGreaterThanGreaterThanToken
                        | SyntaxKind::GreaterThanGreaterThanGreaterThanEqualsToken => {
                            self.report_operator_error(&left_type, operator, &right_type, error_node.unwrap_or(left), None);
                        }
                        SyntaxKind::AsteriskAsteriskToken
                        | SyntaxKind::AsteriskAsteriskEqualsToken => {
                            if self.language_version < ScriptTarget::ES2016 {
                                self.error_message(
                                    error_node.unwrap_or(left),EXPONENTIATION_CANNOT_BE_PERFORMED_ON_BIGINT_VALUES_UNLESS_THE_TARGET_OPTION_IS_SET_TO_ES2016_OR_LATER,
                                    &[],
                                );
                            }
                        }
                        _ => {}
                    }
                    result_type = self.bigint_type();
                } else {
                    self.report_operator_error(&left_type, operator, &right_type, error_node.unwrap_or(left), None);
                    result_type = self.error_type();
                }
                if left_ok && right_ok {
                    self.check_assignment_operator(left, operator, right, &left_type, &result_type);
                    match operator {
                        SyntaxKind::LessThanLessThanToken
                        | SyntaxKind::LessThanLessThanEqualsToken
                        | SyntaxKind::GreaterThanGreaterThanToken
                        | SyntaxKind::GreaterThanGreaterThanEqualsToken
                        | SyntaxKind::GreaterThanGreaterThanGreaterThanToken
                        | SyntaxKind::GreaterThanGreaterThanGreaterThanEqualsToken => {
                            let rhs_eval = self.evaluate(right, Some(right));
                            if let Some(tsox_frontend::evaluator::EvalValue::Number(num_value)) =
                                &rhs_eval.value
                            {
                                if num_value.abs() >= tsox_core::jsnum::Number(32.0) {
                                    let is_enum_member = ast::is_enum_member(
                                        &walk_up_parenthesized_expressions(
                                            &right.parent().unwrap().parent().unwrap(),
                                        ),
                                    );
                                    self.error_or_suggestion_message(
                                        is_enum_member,
                                        error_node.unwrap_or(left),THIS_OPERATION_CAN_BE_SIMPLIFIED_THIS_SHIFT_IS_IDENTICAL_TO_0_1_2,
                                        &[
                                            get_text_of_node(left),
                                            token_to_string(operator).to_string(),
                                            num_value
                                                .remainder(tsox_core::jsnum::Number(32.0))
                                                .to_string(),
                                        ],
                                    );
                                }
                            }
                        }
                        _ => {}
                    }
                }
                result_type
            }
            SyntaxKind::PlusToken | SyntaxKind::PlusEqualsToken => {
                if self.types_identical_to_silent_never(&left_type, &right_type) {
                    return self.silent_never_type();
                }
                if !self.is_type_assignable_to_kind(&left_type, TYPE_FLAGS_STRING_LIKE)
                    && !self.is_type_assignable_to_kind(&right_type, TYPE_FLAGS_STRING_LIKE)
                {
                    left_type = self.check_non_null_type(&left_type, left);
                    right_type = self.check_non_null_type(&right_type, right);
                }
                let mut result_type: Option<Arc<Type>> = None;
                if self.is_type_assignable_to_kind_ex(&left_type, TYPE_FLAGS_NUMBER_LIKE, true)
                    && self.is_type_assignable_to_kind_ex(&right_type, TYPE_FLAGS_NUMBER_LIKE, true)
                {
                    result_type = Some(self.number_type());
                } else if self.is_type_assignable_to_kind_ex(&left_type, TYPE_FLAGS_BIG_INT_LIKE, true)
                    && self.is_type_assignable_to_kind_ex(&right_type, TYPE_FLAGS_BIG_INT_LIKE, true)
                {
                    result_type = Some(self.bigint_type());
                } else if self.is_type_assignable_to_kind_ex(&left_type, TYPE_FLAGS_STRING_LIKE, true)
                    || self.is_type_assignable_to_kind_ex(&right_type, TYPE_FLAGS_STRING_LIKE, true)
                {
                    result_type = Some(self.string_type());
                } else if is_type_any(&left_type) || is_type_any(&right_type) {
                    if self.is_error_type(&left_type) || self.is_error_type(&right_type) {
                        result_type = Some(self.error_type());
                    } else {
                        result_type = Some(self.any_type());
                    }
                }
                if let Some(ref rt) = result_type {
                    if !self.check_for_disallowed_essymbol_operand(left, right, &left_type, &right_type, operator) {
                        return Arc::clone(rt);
                    }
                }
                let Some(result_type) = result_type else {
                    let close_enough_kind =
                        TYPE_FLAGS_NUMBER_LIKE | TYPE_FLAGS_BIG_INT_LIKE | TYPE_FLAGS_STRING_LIKE | TYPE_FLAGS_ANY_OR_UNKNOWN;
                    self.report_operator_error(&left_type, operator, &right_type, error_node.unwrap_or(left), None);
                    return self.any_type();
                };
                if operator == SyntaxKind::PlusEqualsToken {
                    self.check_assignment_operator(left, operator, right, &left_type, &result_type);
                }
                result_type
            }
            SyntaxKind::LessThanToken
            | SyntaxKind::GreaterThanToken
            | SyntaxKind::LessThanEqualsToken
            | SyntaxKind::GreaterThanEqualsToken => {
                if self.check_for_disallowed_essymbol_operand(left, right, &left_type, &right_type, operator) {
                    let non_null_left = self.check_non_null_type(&left_type, left);
                    left_type = self.get_base_type_of_literal_type_for_comparison(&non_null_left);
                    let non_null_right = self.check_non_null_type(&right_type, right);
                    right_type = self.get_base_type_of_literal_type_for_comparison(&non_null_right);
                    let number_or_big_int = Arc::clone(&self.number_or_big_int_type);
                    let types_compatible = if is_type_any(&left_type) || is_type_any(&right_type) {
                        true
                    } else {
                        let left_assignable_to_number = self.is_type_assignable_to(&left_type, &number_or_big_int);
                        let right_assignable_to_number = self.is_type_assignable_to(&right_type, &number_or_big_int);
                        (left_assignable_to_number && right_assignable_to_number)
                            || (!left_assignable_to_number
                                && !right_assignable_to_number
                                && self.are_types_comparable(&left_type, &right_type))
                    };
                    if !types_compatible {
                        self.report_operator_error(&left_type, operator, &right_type, error_node.unwrap_or(left), None);
                    }
                }
                self.boolean_type()
            }
            SyntaxKind::EqualsEqualsToken
            | SyntaxKind::ExclamationEqualsToken
            | SyntaxKind::EqualsEqualsEqualsToken
            | SyntaxKind::ExclamationEqualsEqualsToken => {
                if !check_mode.contains(CheckMode::TypeOnly) {
                    if is_literal_expression_of_object(left)
                        || is_literal_expression_of_object(right)
                    {
                        let eq_type = operator == SyntaxKind::EqualsEqualsToken
                            || operator == SyntaxKind::EqualsEqualsEqualsToken;
                        self.error_message(
                            error_node.unwrap_or(left),THIS_CONDITION_WILL_ALWAYS_RETURN_0_SINCE_JAVASCRIPT_COMPARES_OBJECTS_BY_REFERENCE_NOT_VALUE,
                            &[if eq_type { "false".to_string() } else { "true".to_string() }],
                        );
                    }
                    self.check_nan_equality(error_node.unwrap_or(left), operator, left, right);
                    let types_compatible = self.is_type_equality_comparable_to(&left_type, &right_type)
                        || self.is_type_equality_comparable_to(&right_type, &left_type);
                    if !types_compatible {
                        self.report_operator_error(&left_type, operator, &right_type, error_node.unwrap_or(left), None);
                    }
                }
                self.boolean_type()
            }
            SyntaxKind::InstanceOfKeyword => {
                self.check_instance_of_expression(left, right, &left_type, &right_type, check_mode)
            }
            SyntaxKind::InKeyword => {
                let bin = left.parent().unwrap();
                let silent = self.silent_never_type();
                if Arc::ptr_eq(&left_type, &silent) || Arc::ptr_eq(&right_type, &silent) {
                    silent
                } else {
                    self.check_in_expression(bin.as_binary_expression());
                    self.boolean_type()
                }
            }
            SyntaxKind::AmpersandAmpersandToken | SyntaxKind::AmpersandAmpersandEqualsToken => {
                let mut result_type = Arc::clone(&left_type);
                if self.has_type_facts(&left_type, TypeFacts::TRUTHY) {
                    let mut t = Arc::clone(&left_type);
                    if !self.strict_null_checks {
                        t = self.get_base_type_of_literal_type(&right_type);
                    }
                    let falsy_types = self.extract_definitely_falsy_types(&t);
                    result_type = self.get_union_type(vec![falsy_types, right_type.clone()]);
                }
                if operator == SyntaxKind::AmpersandAmpersandEqualsToken {
                    self.check_assignment_operator(left, operator, right, &left_type, &right_type);
                }
                result_type
            }
            SyntaxKind::BarBarToken | SyntaxKind::BarBarEqualsToken => {
                let mut result_type = Arc::clone(&left_type);
                if self.has_type_facts(&left_type, TypeFacts::FALSY) {
                    let removed_falsy = self.remove_definitely_falsy_types(&left_type);
                    let non_nullable = self.get_non_nullable_type(&removed_falsy);
                    result_type = self.get_union_type_ex(
                        vec![non_nullable, right_type.clone()],
                        UnionReduction::Subtype,
                    );
                }
                if operator == SyntaxKind::BarBarEqualsToken {
                    self.check_assignment_operator(left, operator, right, &left_type, &right_type);
                }
                result_type
            }
            SyntaxKind::QuestionQuestionToken | SyntaxKind::QuestionQuestionEqualsToken => {
                if operator == SyntaxKind::QuestionQuestionToken {
                    self.check_nullish_coalesce_operands(left, right);
                }
                let mut result_type = Arc::clone(&left_type);
                if self.has_type_facts(&left_type, TypeFacts::EQ_UNDEFINED_OR_NULL) {
                    let non_nullable = self.get_non_nullable_type(&left_type);
                    result_type = self.get_union_type_ex(
                        vec![non_nullable, right_type.clone()],
                        UnionReduction::Subtype,
                    );
                }
                if operator == SyntaxKind::QuestionQuestionEqualsToken {
                    self.check_assignment_operator(left, operator, right, &left_type, &right_type);
                }
                result_type
            }
            SyntaxKind::EqualsToken => {
                self.check_assignment_operator(left, operator, right, &left_type, &right_type);
                right_type
            }
            SyntaxKind::CommaToken => {
                if !self.compiler_options.allow_unreachable_code.is_true()
                    && self.is_side_effect_free(left)
                    && !self.is_indirect_call(&left.parent().unwrap())
                {
                    let sf = self.get_source_file_of_node(left).unwrap();
                    let start = tsox_frontend::scanner::skip_trivia(&sf.text, left.pos());
                    let is_in_diag_2657 = tsox_frontend::ast::mig::m3b_2::diagnostics(&sf)
                        .iter()
                        .any(|d| {
                            d.code == JSX_EXPRESSIONS_MUST_HAVE_ONE_PARENT_ELEMENT.code
                                && d.loc.contains(start)
                        });
                    if !is_in_diag_2657 {
                        self.error_message(left,LEFT_SIDE_OF_COMMA_OPERATOR_IS_UNUSED_AND_HAS_NO_SIDE_EFFECTS, &[]);
                    }
                }
                right_type
            }
            _ => panic!("Unhandled case in checkBinaryLikeExpression"),
        }
    }

    pub fn check_call_expression(&mut self, node: &Arc<Node>, check_mode: CheckMode) -> Arc<Type> {
        if let Some(type_arguments) = tsox_frontend::ast::mig::m3c::type_argument_list(node) {
            self.check_grammar_type_arguments(node, type_arguments);
        }
        let signature = self.get_resolved_signature(node);
        if signature
            .as_ref()
            .is_some_and(|sig| self.signature_is_resolving_signature(sig))
        {
            return self.silent_never_type();
        }
        if let Some(sig) = &signature {
            self.check_deprecated_signature(sig, node);
        }
        if node.expression().map(|e| e.kind == SyntaxKind::SuperKeyword).unwrap_or(false) {
            return self.void_type();
        }
        if ast::is_new_expression(node) {
            let declaration = signature.as_ref().and_then(|sig| sig.declaration.clone());
            if let Some(declaration) = declaration {
                if !ast::is_constructor_declaration(&declaration)
                    && !ast::is_construct_signature_declaration(&declaration)
                    && !ast::is_constructor_type_node(&declaration)
                {
                    if self.no_implicit_any {
                        self.error_message(
                            node,X_NEW_EXPRESSION_WHOSE_TARGET_LACKS_A_CONSTRUCT_SIGNATURE_IMPLICITLY_HAS_AN_ANY_TYPE,
                            &[],
                        );
                    }
                    return self.any_type();
                }
            }
        }
        if ast::is_in_js_file(node) && self.is_commonjs_require(node) {
            return self.resolve_external_module_type_by_literal(&node.arguments().unwrap().nodes[0]);
        }
        let return_type = signature
            .as_ref()
            .and_then(|sig| self.get_return_type_of_signature(sig))
            .unwrap_or_else(|| self.error_type());
        if return_type.flags.contains(TYPE_FLAGS_ES_SYMBOL_LIKE) && self.is_symbol_or_symbol_for_call(node) {
            return self
                .get_essymbol_like_type_for_node(&walk_up_parenthesized_expressions(&node.parent().unwrap()));
        }
        if ast::is_call_expression(node)
            && node.question_dot_token().is_none()
            && ast::is_expression_statement(&node.parent().unwrap())
            && return_type.flags.contains(TypeFlags::Void)
            && signature
                .as_ref()
                .and_then(|sig| self.get_type_predicate_of_signature(sig))
                .is_some()
        {
            let expression = node.expression().unwrap();
            if !is_dotted_name(&expression) {
                self.error_message(
                    expression,ASSERTIONS_REQUIRE_THE_CALL_TARGET_TO_BE_AN_IDENTIFIER_OR_QUALIFIED_NAME,
                    &[],
                );
            } else if self.get_effects_signature(node).is_none() {
                self.error_message(
                    expression,ASSERTIONS_REQUIRE_EVERY_NAME_IN_THE_CALL_TARGET_TO_BE_DECLARED_WITH_AN_EXPLICIT_TYPE_ANNOTATION,
                    &[],
                );
                self.get_type_of_dotted_name(expression, None);
            }
        }
        return_type
    }

    pub fn check_class_expression(&mut self, node: &Arc<Node>) -> Arc<Type> {
        self.check_class_like_declaration(node);
        self.check_node_deferred(node);
        self.check_class_expression_external_helpers(node);
        let symbol = self.get_symbol_of_declaration(node).unwrap();
        self.get_type_of_symbol(&symbol)
    }

    pub fn check_class_like_declaration(&mut self, node: &Arc<Node>) {
        let _ = self.check_grammar_class_like_declaration(node);
        self.check_decorators(node);
        self.check_collisions_for_declaration_name(node, node.name());
        let type_parameters_list: Option<&tsox_frontend::ast::NodeList> = match &node.data {
            NodeData::ClassDeclaration(d) => d.type_parameters.as_deref(),
            NodeData::ClassExpression(d) => d.type_parameters.as_deref(),
            _ => None,
        };
        self.check_type_parameters(type_parameters_list);
        self.check_exports_on_merged_declarations(node);
        let symbol = self.get_symbol_of_declaration(node).unwrap();
        let class_type = self.get_declared_type_of_symbol(&symbol);
        let class_this_type = class_type
            .as_interface_type()
            .and_then(|i| i.this_type.clone());
        let type_with_this = self.get_type_with_this_argument(&class_type, None, false);
        let static_type = self.get_type_of_symbol(&symbol);
        self.check_type_parameter_lists_identical(&symbol);
        self.check_function_or_constructor_symbol(&symbol);
        self.check_object_type_for_duplicate_declarations(node, true);
        if !node.flags.contains(ast::NodeFlags::Ambient) {
            self.check_class_for_static_property_name_conflicts(node);
        }
        if let Some(base_type_node) = tsox_frontend::ast::get_class_extends_heritage_element(node)
            .into_iter()
            .next()
        {
            let base_type_args: &[Arc<Node>] = match &base_type_node.data {
                NodeData::ExpressionWithTypeArguments(d) => d
                    .type_arguments
                    .as_ref()
                    .map(|l| l.nodes.as_slice())
                    .unwrap_or(&[]),
                _ => &[],
            };
            self.check_source_elements(base_type_args);
            let base_types = self.get_base_types(&class_type);
            if !base_types.is_empty() {
                let base_type = Arc::clone(&base_types[0]);
                self.check_jsdoc_augments_tag_matches_extends(node, &base_type_node, &base_type);
                let base_constructor_type = self
                    .get_base_constructor_type_of_class(&class_type)
                    .unwrap_or_else(|| self.error_type());
                let static_base_type = self.get_apparent_type(&base_constructor_type);
                self.check_base_type_accessibility(&static_base_type, &base_type_node);
                if let Some(base_expression) = base_type_node.expression() {
                    self.check_source_element(&base_expression);
                }
                if !base_type_args.is_empty() {
                    self.check_source_elements(base_type_args);
                    for constructor in self.get_constructors_for_type_arguments(
                        &static_base_type,
                        base_type_args,
                        &base_type_node,
                    ) {
                        if !self.check_type_argument_constraints(
                            &base_type_node,
                            &constructor.type_parameters,
                        ) {
                            break;
                        }
                    }
                }
                let base_with_this = self
                    .get_type_with_this_argument(&base_type, class_this_type.as_ref(), false);
                if !self.check_type_assignable_to(&type_with_this, &base_with_this, None, None) {
                    self.issue_member_specific_error(
                        node,
                        &type_with_this,
                        &base_with_this,
                        CLASS_0_INCORRECTLY_EXTENDS_BASE_CLASS_1.clone(),
                    );
                } else {
                    let static_base_without_signatures =
                        self.get_type_without_signatures(&static_base_type);
                    self.check_type_assignable_to(
                        &static_type,
                        &static_base_without_signatures,
                        node.name().or(Some(node)),
                        Some(&CLASS_STATIC_SIDE_0_INCORRECTLY_EXTENDS_BASE_CLASS_STATIC_SIDE_1),
                    );
                }
                if base_constructor_type
                    .flags
                    .intersects(TYPE_FLAGS_TYPE_VARIABLE)
                {
                    if !self.is_mixin_constructor_type(&static_type) {
                        self.error_message(
                            node.name().unwrap_or(node),A_MIXIN_CLASS_MUST_HAVE_A_CONSTRUCTOR_WITH_A_SINGLE_REST_PARAMETER_OF_TYPE_ANY,
                            &[],
                        );
                    } else {
                        let construct_signatures = self.get_signatures_of_type(
                            &base_constructor_type,
                            SignatureKind::Construct,
                        );
                        if construct_signatures
                            .iter()
                            .any(|sig| sig.flags.contains(SignatureFlags::Abstract))
                            && !node.has_syntactic_modifier(ast::ModifierFlags::Abstract)
                        {
                            self.error_message(
                                node.name().unwrap_or(node),A_MIXIN_CLASS_THAT_EXTENDS_FROM_A_TYPE_VARIABLE_CONTAINING_AN_ABSTRACT_CONSTRUCT_SIGNATURE_MUST_ALSO_BE_DECLARED_ABSTRACT,
                                &[],
                            );
                        }
                    }
                }
                let static_is_class = static_base_type
                    .symbol()
                    .is_some_and(|s| s.flags.intersects(ast::SymbolFlags::Class));
                if !static_is_class
                    && !base_constructor_type
                        .flags
                        .intersects(TYPE_FLAGS_TYPE_VARIABLE)
                {
                    let constructors = self.get_instantiated_constructors_for_type_arguments(
                        &static_base_type,
                        base_type_args,
                        Some(&base_type_node),
                    );
                    let all_same_return = constructors.iter().all(|sig| {
                        let return_type = self.get_return_type_of_signature(sig);
                        return_type.is_some_and(|rt| self.is_type_identical_to(&rt, &base_type))
                    });
                    if !all_same_return {
                        if let Some(base_expression) = base_type_node.expression() {
                            self.error_message(
                                &base_expression,BASE_CONSTRUCTORS_MUST_ALL_HAVE_THE_SAME_RETURN_TYPE,
                                &[],
                            );
                        }
                    }
                }
                self.check_kinds_of_property_member_overrides(&class_type, &base_type);
            }
        }
        self.check_members_for_override_modifier(node);
        let implemented_type_nodes =
            tsox_frontend::ast::get_implements_heritage_clause_elements(node);
        for type_ref_node in &implemented_type_nodes {
            if tsox_frontend::ast::node_data_generated::is_expression_with_type_arguments(
                type_ref_node,
            ) {
                if let Some(expr) = type_ref_node.expression() {
                    if !tsox_frontend::ast::is_entity_name_expression(&expr)
                        || tsox_frontend::ast::is_optional_chain(&expr)
                    {
                        self.error_message(
                            &expr,A_CLASS_CAN_ONLY_IMPLEMENT_AN_IDENTIFIER_SLASHQUALIFIED_NAME_WITH_OPTIONAL_TYPE_ARGUMENTS,
                            &[],
                        );
                    }
                }
            }
            self.check_type_reference_node(type_ref_node);
            let type_from_node = self.get_type_from_type_node(type_ref_node);
            let t = self.get_reduced_type(&type_from_node);
            if !self.is_error_type(&t) {
                if self.is_valid_base_type(&t) {
                    let generic_diag =
                        if t.symbol().is_some_and(|s| s.flags.intersects(ast::SymbolFlags::Class))
                        {
                            &CLASS_0_INCORRECTLY_IMPLEMENTS_CLASS_1_DID_YOU_MEAN_TO_EXTEND_1_AND_INHERIT_ITS_MEMBERS_AS_A_SUBCLASS
                        } else {
                            &CLASS_0_INCORRECTLY_IMPLEMENTS_INTERFACE_1
                        };
                    let base_with_this =
                        self.get_type_with_this_argument(&t, class_this_type.as_ref(), false);
                    if !self.check_type_assignable_to(&type_with_this, &base_with_this, None, None)
                    {
                        self.issue_member_specific_error(
                            node,
                            &type_with_this,
                            &base_with_this,
                            generic_diag.clone(),
                        );
                    }
                } else {
                    self.error_message(
                        type_ref_node,A_CLASS_CAN_ONLY_IMPLEMENT_AN_OBJECT_TYPE_OR_INTERSECTION_OF_OBJECT_TYPES_WITH_STATICALLY_KNOWN_MEMBERS,
                        &[],
                    );
                }
            }
        }
        self.check_index_constraints(&class_type, node);
        self.check_index_constraints(&static_type, node);
        self.check_class_or_interface_for_duplicate_index_signatures(node);
        self.check_property_initialization(node);
    }

    pub fn check_class_expression_deferred(&mut self, node: &Arc<Node>) {
        let members = node.members().map(|m| m.nodes.as_slice()).unwrap_or(&[]);
        self.check_source_elements(members);
        self.register_for_unused_identifiers_check(node);
    }

    pub fn check_class_expression_external_helpers(&mut self, node: &Arc<Node>) {
        if node.name().is_some() {
            return;
        }
        let parent = walk_up_outer_expressions(node).unwrap_or_else(|| Arc::clone(node));
        if !is_named_evaluation_source(&parent) {
            return;
        }
        let will_transform_es_decorators = !self.legacy_decorators
            && self.language_version < ScriptTarget::ES2022;
        let location = if will_transform_es_decorators
            && class_or_constructor_parameter_is_decorated(false, node)
        {
            let decorators = node.decorators();
            decorators
                .first()
                .map(|d| Arc::clone(*d))
                .unwrap_or_else(|| Arc::clone(node))
        } else {
            match self.get_first_transformable_static_class_element(node) {
                Some(l) => l,
                None => return,
            }
        };
        self.check_external_emit_helpers(&location, ExternalEmitHelpers::SetFunctionName.bits());
        if (ast::is_property_assignment(&parent)
            || ast::is_property_declaration(&parent)
            || ast::is_binding_element(&parent))
            && parent.name().is_some_and(|n| ast::is_computed_property_name(n))
        {
            self.check_external_emit_helpers(&location, ExternalEmitHelpers::PropKey.bits());
        }
    }

    pub fn get_first_transformable_static_class_element(&mut self, node: &Arc<Node>) -> Option<Arc<Node>> {
        let will_transform_static_elements_of_decorated_class = !self.legacy_decorators
            && self.language_version < ScriptTarget::ES2022
            && class_or_constructor_parameter_is_decorated(false, node);
        let will_transform_private_elements_or_class_static_blocks =
            self.language_version < ScriptTarget::ES2022
                || self.language_version < ScriptTarget::ES2022;
        let will_transform_initializers = !self.emit_standard_class_fields;
        if will_transform_static_elements_of_decorated_class
            || will_transform_private_elements_or_class_static_blocks
        {
            let members = node.members().map(|m| m.nodes.as_slice()).unwrap_or(&[]);
            for member in members {
                if will_transform_static_elements_of_decorated_class
                    && class_element_or_class_element_parameter_is_decorated(false, member, node)
                {
                    if let Some(first_decorator) =
                        node.decorators().first().map(|d| Arc::clone(*d))
                    {
                        return Some(first_decorator);
                    }
                    return Some(Arc::clone(node));
                } else if will_transform_private_elements_or_class_static_blocks {
                    if ast::is_class_static_block_declaration(member) {
                        return Some(Arc::clone(member));
                    } else if ast::is_static(member)
                        && (is_private_identifier_class_element_declaration(member)
                            || (will_transform_initializers && is_initialized_property(member)))
                    {
                        return Some(Arc::clone(member));
                    }
                }
            }
        }
        None
    }

    pub fn check_class_expression_deferred_unused_placeholder(&mut self) {}

    pub fn check_class_name_collision_with_object(&mut self, name: &Arc<Node>) {
        if name.text() == "Object"
            && self
                .program
                .get_emit_module_format_of_file(
                    &self.get_source_file_of_node(name).unwrap().file_name,
                )
                < ModuleKind::ES2015
        {
            self.error_message(
                name,CLASS_NAME_CANNOT_BE_OBJECT_WHEN_TARGETING_ES5_AND_ABOVE_WITH_MODULE_0,
                &[module_kind_string(self.module_kind)],
            );
        }
    }

    pub fn check_collision_with_global_object_in_generated_code(
        &mut self,
        node: &Arc<Node>,
        name: Option<&Arc<Node>>,
    ) {
        let Some(name) = name else { return };
        if ast::is_class_like(node) || !self.need_collision_check_for_identifier(node, Some(name), "Object") {
            return;
        }
        if ast::is_module_declaration(node)
            && ast::get_module_instance_state(node) != ModuleInstanceState::Instantiated
        {
            return;
        }
        let Some(parent) = Checker::get_declaration_container(node) else { return };
        if ast::is_source_file(&parent)
            && self
                .get_source_file_of_node(&parent)
                .is_some_and(|sf| ast::is_external_or_common_js_module(&sf))
            && self
                .program
                .get_emit_module_format_of_file(
                    &self.get_source_file_of_node(&parent).unwrap().file_name,
                )
                == ModuleKind::CommonJS
        {
            self.error_skipped_on_no_emit_message(
                name,DUPLICATE_IDENTIFIER_0_COMPILER_RESERVES_NAME_1_IN_TOP_LEVEL_SCOPE_OF_A_MODULE,
                &[declaration_name_to_string(Some(name)), declaration_name_to_string(Some(name))],
            );
        }
    }

    pub fn check_collision_with_global_promise_in_generated_code(
        &mut self,
        node: &Arc<Node>,
        name: Option<&Arc<Node>>,
    ) {
        let Some(name) = name else { return };
        if self.language_version >= ScriptTarget::ES2017
            || !self.need_collision_check_for_identifier(node, Some(name), "Promise")
        {
            return;
        }
        if ast::is_module_declaration(node)
            && ast::get_module_instance_state(node) != ModuleInstanceState::Instantiated
        {
            return;
        }
        let Some(parent) = Checker::get_declaration_container(node) else { return };
        if ast::is_source_file(&parent)
            && self
                .get_source_file_of_node(&parent)
                .is_some_and(|sf| ast::is_external_or_common_js_module(&sf))
            && parent.flags.contains(ast::NodeFlags::HasAsyncFunctions)
        {
            self.error_skipped_on_no_emit_message(
                name,DUPLICATE_IDENTIFIER_0_COMPILER_RESERVES_NAME_1_IN_TOP_LEVEL_SCOPE_OF_A_MODULE_CONTAINING_ASYNC_FUNCTIONS,
                &[declaration_name_to_string(Some(name)), declaration_name_to_string(Some(name))],
            );
        }
    }

    pub fn check_collision_with_require_exports_in_generated_code(
        &mut self,
        node: &Arc<Node>,
        name: Option<&Arc<Node>>,
    ) {
        if self
            .program
            .get_emit_module_format_of_file(
                &self.get_source_file_of_node(node).unwrap().file_name,
            )
            >= ModuleKind::ES2015
        {
            return;
        }
        let Some(name) = name else { return };
        if !self.need_collision_check_for_identifier(node, Some(name), "require")
            && !self.need_collision_check_for_identifier(node, Some(name), "exports")
        {
            return;
        }
        if ast::is_module_declaration(node)
            && ast::get_module_instance_state(node) != ModuleInstanceState::Instantiated
        {
            return;
        }
        let Some(parent) = Checker::get_declaration_container(node) else { return };
        if ast::is_source_file(&parent)
            && self
                .get_source_file_of_node(&parent)
                .is_some_and(|sf| ast::is_external_or_common_js_module(&sf))
        {
            self.error_skipped_on_no_emit_message(
                name,DUPLICATE_IDENTIFIER_0_COMPILER_RESERVES_NAME_1_IN_TOP_LEVEL_SCOPE_OF_A_MODULE,
                &[declaration_name_to_string(Some(name)), declaration_name_to_string(Some(name))],
            );
        }
    }

    pub fn check_collisions_for_declaration_name(&mut self, node: &Arc<Node>, name: Option<&Arc<Node>>) {
        let Some(name) = name else { return };
        self.check_collision_with_require_exports_in_generated_code(node, Some(name));
        self.check_collision_with_global_object_in_generated_code(node, Some(name));
        self.check_collision_with_global_promise_in_generated_code(node, Some(name));
        self.record_potential_collision_with_weakmap_set_in_generated_code(node, name);
        self.record_potential_collision_with_reflect_in_generated_code(node, name);
        if ast::is_class_like(node) {
            self.check_type_name_is_reserved(name, CLASS_NAME_CANNOT_BE_0);
            if !node.flags.contains(ast::NodeFlags::Ambient) {
                self.check_class_name_collision_with_object(name);
            }
        } else if ast::is_enum_declaration(node) {
            self.check_type_name_is_reserved(name, ENUM_NAME_CANNOT_BE_0);
        }
    }

    pub fn record_potential_collision_with_weakmap_set_in_generated_code(
        &mut self,
        node: &Arc<Node>,
        name: &Arc<Node>,
    ) {
        if self.language_version <= ScriptTarget::ES2021
            && (self.need_collision_check_for_identifier(node, Some(name), "WeakMap")
                || self.need_collision_check_for_identifier(node, Some(name), "WeakSet"))
        {
            let node = Arc::clone(node);
            let checker_ptr =
                crate::checker::mig::m2f_4::r28k4_defs::SendCheckerPtr::from_checker(self);
            self.add_deferred_diagnostic(Box::new(move || {
                let c = unsafe { checker_ptr.get() };
                c.check_weakmap_set_collision(&node);
            }));
        }
    }

    pub fn check_weakmap_set_collision(&mut self, node: &Arc<Node>) {
        let enclosing_block_scope = get_enclosing_block_scope_container(node);
        if let Some(scope) = enclosing_block_scope {
            if self
                .get_node_check_flags(&scope)
                .contains(NodeCheckFlags::ContainsClassWithPrivateIdentifiers)
            {
                let name = node.name();
                if let Some(name) = name {
                    if ast::is_identifier(name) {
                        self.error_skipped_on_no_emit_message(
                            node,COMPILER_RESERVES_NAME_0_WHEN_EMITTING_PRIVATE_IDENTIFIER_DOWNLEVEL,
                            &[name.text().to_string()],
                        );
                    }
                }
            }
        }
    }

    pub fn record_potential_collision_with_reflect_in_generated_code(
        &mut self,
        node: &Arc<Node>,
        name: &Arc<Node>,
    ) {
        if self.language_version <= ScriptTarget::ES2021
            && self.need_collision_check_for_identifier(node, Some(name), "Reflect")
        {
            let node = Arc::clone(node);
            let checker_ptr =
                crate::checker::mig::m2f_4::r28k4_defs::SendCheckerPtr::from_checker(self);
            self.add_deferred_diagnostic(Box::new(move || {
                let c = unsafe { checker_ptr.get() };
                c.check_reflect_collision(&node);
            }));
        }
    }

    pub fn check_reflect_collision(&mut self, node: &Arc<Node>) {
        let mut has_collision = false;
        if ast::is_class_expression(node) {
            let members = node.members().map(|m| m.nodes.as_slice()).unwrap_or(&[]);
            for member in members {
                if self
                    .get_node_check_flags(member)
                    .contains(NodeCheckFlags::ContainsSuperPropertyInStaticInitializer)
                {
                    has_collision = true;
                    break;
                }
            }
        } else if ast::is_function_expression(node) {
            if self
                .get_node_check_flags(node)
                .contains(NodeCheckFlags::ContainsSuperPropertyInStaticInitializer)
            {
                has_collision = true;
            }
        } else {
            let container = get_enclosing_block_scope_container(node);
            if let Some(container) = container {
                if self
                    .get_node_check_flags(&container)
                    .contains(NodeCheckFlags::ContainsSuperPropertyInStaticInitializer)
                {
                    has_collision = true;
                }
            }
        }
        if has_collision {
            let name = node.name();
            if let Some(name) = name {
                if ast::is_identifier(name) {
                    self.error_skipped_on_no_emit_message(
                        node,DUPLICATE_IDENTIFIER_0_COMPILER_RESERVES_NAME_1_WHEN_EMITTING_SUPER_REFERENCES_IN_STATIC_INITIALIZERS,
                        &[declaration_name_to_string(Some(name)), "Reflect".to_string()],
                    );
                }
            }
        }
    }

    pub fn check_conditional_expression(&mut self, node: &Arc<Node>, check_mode: CheckMode) -> Arc<Type> {
        let cond = node.as_conditional_expression();
        let t = self.check_truthiness_expression(&cond.condition, check_mode);
        self.check_testing_known_truthy_callable_or_awaitable_or_enum_member_type(
            &cond.condition,
            &t,
            Some(&cond.when_true),
        );
        let type1 = self.check_expression_ex(&cond.when_true, check_mode);
        let type2 = self.check_expression_ex(&cond.when_false, check_mode);
        self.get_union_type_ex(vec![type1, type2], UnionReduction::Subtype)
    }

    pub fn check_constructor_declaration(&mut self, node: &Arc<Node>) {
        self.check_signature_declaration(node);
        if !self.check_grammar_constructor_type_parameters(node) {
            self.check_grammar_constructor_type_annotation(node);
        }
        if let Some(body) = node.body() {
            self.check_source_element(&body);
        }
        let symbol = self.get_symbol_of_declaration(node).unwrap();
        self.check_function_or_constructor_symbol(&symbol);
        let body = match node.body() {
            Some(b) if !ast::node_is_missing(Some(&b)) => b,
            _ => return,
        };
        let containing_class_decl = node.parent().unwrap();
        if ast::get_class_extends_heritage_element(&containing_class_decl).is_none() {
            return;
        }
        let class_extends_null = self.class_declaration_extends_null(&containing_class_decl);
        let super_call = find_first_super_call(&body);
        if let Some(super_call) = super_call {
            if class_extends_null {
                self.error_message(&super_call,A_CONSTRUCTOR_CANNOT_CONTAIN_A_SUPER_CALL_WHEN_ITS_CLASS_EXTENDS_NULL, &[]);
            }
            let super_call_should_be_root_level = !self.emit_standard_class_fields
                && (node.parent().unwrap().members().map(|m| {
                    m.nodes.iter().any(|m| {
                        is_instance_property_with_initializer_or_private_identifier_property(m)
                    })
                }).unwrap_or(false)
                || node
                    .parameters()
                    .map(|p| p.nodes.as_slice())
                    .unwrap_or(&[])
                    .iter()
                    .any(|p| {
                        ast::has_syntactic_modifier(p, ast::ModifierFlags::ParameterPropertyModifier)
                    }));
            if super_call_should_be_root_level {
                if !super_call_is_root_level_in_constructor(&super_call, &body) {
                    self.error_message(&super_call,A_SUPER_CALL_MUST_BE_A_ROOT_LEVEL_STATEMENT_WITHIN_A_CONSTRUCTOR_OF_A_DERIVED_CLASS_THAT_CONTAINS_INITIALIZED_PROPERTIES_PARAMETER_PROPERTIES_OR_PRIVATE_IDENTIFIERS, &[]);
                } else {
                    let mut super_call_statement: Option<Arc<Node>> = None;
                    for statement in tsox_frontend::ast::mig::m3c::statements(&body) {
                        if ast::is_expression_statement(statement)
                            && is_super_call(&skip_outer_expressions(
                                &statement.expression().unwrap(),
                                OuterExpressionKinds::all(),
                            ))
                        {
                            super_call_statement = Some(Arc::clone(statement));
                            break;
                        }
                        if node_immediately_references_super_or_this(statement) {
                            break;
                        }
                    }
                    if super_call_statement.is_none() {
                        self.error_message(node,A_SUPER_CALL_MUST_BE_THE_FIRST_STATEMENT_IN_THE_CONSTRUCTOR_TO_REFER_TO_SUPER_OR_THIS_WHEN_A_DERIVED_CLASS_CONTAINS_INITIALIZED_PROPERTIES_PARAMETER_PROPERTIES_OR_PRIVATE_IDENTIFIERS, &[]);
                    }
                }
            }
        } else if !class_extends_null {
            self.error_message(node,CONSTRUCTORS_FOR_DERIVED_CLASSES_MUST_CONTAIN_A_SUPER_CALL, &[]);
        }
    }

    pub fn check_contextual_deprecations(&mut self, node: &Arc<Node>) {
        let contextual_type = self.get_apparent_type_of_contextual_type(node, ContextFlags::None);
        for property in tsox_frontend::ast::mig::m3b::properties(node) {
            if self.is_canceled() {
                return;
            }
            if let Some(name) = property.name() {
                if !ast::is_computed_property_name(name) {
                    self.check_deprecated_property(name, contextual_type.as_ref());
                }
            }
        }
    }

    pub fn check_deprecated_property(&mut self, name: &Arc<Node>, contextual_type: Option<&Arc<Type>>) {
        let Some(contextual_type) = contextual_type else { return };
        let prop = self.get_property_of_type(contextual_type, &name.text());
        let Some(prop) = prop else { return };
        if prop.declarations.is_empty() {
            return;
        }
        if self.is_deprecated_symbol(&prop) {
            self.add_deprecated_suggestion(name, &prop.declarations, &name.text());
        }
    }

    pub fn check_declaration_initializer(
        &mut self,
        declaration: &Arc<Node>,
        check_mode: CheckMode,
        contextual_type: Option<&Arc<Type>>,
    ) -> Arc<Type> {
        let initializer = declaration.initializer().unwrap();
        let mut t = self.get_quick_type_of_expression(&initializer);
        let t = match t {
            Some(t) => t,
            None => match contextual_type {
                Some(ct) => self.check_expression_with_contextual_type(
                    &initializer,
                    ct,
                    None,
                    check_mode,
                ),
                None => self.check_expression_cached_ex(&initializer, check_mode),
            },
        };
        if ast::is_parameter_declaration(&ast::get_root_declaration(declaration)) {
            let name = declaration.name().unwrap();
            match name.kind {
                SyntaxKind::ObjectBindingPattern => {
                    if is_object_literal_type(&t) {
                        return self.pad_object_literal_type(&t, name);
                    }
                }
                SyntaxKind::ArrayBindingPattern => {
                    if is_tuple_type(&t) {
                        return self.pad_tuple_type(&t, name);
                    }
                }
                _ => {}
            }
        }
        t
    }

    pub fn pad_object_literal_type(&mut self, t: &Arc<Type>, pattern: &Arc<Node>) -> Arc<Type> {
        let mut missing_elements: Vec<Arc<Node>> = Vec::new();
        let pattern_elements = pattern.elements().map(|l| l.nodes.as_slice()).unwrap_or(&[]);
        for e in pattern_elements {
            if e.initializer().is_some() {
                let name = self.get_property_name_from_binding_element(e);
                if name != crate::checker::mig::r18k6_defs::InternalSymbolName::Missing.to_string()
                    && self.get_property_of_type(t, &name).is_none()
                {
                    missing_elements.push(Arc::clone(e));
                }
            }
        }
        if missing_elements.is_empty() {
            return Arc::clone(t);
        }
        let mut members = SymbolTable::new();
        for prop in self.get_properties_of_object_type(t) {
            members.insert(prop.name.clone(), prop);
        }
        for e in &missing_elements {
            let name = self.get_property_name_from_binding_element(e);
            let symbol = self.new_symbol(
                tsox_frontend::ast::SymbolFlags::Property
                    | tsox_frontend::ast::SymbolFlags::Optional,
                &name,
            );
            let resolved = self.get_type_from_binding_element(e, false, false);
            self.value_symbol_links.get_or_default(&symbol).resolved_type = Some(resolved);
            members.insert(symbol.name.clone(), symbol);
        }
        let result = self.new_object_type(t.object_flags, t.symbol.clone());
        let index_infos = self.get_index_infos_of_type(t);
        self.set_structured_type_members(
            &result,
            Some(members),
            Vec::new(),
            Vec::new(),
            index_infos,
        );
        result
    }

    pub fn check_decorator(&mut self, node: &Arc<Node>) {
        self.check_grammar_decorator(node);
        let signature = self.get_resolved_signature(node);
        if let Some(sig) = &signature {
            self.check_deprecated_signature(sig, node);
        }
        let return_type = signature
            .as_ref()
            .and_then(|sig| self.get_return_type_of_signature(sig))
            .unwrap_or_else(|| self.error_type());
        if return_type.flags.contains(TypeFlags::Any) {
            return;
        }
        let decorator_signature = match self.get_decorator_call_signature(node) {
            Some(ds) => ds,
            None => return,
        };
        if decorator_signature.resolved_return_type.get().is_none() {
            return;
        }
        let head_message: &'static Message = match node.parent().unwrap().kind {
            SyntaxKind::ClassDeclaration | SyntaxKind::ClassExpression => {
                &DECORATOR_FUNCTION_RETURN_TYPE_0_IS_NOT_ASSIGNABLE_TO_TYPE_1
            }
            SyntaxKind::PropertyDeclaration if !self.legacy_decorators => {
                &DECORATOR_FUNCTION_RETURN_TYPE_0_IS_NOT_ASSIGNABLE_TO_TYPE_1
            }
            SyntaxKind::PropertyDeclaration | SyntaxKind::Parameter => {
                &DECORATOR_FUNCTION_RETURN_TYPE_IS_0_BUT_IS_EXPECTED_TO_BE_VOID_OR_ANY
            }
            SyntaxKind::MethodDeclaration | SyntaxKind::GetAccessor | SyntaxKind::SetAccessor => {
                &DECORATOR_FUNCTION_RETURN_TYPE_0_IS_NOT_ASSIGNABLE_TO_TYPE_1
            }
            _ => panic!("Unhandled case in checkDecorator"),
        };
        let expected_return_type = decorator_signature.resolved_return_type.get().cloned().unwrap();
        self.check_type_assignable_to(
            &return_type,
            &expected_return_type,
            node.expression(),
            Some(head_message),
        );
    }

    pub fn check_deferred_nodes(&mut self, context: &tsox_frontend::ast::SourceFile) {
        let deferred =
            crate::checker::mig::m1b::r25k1_defs::take_source_file_deferred_nodes(context);
        for node in deferred {
            if self.is_canceled() {
                break;
            }
            self.check_deferred_node(&node);
        }
    }

    pub fn check_deferred_node(&mut self, node: &Arc<Node>) {
        let save_current_node = self.current_node.clone();
        self.current_node = Some(Arc::clone(node));
        self.instantiation_count = 0;
        match node.kind {
            SyntaxKind::CallExpression
            | SyntaxKind::NewExpression
            | SyntaxKind::TaggedTemplateExpression
            | SyntaxKind::Decorator
            | SyntaxKind::JsxOpeningElement => {
                self.resolve_untyped_call(node);
            }
            SyntaxKind::FunctionExpression
            | SyntaxKind::ArrowFunction
            | SyntaxKind::MethodDeclaration
            | SyntaxKind::MethodSignature => {
                self.check_function_expression_or_object_literal_method_deferred(node);
            }
            SyntaxKind::GetAccessor | SyntaxKind::SetAccessor => {
                self.check_accessor_declaration(node);
            }
            SyntaxKind::ClassExpression => {
                self.check_class_expression_deferred(node);
            }
            SyntaxKind::TypeParameter => {
                self.check_type_parameter_deferred(node);
            }
            SyntaxKind::JsxSelfClosingElement => {
                self.check_jsx_self_closing_element_deferred(node);
            }
            SyntaxKind::JsxElement => {
                self.check_jsx_element_deferred(node);
            }
            SyntaxKind::TypeAssertionExpression | SyntaxKind::AsExpression => {
                self.check_assertion_deferred(node);
            }
            SyntaxKind::VoidExpression => {
                self.check_expression(&node.expression().unwrap());
            }
            SyntaxKind::BinaryExpression => {
                if ast::is_instance_of_expression(node) {
                    self.resolve_untyped_call(node);
                }
            }
            SyntaxKind::ObjectLiteralExpression | SyntaxKind::JsxAttributes => {
                self.check_contextual_deprecations(node);
            }
            _ => {}
        }
        self.current_node = save_current_node;
    }
}

fn core_OrElse<T>(v: Option<T>, f: impl FnOnce() -> T) -> T {
    v.unwrap_or_else(f)
}

impl Checker {
    pub fn check_do_statement(&mut self, node: &Arc<Node>) {
        self.check_grammar_statement_in_ambient_context(node);
        let NodeData::DoStatement(data) = &node.data else {
            return;
        };
        self.check_source_element(&data.statement);
        self.check_truthiness_expression(&data.expression, CheckMode::Normal);
    }

    pub fn check_delete_expression(&mut self, node: &Arc<Node>) -> Arc<Type> {
        self.check_expression(&node.expression().unwrap());
        let expr = ast::skip_parentheses(&node.expression().unwrap());
        if !ast::is_access_expression(&expr) {
            self.error_message(&expr,THE_OPERAND_OF_A_DELETE_OPERATOR_MUST_BE_A_PROPERTY_REFERENCE, &[]);
            return self.boolean_type();
        }
        if ast::is_property_access_expression(&expr)
            && expr.name().is_some_and(|n| ast::is_private_identifier(n))
        {
            self.error_message(&expr,THE_OPERAND_OF_A_DELETE_OPERATOR_CANNOT_BE_A_PRIVATE_IDENTIFIER, &[]);
        }
        let Some(resolved_symbol) = self.get_resolved_symbol_or_nil(&expr) else {
            return self.boolean_type();
        };
        let symbol = self.get_export_symbol_of_value_symbol_if_exported(&resolved_symbol);
        if self.is_readonly_symbol(&symbol) {
            self.error_message(&expr,THE_OPERAND_OF_A_DELETE_OPERATOR_CANNOT_BE_A_READ_ONLY_PROPERTY, &[]);
        } else {
            self.check_delete_expression_must_be_optional(&expr, &symbol);
        }
        self.boolean_type()
    }

    pub fn check_delete_expression_must_be_optional(&mut self, expr: &Arc<Node>, symbol: &Arc<Symbol>) {
        let t = self.get_type_of_symbol(symbol);
        if self.strict_null_checks
            && !t.flags.intersects(TYPE_FLAGS_ANY_OR_UNKNOWN | TypeFlags::Never)
        {
            let is_optional = if self.exact_optional_property_types {
                symbol.flags.intersects(tsox_frontend::ast::SymbolFlags::Optional)
            } else {
                self.has_type_facts(&t, TypeFacts::IS_UNDEFINED)
            };
            if !is_optional {
                self.error_message(expr,THE_OPERAND_OF_A_DELETE_OPERATOR_MUST_BE_OPTIONAL, &[]);
            }
        }
    }

    pub fn get_async_from_sync_iteration_types(
        &mut self,
        iteration_types: IterationTypes,
        error_node: Option<&Arc<Node>>,
    ) -> IterationTypes {
        let any_type = self.any_type();
        let all_any = iteration_types.yield_type.as_ref().is_some_and(|y| self.types_identical(y, &any_type))
            && iteration_types.return_type.as_ref().is_some_and(|r| self.types_identical(r, &any_type))
            && iteration_types.next_type.as_ref().is_some_and(|n| self.types_identical(n, &any_type));
        if !iteration_types.has_types() || all_any
        {
            return iteration_types;
        }
        if error_node.is_some() {
            self.get_global_awaited_symbol();
        }
        IterationTypes {
            yield_type: iteration_types
                .yield_type
                .as_ref()
                .and_then(|y| self.get_awaited_type_ex(y, error_node, None, &[])),
            return_type: iteration_types
                .return_type
                .as_ref()
                .and_then(|r| self.get_awaited_type_ex(r, error_node, None, &[])),
            next_type: iteration_types.next_type,
        }
    }

    pub fn get_cannot_find_name_diagnostic_for_name(&mut self, node: &Arc<Node>) -> &'static Message {
        match node.text() {
            "document" | "console" => &CANNOT_FIND_NAME_0_DO_YOU_NEED_TO_CHANGE_YOUR_TARGET_LIBRARY_TRY_CHANGING_THE_LIB_COMPILER_OPTION_TO_INCLUDE_DOM,
            "$" => {
                if self.compiler_options.uses_wildcard_types() {
                    &CANNOT_FIND_NAME_0_DO_YOU_NEED_TO_INSTALL_TYPE_DEFINITIONS_FOR_JQUERY_TRY_NPM_I_SAVE_DEV_TYPES_SLASHJQUERY
                } else {
                    &CANNOT_FIND_NAME_0_DO_YOU_NEED_TO_INSTALL_TYPE_DEFINITIONS_FOR_JQUERY_TRY_NPM_I_SAVE_DEV_TYPES_SLASHJQUERY_AND_THEN_ADD_JQUERY_TO_THE_TYPES_FIELD_IN_YOUR_TSCONFIG
                }
            }
            "beforeEach" | "describe" | "suite" | "it" | "test" => {
                if self.compiler_options.uses_wildcard_types() {
                    &CANNOT_FIND_NAME_0_DO_YOU_NEED_TO_INSTALL_TYPE_DEFINITIONS_FOR_A_TEST_RUNNER_TRY_NPM_I_SAVE_DEV_TYPES_SLASHJEST_OR_NPM_I_SAVE_DEV_TYPES_SLASHMOCHA
                } else {
                    &CANNOT_FIND_NAME_0_DO_YOU_NEED_TO_INSTALL_TYPE_DEFINITIONS_FOR_A_TEST_RUNNER_TRY_NPM_I_SAVE_DEV_TYPES_SLASHJEST_OR_NPM_I_SAVE_DEV_TYPES_SLASHMOCHA_AND_THEN_ADD_JEST_OR_MOCHA_TO_THE_TYPES_FIELD_IN_YOUR_TSCONFIG
                }
            }
            "process" | "require" | "Buffer" | "module" | "NodeJS" => {
                if self.compiler_options.uses_wildcard_types() {
                    &CANNOT_FIND_NAME_0_DO_YOU_NEED_TO_INSTALL_TYPE_DEFINITIONS_FOR_NODE_TRY_NPM_I_SAVE_DEV_TYPES_SLASHNODE
                } else {
                    &CANNOT_FIND_NAME_0_DO_YOU_NEED_TO_INSTALL_TYPE_DEFINITIONS_FOR_NODE_TRY_NPM_I_SAVE_DEV_TYPES_SLASHNODE_AND_THEN_ADD_NODE_TO_THE_TYPES_FIELD_IN_YOUR_TSCONFIG
                }
            }
            "Bun" => {
                if self.compiler_options.uses_wildcard_types() {
                    &CANNOT_FIND_NAME_0_DO_YOU_NEED_TO_INSTALL_TYPE_DEFINITIONS_FOR_BUN_TRY_NPM_I_SAVE_DEV_TYPES_SLASHBUN
                } else {
                    &CANNOT_FIND_NAME_0_DO_YOU_NEED_TO_INSTALL_TYPE_DEFINITIONS_FOR_BUN_TRY_NPM_I_SAVE_DEV_TYPES_SLASHBUN_AND_THEN_ADD_BUN_TO_THE_TYPES_FIELD_IN_YOUR_TSCONFIG
                }
            }
            "Map" | "Set" | "Promise" | "ast.Symbol" | "WeakMap" | "WeakSet" | "Iterator"
            | "AsyncIterator" | "SharedArrayBuffer" | "Atomics" | "AsyncIterable"
            | "AsyncIterableIterator" | "AsyncGenerator" | "AsyncGeneratorFunction" | "BigInt"
            | "Reflect" | "BigInt64Array" | "BigUint64Array" => {
                &CANNOT_FIND_NAME_0_DO_YOU_NEED_TO_CHANGE_YOUR_TARGET_LIBRARY_TRY_CHANGING_THE_LIB_COMPILER_OPTION_TO_1_OR_LATER
            }
            "await" if node.parent().is_some_and(|p| ast::is_call_expression(&p)) => {
                &CANNOT_FIND_NAME_0_DID_YOU_MEAN_TO_WRITE_THIS_IN_AN_ASYNC_FUNCTION
            }
            _ => {
                if node.parent().is_some_and(|p| p.kind == SyntaxKind::ShorthandPropertyAssignment) {
                    &NO_VALUE_EXISTS_IN_SCOPE_FOR_THE_SHORTHAND_PROPERTY_0_EITHER_DECLARE_ONE_OR_PROVIDE_AN_INITIALIZER
                } else {
                    &CANNOT_FIND_NAME_0
                }
            }
        }
    }

    pub fn find_resolution_cycle_start_index(
        &self,
        target: TypeSystemEntity,
        property_name: TypeSystemPropertyName,
    ) -> i32 {
        for i in (self.resolution_start..self.type_resolutions.len()).rev() {
            let resolution = &self.type_resolutions[i];
            if type_resolution_has_property_shared(self, resolution) {
                return -1;
            }
            if resolution.target == target && resolution.property_name == property_name {
                return i as i32;
            }
        }
        -1
    }

    pub fn get_array_member_call_signatures(&mut self, t: &Arc<Type>) -> Vec<Arc<Signature>> {
        let mut member_name: Option<String> = None;
        for (i, t) in t.types().unwrap_or(&[]).iter().enumerate() {
            let symbol = t.symbol();
            if !t.object_flags().contains(ObjectFlags::Instantiated)
                || symbol.is_none()
                || !self.is_array_or_tuple_symbol(symbol.as_ref().unwrap().parent().as_ref())
            {
                return Vec::new();
            }
            let symbol = symbol.unwrap();
            if i == 0 {
                member_name = Some(symbol.name.clone());
            } else if member_name.as_deref() != Some(&symbol.name) {
                return Vec::new();
            }
        }
        let Some(member_name) = member_name else { return Vec::new() };
        let mut readonly_by_id: std::collections::HashMap<_, bool> = std::collections::HashMap::new();
        for elem in t.types().unwrap_or(&[]) {
            let is_ro = elem
                .symbol()
                .and_then(|s| s.parent())
                .is_some_and(|p| self.is_readonly_array_symbol(Some(&p)));
            readonly_by_id.insert(elem.id(), is_ro);
        }
        let global_array = self.global_array_type();
        let global_readonly = self.global_readonly_array_type();
        let readonly_symbol_id = global_readonly
            .symbol()
            .map(|s| s.id());
        let array_arg = self.map_type(t, &mut |elem: &Arc<Type>| {
            let is_readonly = readonly_by_id.get(&elem.id()).copied().unwrap_or(false);
            let array_type = if is_readonly { &global_readonly } else { &global_array };
            let type_parameter = array_type.as_interface_type().unwrap().type_parameters()[0].clone();
            elem.mapper().map(|m| m.map(&type_parameter))
        });
        let Some(array_arg) = array_arg else { return Vec::new() };
        let readonly = some_type(t, &|t: &Arc<Type>| {
            t.symbol()
                .and_then(|s| s.parent())
                .is_some_and(|p| readonly_symbol_id.is_some_and(|id| p.id() == id))
        });
        let array_type = self.create_array_type_ex(array_arg, readonly);
        let prop_type = self.get_type_of_property_of_type(&array_type, &member_name);
        let prop_type = prop_type.unwrap_or_else(|| self.unknown_type());
        self.get_signatures_of_type(&prop_type, SignatureKind::Call)
    }


    pub fn get_conditional_type_instantiation(
        &mut self,
        t: &Arc<Type>,
        mapper: Option<&TypeMapper>,
        for_constraint: bool,
        alias: Option<&TypeAlias>,
    ) -> Arc<Type> {
        let root_key = t
            .as_conditional_type()
            .unwrap()
            .root
            .as_deref()
            .map(|root| root as *const ConditionalRoot as usize)
            .unwrap_or(0);
        let Some(root) = t.as_conditional_type().unwrap().root.as_deref() else {
            return Arc::clone(t);
        };
        if !root.outer_type_parameters.is_empty() {
            let type_arguments: Vec<Arc<Type>> = root
                .outer_type_parameters
                .iter()
                .map(|p| mapper.unwrap().map(p))
                .collect();
            let alias_boxed: Option<Box<TypeAlias>> = alias.map(|a| Box::new(a.clone()));
            let key = get_conditional_type_key(&type_arguments, &alias_boxed, for_constraint);
            if let Some(result) = r25k3_defs::conditional_type_instantiations_get(root_key, key) {
                return result;
            }
            let new_mapper = Arc::new(new_type_mapper(
                root.outer_type_parameters.clone(),
                type_arguments.clone(),
            ));
            let root_for_call: Arc<ConditionalRoot> = Arc::new(root.clone());
            let alias_for_call: Option<Arc<TypeAlias>> = alias.map(|a| Arc::new(a.clone()));
            let check_type = root
                .check_type
                .clone()
                .unwrap_or_else(|| self.unknown_type());
            let mut distribution_type: Option<Arc<Type>> = None;
            if root.is_distributive {
                distribution_type = Some(self.get_reduced_type(&new_mapper.map(&check_type)));
            }
            let result = if let Some(distribution_type) = distribution_type {
                if !self.types_identical(&check_type, &distribution_type)
                    && distribution_type
                        .flags
                        .intersects(TypeFlags::Union | TypeFlags::Never)
                {
                    let checker_ptr: *mut Checker = self;
                    let root_for_closure = Arc::clone(&root_for_call);
                    let check_type_for_closure = Arc::clone(&check_type);
                    let mapper_for_closure = Arc::clone(&new_mapper);
                    unsafe {
                        (*checker_ptr).map_type_with_alias(
                            &distribution_type,
                            &mut |t: &Arc<Type>| {
                                Some((*checker_ptr).get_conditional_type(
                                    Arc::clone(&root_for_closure),
                                    Some(Arc::new(prepend_type_mapping(
                                        Arc::clone(&check_type_for_closure),
                                        Arc::clone(t),
                                        Some(mapper_for_closure.as_ref()),
                                    ))),
                                    for_constraint,
                                    None,
                                ))
                            },
                            alias,
                        )
                    }
                    .unwrap_or_else(|| Arc::clone(t))
                } else {
                    self.get_conditional_type(
                        Arc::clone(&root_for_call),
                        Some(Arc::clone(&new_mapper)),
                        for_constraint,
                        alias_for_call.clone(),
                    )
                }
            } else {
                self.get_conditional_type(
                    Arc::clone(&root_for_call),
                    Some(Arc::clone(&new_mapper)),
                    for_constraint,
                    alias_for_call.clone(),
                )
            };
            r25k3_defs::conditional_type_instantiations_insert(root_key, key, Arc::clone(&result));
            return result;
        }
        Arc::clone(t)
    }

    pub fn get_array_or_tuple_target_type(&mut self, node: &Arc<Node>) -> Arc<Type> {
        let containing_parent = node.parent().unwrap();
        let readonly = self.is_readonly_type_operator(&containing_parent);
        let element_type = Checker::get_array_element_type_node(node);
        if let Some(_element_type) = element_type {
            if readonly {
                return self.global_readonly_array_type();
            }
            return self.global_array_type();
        }
        let element_infos: Vec<TupleElementInfo> = node
            .elements()
            .map(|l| l.nodes.as_slice())
            .unwrap_or(&[])
            .iter()
            .map(|e| self.get_tuple_element_info(e))
            .collect();
        self.get_tuple_target_type(&element_infos, readonly)
    }

    pub fn get_big_int_literal_type(&mut self, value: tsox_core::jsnum::PseudoBigInt) -> Arc<Type> {
        let key = value.to_string();
        if let Some(t) = self.bigint_literal_types.get(&key) {
            return Arc::clone(t);
        }
        let t = self.new_literal_type(TypeFlags::BigIntLiteral, LiteralValue::BigInt(value), None);
        self.bigint_literal_types.insert(key, Arc::clone(&t));
        t
    }

    pub fn error_if_writing_to_readonly_index(
        &mut self,
        index_info: Option<&IndexInfo>,
        object_type: &Arc<Type>,
        access_expression: Option<&Arc<Node>>,
    ) {
        if let (Some(index_info), Some(access_expression)) = (index_info, access_expression) {
            if index_info.is_readonly
                && (is_assignment_target(access_expression) || is_delete_target(access_expression))
            {
                let type_string = self.type_to_string(object_type);
                self.error_message(
                    access_expression,INDEX_SIGNATURE_IN_TYPE_0_ONLY_PERMITS_READING,
                    &[type_string],
                );
            }
        }
    }

    pub fn contains_undefined_type(&mut self, t: &Arc<Type>) -> bool {
        let mut t = Arc::clone(t);
        if t.flags.contains(TypeFlags::Union) {
            t = t.types().unwrap_or(&[])[0].clone();
        }
        t.flags.contains(TypeFlags::Undefined)
    }

    pub fn could_access_optional_property(&mut self, object_type: &Arc<Type>, index_type: &Arc<Type>) -> bool {
        let index_constraint = self.get_base_constraint_of_type(index_type);
        if let Some(index_constraint) = index_constraint {
            let properties = self.get_properties_of_type(object_type).to_vec();
            return properties.iter().any(|p| {
                p.flags.intersects(tsox_frontend::ast::SymbolFlags::Optional)
                    && {
                        let literal_type = self.get_literal_type_from_property(p);
                        self.is_type_assignable_to(&literal_type, &index_constraint)
                    }
            });
        }
        false
    }

    pub fn distribute_object_over_index_type(
        &mut self,
        object_type: &Arc<Type>,
        index_type: &Arc<Type>,
        writing: bool,
    ) -> Option<Arc<Type>> {
        if index_type.flags.contains(TypeFlags::Union) {
            let types: Vec<Arc<Type>> = index_type
                .types()
                .unwrap_or(&[])
                .iter()
                .map(|t| {
                    let access = self.get_indexed_access_type(object_type, t);
                    self.get_simplified_type(&access, writing)
                })
                .collect();
            if writing {
                return Some(self.get_intersection_type(types));
            }
            return Some(self.get_union_type(types));
        }
        None
    }

    pub fn distribute_index_over_object_type(
        &mut self,
        object_type: &Arc<Type>,
        index_type: &Arc<Type>,
        writing: bool,
    ) -> Option<Arc<Type>> {
        if object_type.flags.contains(TypeFlags::Union)
            || (object_type.flags.contains(TypeFlags::Intersection)
                && !self.should_defer_index_type(object_type, IndexFlags::None))
        {
            let types: Vec<Arc<Type>> = object_type
                .types()
                .unwrap_or(&[])
                .iter()
                .map(|t| {
                    let access = self.get_indexed_access_type(t, index_type);
                    self.get_simplified_type(&access, writing)
                })
                .collect();
            if object_type.flags.contains(TypeFlags::Intersection) || writing {
                return Some(self.get_intersection_type(types));
            }
            return Some(self.get_union_type(types));
        }
        None
    }

    pub fn extract_types_of_kind(&mut self, t: &Arc<Type>, kind: TypeFlags) -> Arc<Type> {
        self.filter_type(t, &mut |t: &Arc<Type>| t.flags.intersects(kind))
    }

    pub fn extract_definitely_falsy_types(&mut self, t: &Arc<Type>) -> Arc<Type> {
        let checker_ptr: *mut Checker = self;
        unsafe {
            (*checker_ptr)
                .map_type(t, &mut |t: &Arc<Type>| {
                    Some((*checker_ptr).get_definitely_falsy_part_of_type(t))
                })
                .unwrap_or_else(|| Arc::clone(t))
        }
    }

    pub fn get_combined_mapped_type_optionality(&mut self, t: &Arc<Type>) -> i32 {
        if t.object_flags().contains(ObjectFlags::Mapped) {
            let optionality = get_mapped_type_optionality(t);
            if optionality != 0 {
                return optionality;
            }
            let modifiers_type = self.get_modifiers_type_from_mapped_type(t);
            return self.get_combined_mapped_type_optionality(&modifiers_type);
        }
        if t.flags.contains(TypeFlags::Intersection) {
            let types = t.types().unwrap_or(&[]).to_vec();
            let Some(first) = types.first() else { return 0 };
            let optionality = self.get_combined_mapped_type_optionality(first);
            for t in &types[1..] {
                if self.get_combined_mapped_type_optionality(t) != optionality {
                    return 0;
                }
            }
            return optionality;
        }
        0
    }

    pub fn get_constraint_declaration(&self, t: &Arc<Type>) -> Option<Arc<Node>> {
        if let Some(symbol) = t.symbol() {
            for d in &symbol.declarations {
                if ast::is_type_parameter_declaration(d) {
                    if let Some(constraint) = d.as_type_parameter_declaration().constraint.clone() {
                        return Some(constraint);
                    }
                }
            }
        }
        None
    }

    pub fn get_contextual_iteration_type(
        &mut self,
        kind: IterationTypeKind,
        function_decl: Option<&Arc<Node>>,
    ) -> Option<Arc<Type>> {
        let function_decl = function_decl?;
        let is_async = get_function_flags(Some(function_decl))
            .contains(tsox_frontend::ast::mig::m3e::FunctionFlags::ASYNC);
        let contextual_return_type = self.get_contextual_return_type(function_decl, ContextFlags::None);
        if let Some(contextual_return_type) = contextual_return_type {
            return self.get_iteration_type_of_generator_function_return_type(
                kind,
                &contextual_return_type,
                is_async,
            );
        }
        None
    }

    pub fn create_synthetic_expression(
        &mut self,
        parent: &Arc<Node>,
        t: &Arc<Type>,
        is_spread: bool,
        tuple_name_source: Option<&Arc<Node>>,
    ) -> Arc<Node> {
        let mut result = self.factory.new_synthetic_expression(t, is_spread, tuple_name_source);
        self.type_node_links.get_or_default(&result).resolved_type = Some(Arc::clone(t));
        if let Some(result) = Arc::get_mut(&mut result) {
            result.loc = parent.loc;
        }
        result.set_parent(parent);
        result
    }

    pub fn get_class_member_decorator_context_override_type(
        &mut self,
        name_type: &Arc<Type>,
        is_private: bool,
        is_static: bool,
    ) -> Arc<Type> {
        let kind = if is_private {
            if is_static {
                CachedTypeKind::DecoratorContextPrivateStatic
            } else {
                CachedTypeKind::DecoratorContextPrivate
            }
        } else if is_static {
            CachedTypeKind::DecoratorContextStatic
        } else {
            CachedTypeKind::DecoratorContext
        };
        let key = CachedTypeKey { kind, type_id: name_type.id() };
        if let Some(override_type) = self.cached_types.get(&key) {
            return Arc::clone(override_type);
        }
        let mut members = tsox_frontend::ast::SymbolTable::new();
        members.insert("name".to_string(), self.new_property("name", name_type));
        members.insert(
            "private".to_string(),
            self.new_property(
                "private",
                &if is_private { self.true_type() } else { self.false_type() },
            ),
        );
        members.insert(
            "static".to_string(),
            self.new_property(
                "static",
                &if is_static { self.true_type() } else { self.false_type() },
            ),
        );
        let override_type = self.new_object_type(ObjectFlags::Anonymous, None);
        self.set_structured_type_members(&override_type, Some(members), Vec::new(), Vec::new(), Vec::new());
        self.cached_types.insert(key, Arc::clone(&override_type));
        override_type
    }

    pub fn get_class_element_property_key_type(&mut self, element: &Arc<Node>) -> Arc<Type> {
        let name = element.name().unwrap();
        match name.kind {
            SyntaxKind::Identifier | SyntaxKind::NumericLiteral | SyntaxKind::StringLiteral => {
                self.get_string_literal_type(name.text())
            }
            SyntaxKind::ComputedPropertyName => {
                let name_type = self.check_computed_property_name_type(name);
                if self.is_type_assignable_to_kind(&name_type, TYPE_FLAGS_ES_SYMBOL_LIKE) {
                    return name_type;
                }
                self.string_type()
            }
            _ => self.error_type(),
        }
    }

    pub fn find_contextual_node(&self, node: &Arc<Node>, include_caches: bool) -> i32 {
        for (i, info) in self.contextual_infos.iter().enumerate() {
            if info
                .node
                .as_ref()
                .is_some_and(|n| Arc::ptr_eq(node, n))
                && (include_caches || !info.is_cache)
            {
                return i as i32;
            }
        }
        -1
    }

    pub fn get_awaited_type_ex(
        &mut self,
        t: &Arc<Type>,
        error_node: Option<&Arc<Node>>,
        diagnostic_message: Option<&'static Message>,
        _args: &[&str],
    ) -> Option<Arc<Type>> {
        let awaited_type = self.get_awaited_type_no_alias_ex(t, error_node, diagnostic_message, &[]);
        awaited_type.map(|a| self.create_awaited_type_if_needed(&a))
    }

    pub fn create_awaited_type_if_needed(&mut self, t: &Arc<Type>) -> Arc<Type> {
        if self.is_awaited_type_needed(t) {
            if let Some(awaited_type) = self.try_create_awaited_type(t) {
                return awaited_type;
            }
        }
        Arc::clone(t)
    }

    pub fn get_awaited_type_of_promise(&mut self, t: &Arc<Type>) -> Option<Arc<Type>> {
        self.get_awaited_type_of_promise_ex(t, None, None, &[])
    }

    pub fn get_awaited_type_of_promise_ex(
        &mut self,
        t: &Arc<Type>,
        error_node: Option<&Arc<Node>>,
        diagnostic_message: Option<&'static Message>,
        args: &[&str],
    ) -> Option<Arc<Type>> {
        let promised_type = self.get_promised_type_of_promise_ex(t, error_node, None);
        if let Some(promised_type) = promised_type {
            return self.get_awaited_type_ex(&promised_type, error_node, diagnostic_message, args);
        }
        None
    }

    pub fn get_applicable_index_infos(&mut self, t: &Arc<Type>, key_type: &Arc<Type>) -> Vec<Arc<IndexInfo>> {
        self.get_index_infos_of_type(t)
            .into_iter()
            .filter(|info| self.is_applicable_index_type(key_type, info.key_type.as_ref().unwrap()))
            .collect()
    }

    pub fn get_applicable_index_symbol(&mut self, t: &Arc<Type>, key_type: &Arc<Type>) -> Option<Arc<Symbol>> {
        let mut info = self.get_applicable_index_info(t, key_type)?;
        if self.index_info_is_any_base_type(&info) {
            return None;
        }
        if info.index_symbol.is_none() {
            let mut declarations: Vec<Arc<Node>> = Vec::new();
            if let Some(declaration) = info.declaration.clone() {
                declarations.push(declaration);
            } else {
                for info in self.get_index_infos_of_type(t) {
                    if let Some(declaration) = info.declaration.clone() {
                        if self.is_applicable_index_type(key_type, info.key_type.as_ref().unwrap()) {
                            declarations.push(declaration);
                        }
                    }
                }
            }
            if !declarations.is_empty() {
                let mut symbol = self.new_symbol(
                    tsox_frontend::ast::SymbolFlags::Property,
                    tsox_frontend::ast::INTERNAL_SYMBOL_NAME_INDEX,
                );
                if let Some(symbol_mut) = Arc::get_mut(&mut symbol) {
                    symbol_mut.check_flags |= tsox_frontend::ast::CheckFlags::IndexSymbol;
                    symbol_mut.declarations = declarations.clone();
                    symbol_mut.value_declaration = Some(declarations[0].clone());
                }
                if let Some(parent) = t.symbol() {
                    symbol.set_parent(parent);
                }
                let links = self.value_symbol_links.get_or_default(&symbol);
                links.resolved_type = info.value_type.clone();
                if let Some(info_mut) = Arc::get_mut(&mut info) {
                    info_mut.index_symbol = Some(symbol.clone());
                }
            }
        }
        info.index_symbol.clone()
    }
}

pub fn every_type(t: &Arc<Type>, f: &dyn Fn(&Arc<Type>) -> bool) -> bool {
    if t.flags.intersects(TYPE_FLAGS_UNION) {
        match &t.data {
            TypeData::Union(u) => u.union_or_intersection.types.iter().all(|t| f(t)),
            _ => f(t),
        }
    } else {
        f(t)
    }
}

pub fn walk_up_parenthesized_expressions(node: &Arc<Node>) -> Arc<Node> {
    let mut current = Arc::clone(node);
    while let Some(parent) = current.parent() {
        if ast::is_parenthesized_expression(&parent) {
            current = parent;
        } else {
            break;
        }
    }
    current
}

pub fn class_or_constructor_parameter_is_decorated(
    use_legacy_decorators: bool,
    node: &Arc<Node>,
) -> bool {
    if node_is_decorated(use_legacy_decorators, node, None, None) {
        return true;
    }
    if let Some(constructor) = get_first_constructor_with_body(node) {
        if let Some(parameters) = node_parameters(&constructor) {
            if parameters
                .nodes
                .iter()
                .any(|p| node_is_decorated(use_legacy_decorators, p, Some(&constructor), Some(node)))
            {
                return true;
            }
        }
    }
    false
}

pub fn is_tuple_type(t: &Arc<Type>) -> bool {
    is_tuple_type_ref(t)
}

pub fn is_type_any(t: &Arc<Type>) -> bool {
    is_type_any_ref(t)
}

pub fn is_object_literal_type(t: &Arc<Type>) -> bool {
    is_object_literal_type_ref(t)
}

pub fn is_array_or_tuple_type(t: &Arc<Type>) -> bool {
    is_array_or_tuple_type_ref(t)
}

pub fn is_literal_type(t: &Arc<Type>) -> bool {
    is_literal_type_ref(t)
}

pub fn is_initialized_property(member: &Arc<Node>) -> bool {
    is_property_declaration(member) && initializer(member).is_some()
}

use std::sync::OnceLock;

impl Checker {
    pub fn undefined_or_missing_type(&self) -> Arc<Type> {
        self.undefined_type()
    }

    pub fn implicit_never_type(&self) -> Arc<Type> {
        self.never_type()
    }

    pub fn wildcard_type(&self) -> Arc<Type> {
        self.any_type()
    }

    pub fn silent_never_type(&self) -> Arc<Type> {
        self.silent_never_type.get_or_init(|| self.never_type()).clone()
    }

    pub fn undefined_widening_type(&mut self) -> Arc<Type> {
        self.create_widening_type(&self.undefined_type())
    }

    pub fn string_number_symbol_type(&mut self) -> Arc<Type> {
        self.get_union_type(vec![self.string_type(), self.number_type(), self.es_symbol_type()])
    }

    pub fn no_constraint_type(&mut self) -> Arc<Type> {
        if self.no_constraint_type.get().is_none() {
            let t = self.new_object_type(ObjectFlags::Anonymous, None);
            self.set_structured_type_members(&t, None, Vec::new(), Vec::new(), Vec::new());
            let _ = self.no_constraint_type.set(t);
        }
        self.no_constraint_type.get().unwrap().clone()
    }

    pub fn empty_generic_type(&mut self) -> Arc<Type> {
        if self.empty_generic_type.get().is_none() {
            let t = self.new_object_type(ObjectFlags::Anonymous, None);
            self.set_structured_type_members(&t, None, Vec::new(), Vec::new(), Vec::new());
            let _ = self.empty_generic_type.set(t);
        }
        self.empty_generic_type.get().unwrap().clone()
    }

    pub fn any_array_type(&mut self) -> Arc<Type> {
        if let Some(t) = self.any_array_type.get() {
            return Arc::clone(t);
        }
        let any = self.any_type();
        let arr = self.create_array_type(any);
        match self.any_array_type.set(Arc::clone(&arr)) {
            Ok(()) => arr,
            Err(_) => self.any_array_type.get().cloned().unwrap_or(arr),
        }
    }

    pub fn global_array_type(&mut self) -> Arc<Type> {
        if self.global_array_type.get().is_none() {
            let t = self.get_global_type("Array", 1, true);
            let _ = self.global_array_type.set(t);
        }
        self.global_array_type.get().unwrap().clone()
    }

    pub fn global_readonly_array_type(&mut self) -> Arc<Type> {
        if self.global_readonly_array_type.get().is_none() {
            let t = self.get_global_type("ReadonlyArray", 1, false);
            let _ = self.global_readonly_array_type.set(t);
        }
        self.global_readonly_array_type.get().unwrap().clone()
    }

    pub fn get_global_awaited_symbol(&mut self) -> Option<Arc<Symbol>> {
        self.get_global_symbol("Awaited", tsox_frontend::ast::SymbolFlags::Property, None)
    }

    pub fn types_identical(&mut self, a: &Arc<Type>, b: &Arc<Type>) -> bool {
        self.is_type_identical_to(a, b)
    }

    pub fn types_identical_to_silent_never(&mut self, a: &Arc<Type>, b: &Arc<Type>) -> bool {
        self.is_type_identical_to(a, b)
    }

    pub fn get_non_nullable_type(&mut self, t: &Arc<Type>) -> Arc<Type> {
        self.get_non_nullable_type_of(t)
    }

    pub fn is_type_assignable_to_kind(&mut self, source: &Arc<Type>, kind: TypeFlags) -> bool {
        self.is_type_assignable_to_kind_ex(source, kind, false)
    }
}
