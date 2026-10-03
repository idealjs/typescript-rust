#![allow(unused_imports)]

use crate::checker::checker::*;
use std::sync::Arc;
use tsox_frontend::ast::NodeData;
use tsox_frontend::ast::NodeFlags;
use tsox_frontend::ast::SyntaxKind;
use tsox_frontend::ast::{is_class_expression, is_computed_property_name, is_function_expression, skip_parentheses, skip_outer_expression_all};
use crate::checker::mig::m1c_3::create_diagnostic_for_node_message;
use tsox_frontend::scanner::mig::m3i::declaration_name_to_string;
use crate::checker::utilities_get_assignment_target::entity_name_to_string;
use crate::checker::utilities_get_assignment_target::get_binding_element_property_name;
use crate::checker::exports_union_reduction::get_declaration_modifier_flags_from_symbol_ex;
use crate::checker::utilities_get_assignment_target::is_this_initialized_declaration;
use crate::checker::utilities_is_private_within_ambient::is_this_initialized_object_binding_expression;
use crate::checker::utilities_has_only_expression_initialization::is_this_property;
use super::m1b::jsnum_from_string;
use super::m1b::parse_pseudo_big_int;
use crate::checker::mig::m2c_3::some_type;
use super::m1e::r26k3_defs::some_type_with_checker;
use tsox_frontend::scanner::token_to_string;
use tsox_core::diagnostics::messages_generated::*;
use crate::checker::utilities_is_private_within_ambient::is_class_instance_property;
use super::m1b::{PredicateSemantics, has_decorators, is_accessibility_modifier_set};
use tsox_frontend::ast::mig::x4ast::get_class_like_declaration_of_symbol;
use tsox_frontend::ast::mig::m3g_2::is_part_of_type_query;
use crate::checker::checker_this_container::get_this_container;

pub type NonNullReporter = fn(&mut Checker, &Arc<Node>, TypeFacts);

use tsox_core::diagnostics::messages_generated::{
    THE_VALUE_0_CANNOT_BE_USED_HERE as THE_VALUE_0CANNOT_BE_USED_HERE,
    X_0_IS_POSSIBLY_UNDEFINED as X_0IS_POSSIBLY_UNDEFINED,
    X_0_AND_1_OPERATIONS_CANNOT_BE_MIXED_WITHOUT_PARENTHESES
        as X_0AND_1OPERATIONS_CANNOT_BE_MIXED_WITHOUT_PARENTHESES,
    THE_0_OPERATOR_CANNOT_BE_APPLIED_TO_TYPE_SYMBOL as THE_0OPERATOR_CANNOT_BE_APPLIED_TO_TYPE_SYMBOL,
    OPERATOR_0_CANNOT_BE_APPLIED_TO_TYPE_1 as OPERATOR_0CANNOT_BE_APPLIED_TO_TYPE_1,
    ABSTRACT_METHOD_0_IN_CLASS_1_CANNOT_BE_ACCESSED_VIA_SUPER_EXPRESSION
        as ABSTRACT_METHOD_0IN_CLASS_1CANNOT_BE_ACCESSED_VIA_SUPER_EXPRESSION,
    CLASS_FIELD_0_DEFINED_BY_THE_PARENT_CLASS_IS_NOT_ACCESSIBLE_IN_THE_CHILD_CLASS_VIA_SUPER
        as CLASS_FIELD_0DEFINED_BY_THE_PARENT_CLASS_IS_NOT_ACCESSIBLE_IN_THE_CHILD_CLASS_VIA_SUPER,
    ABSTRACT_PROPERTY_0_IN_CLASS_1_CANNOT_BE_ACCESSED_IN_THE_CONSTRUCTOR
        as ABSTRACT_PROPERTY_0IN_CLASS_1CANNOT_BE_ACCESSED_IN_THE_CONSTRUCTOR,
    PROPERTY_0_IS_PRIVATE_AND_ONLY_ACCESSIBLE_WITHIN_CLASS_1
        as PROPERTY_0IS_PRIVATE_AND_ONLY_ACCESSIBLE_WITHIN_CLASS_1,
    PROPERTY_0_IS_PROTECTED_AND_ONLY_ACCESSIBLE_WITHIN_CLASS_1_AND_ITS_SUBCLASSES
        as PROPERTY_0IS_PROTECTED_AND_ONLY_ACCESSIBLE_WITHIN_CLASS_1AND_ITS_SUBCLASSES,
    TYPE_0_DOES_NOT_SATISFY_THE_EXPECTED_TYPE_1 as TYPE_0DOES_NOT_SATISFY_THE_EXPECTED_TYPE_1,
};

impl Checker {
    pub fn check_non_null_assertion(&mut self, node: &Arc<Node>) -> Arc<Type> { ::tsox_core::fntrace::enter("check_non_null_assertion"); 
        if node.flags.contains(NodeFlags::OptionalChain) {
            return self.check_non_null_chain(node);
        }
        let expression = node.expression().cloned().unwrap_or_else(|| Arc::clone(node));
        let t = self.check_expression_ex(&expression, CheckMode::Normal);
        self.get_non_nullable_type(&t)
    }

    pub fn check_non_null_chain(&mut self, node: &Arc<Node>) -> Arc<Type> { ::tsox_core::fntrace::enter("check_non_null_chain"); 
        let expression = node.expression().cloned().unwrap_or_else(|| Arc::clone(node));
        let left_type = self.check_expression_ex(&expression, CheckMode::Normal);
        let non_optional_type = self.get_optional_expression_type(&left_type, &expression);
        let non_nullable = self.get_non_nullable_type(&non_optional_type);
        self.propagate_optional_type_marker(&non_nullable, node, !Arc::ptr_eq(&non_optional_type, &left_type))
    }

    pub fn check_non_null_expression(&mut self, node: &Arc<Node>) -> Arc<Type> { ::tsox_core::fntrace::enter("check_non_null_expression"); 
        let t = self.check_expression_ex(node, CheckMode::Normal);
        self.check_non_null_type(&t, node)
    }

    pub fn check_non_null_type_with_reporter(&mut self, t: &Arc<Type>, node: &Arc<Node>, report_error: NonNullReporter) -> Arc<Type> { ::tsox_core::fntrace::enter("check_non_null_type_with_reporter"); 
        if self.strict_null_checks && t.flags.contains(TypeFlags::Unknown) {
            if tsox_frontend::ast::is_entity_name_expression(node) {
                let node_text = entity_name_to_string(node);
                if node_text.len() < 100 {
                    self.error_message(node,X_0_IS_OF_TYPE_UNKNOWN, &[node_text]);
                    return self.error_type();
                }
            }
            self.error_message(node,OBJECT_IS_OF_TYPE_UNKNOWN, &[]);
            return self.error_type();
        }
        let facts = self.get_type_facts(t, TypeFacts::IS_UNDEFINED_OR_NULL);
        if facts.intersects(TypeFacts::IS_UNDEFINED_OR_NULL) {
            report_error(self, node, facts);
            let non_nullable = self.get_non_nullable_type(t);
            if non_nullable.flags.intersects(TypeFlags::NULLABLE | TypeFlags::Never) {
                return self.error_type();
            }
            return non_nullable;
        }
        Arc::clone(t)
    }

    pub fn check_non_null_non_void_type(&mut self, t: &Arc<Type>, node: &Arc<Node>) -> Arc<Type> { ::tsox_core::fntrace::enter("check_non_null_non_void_type"); 
        let non_null_type = self.check_non_null_type(t, node);
        if non_null_type.flags.contains(TypeFlags::Void) {
            if tsox_frontend::ast::is_entity_name_expression(node) {
                let node_text = entity_name_to_string(node);
                if tsox_frontend::ast::is_identifier(node) && node_text == "undefined" {
                    self.error_message(node, THE_VALUE_0CANNOT_BE_USED_HERE, &[node_text]);
                    return non_null_type;
                }
                if node_text.len() < 100 {
                    self.error_message(node, X_0IS_POSSIBLY_UNDEFINED, &[node_text]);
                    return non_null_type;
                }
            }
            self.error_message(node,OBJECT_IS_POSSIBLY_UNDEFINED, &[]);
        }
        non_null_type
    }

    pub fn check_nullish_coalesce_operand_left(&mut self, left: &Arc<Node>) { ::tsox_core::fntrace::enter("check_nullish_coalesce_operand_left"); 
        let left_target = skip_outer_expression_all(left);
        let nullish_semantics = self.get_syntactic_nullishness_semantics(&left_target);
        if nullish_semantics != PredicateSemantics::Sometimes {
            if nullish_semantics == PredicateSemantics::Always {
                self.error_message(&left_target,THIS_EXPRESSION_IS_ALWAYS_NULLISH, &[]);
            } else {
                self.error_message(&left_target,RIGHT_OPERAND_OF_IS_UNREACHABLE_BECAUSE_THE_LEFT_OPERAND_IS_NEVER_NULLISH, &[]);
            }
        }
    }

    pub fn check_nullish_coalesce_operands(&mut self, left: &Arc<Node>, right: &Arc<Node>) { ::tsox_core::fntrace::enter("check_nullish_coalesce_operands"); 
        let parent = left.parent();
        let grandparent = parent.and_then(|p| p.parent());
        let grandparent_is_binary = grandparent.as_ref().map(|g| tsox_frontend::ast::is_binary_expression(g)).unwrap_or(false);
        if grandparent_is_binary {
            let grandparent = grandparent.unwrap();
            let (grandparent_left, grandparent_operator_kind) = match &grandparent.data {
                NodeData::BinaryExpression(data) => (Arc::clone(&data.left), data.operator_token.kind),
                _ => (Arc::clone(&grandparent), SyntaxKind::Unknown),
            };
            if tsox_frontend::ast::is_binary_expression(&grandparent_left) && grandparent_operator_kind == SyntaxKind::BarBarToken {
                let first = token_to_string(SyntaxKind::QuestionQuestionToken).to_string();
                let second = token_to_string(grandparent_operator_kind).to_string();
                self.grammar_error_on_node_with_args(&grandparent_left, &X_0AND_1OPERATIONS_CANNOT_BE_MIXED_WITHOUT_PARENTHESES, &[first, second]);
            }
        } else if tsox_frontend::ast::is_binary_expression(left) {
            let operator_kind = match &left.data {
                NodeData::BinaryExpression(data) => data.operator_token.kind,
                _ => SyntaxKind::Unknown,
            };
            if operator_kind == SyntaxKind::BarBarToken || operator_kind == SyntaxKind::AmpersandAmpersandToken {
                let first = token_to_string(operator_kind).to_string();
                let second = token_to_string(SyntaxKind::QuestionQuestionToken).to_string();
                self.grammar_error_on_node_with_args(left, &X_0AND_1OPERATIONS_CANNOT_BE_MIXED_WITHOUT_PARENTHESES, &[first, second]);
            }
        } else if tsox_frontend::ast::is_binary_expression(right) {
            let operator_kind = match &right.data {
                NodeData::BinaryExpression(data) => data.operator_token.kind,
                _ => SyntaxKind::Unknown,
            };
            if operator_kind == SyntaxKind::AmpersandAmpersandToken {
                let first = token_to_string(SyntaxKind::QuestionQuestionToken).to_string();
                let second = token_to_string(operator_kind).to_string();
                self.grammar_error_on_node_with_args(right, &X_0AND_1OPERATIONS_CANNOT_BE_MIXED_WITHOUT_PARENTHESES, &[first, second]);
            }
        }
        self.check_nullish_coalesce_operand_left(left);
    }

    pub fn check_object_literal_method(&mut self, node: &Arc<Node>, check_mode: CheckMode) -> Arc<Type> { ::tsox_core::fntrace::enter("check_object_literal_method"); 
        self.check_grammar_method(node);
        let name = node.name();
        if name.as_ref().map(|n| is_computed_property_name(n)).unwrap_or(false) {
            let name = name.unwrap();
            self.check_computed_property_name(&name);
        }
        let uninstantiated_type = self.check_function_expression_or_object_literal_method(node, check_mode);
        self.instantiate_type_with_single_generic_call_signature(node, &uninstantiated_type, check_mode)
    }

    pub fn check_parenthesized_expression(&mut self, node: &Arc<Node>, check_mode: CheckMode) -> Arc<Type> { ::tsox_core::fntrace::enter("check_parenthesized_expression"); 
        let expression = node.expression().cloned().unwrap_or_else(|| Arc::clone(node));
        self.check_expression_ex(&expression, check_mode)
    }

    pub fn check_postfix_unary_expression(&mut self, node: &Arc<Node>) -> Arc<Type> { ::tsox_core::fntrace::enter("check_postfix_unary_expression"); 
        let (operand, _operator) = match &node.data {
            NodeData::PostfixUnaryExpression(data) => (Arc::clone(&data.operand), data.operator),
            _ => (Arc::clone(node), SyntaxKind::Unknown),
        };
        let operand_type = self.check_expression_ex(&operand, CheckMode::Normal);
        if Arc::ptr_eq(&operand_type, &self.silent_never_type()) {
            return self.silent_never_type();
        }
        let non_null = self.check_non_null_type(&operand_type, &operand);
        let ok = self.check_arithmetic_operand_type(&operand, &non_null, AN_ARITHMETIC_OPERAND_MUST_BE_OF_TYPE_ANY_NUMBER_BIGINT_OR_AN_ENUM_TYPE, false);
        if ok {
            self.check_reference_expression(
                &operand,
                THE_OPERAND_OF_AN_INCREMENT_OR_DECREMENT_OPERATOR_MUST_BE_A_VARIABLE_OR_A_PROPERTY_ACCESS,
                THE_OPERAND_OF_AN_INCREMENT_OR_DECREMENT_OPERATOR_MAY_NOT_BE_AN_OPTIONAL_PROPERTY_ACCESS,
            );
        }
        self.get_unary_result_type(&operand_type)
    }

    pub fn check_prefix_unary_expression(&mut self, node: &Arc<Node>) -> Arc<Type> { ::tsox_core::fntrace::enter("check_prefix_unary_expression"); 
        let (operator, operand) = match &node.data {
            NodeData::PrefixUnaryExpression(data) => (data.operator, Arc::clone(&data.operand)),
            _ => (SyntaxKind::Unknown, Arc::clone(node)),
        };
        let operand_type = self.check_expression_ex(&operand, CheckMode::Normal);
        if Arc::ptr_eq(&operand_type, &self.silent_never_type()) {
            return self.silent_never_type();
        }
        match operand.kind {
            SyntaxKind::NumericLiteral => match operator {
                SyntaxKind::MinusToken => {
                    let value = jsnum_negate(jsnum_from_string(operand.text()));
                    let literal = self.get_number_literal_type(value);
                    return self.get_fresh_type_of_literal_type(&literal);
                }
                SyntaxKind::PlusToken => {
                    let value = jsnum_from_string(operand.text());
                    let literal = self.get_number_literal_type(value);
                    return self.get_fresh_type_of_literal_type(&literal);
                }
                _ => {}
            },
            SyntaxKind::BigIntLiteral => {
                if operator == SyntaxKind::MinusToken {
                    let positive = parse_pseudo_big_int(&operand.text());
                    let negative = tsox_core::jsnum::PseudoBigInt::new(&positive.to_string(), true);
                    let literal = self.get_big_int_literal_type(negative);
                    return self.get_fresh_type_of_literal_type(&literal);
                }
            }
            _ => {}
        }
        match operator {
            SyntaxKind::PlusToken | SyntaxKind::MinusToken | SyntaxKind::TildeToken => {
                self.check_non_null_type(&operand_type, &operand);
                if self.maybe_type_of_kind_considering_base_constraint(&operand_type, TypeFlags::ES_SYMBOL_LIKE) {
                    let operator_string = token_to_string(operator);
                    self.error_message(&operand, THE_0OPERATOR_CANNOT_BE_APPLIED_TO_TYPE_SYMBOL, &[operator_string.to_string()]);
                }
                if operator == SyntaxKind::PlusToken {
                    if self.maybe_type_of_kind_considering_base_constraint(&operand_type, TYPE_FLAGS_BIG_INT_LIKE) {
                        let operator_string = token_to_string(operator);
                        let base = self.get_base_type_of_literal_type(&operand_type);
                        let base_string = self.type_to_string(&base);
                        self.error_message(&operand, OPERATOR_0CANNOT_BE_APPLIED_TO_TYPE_1, &[operator_string.to_string(), base_string]);
                    }
                    return self.number_type();
                }
                self.get_unary_result_type(&operand_type)
            }
            SyntaxKind::ExclamationToken => {
                self.check_truthiness_of_type(&operand);
                let facts = self.get_type_facts(&operand_type, TypeFacts::TRUTHY | TypeFacts::FALSY);
                if facts == TypeFacts::TRUTHY {
                    return self.false_type();
                } else if facts == TypeFacts::FALSY {
                    return self.true_type();
                }
                self.boolean_type()
            }
            SyntaxKind::PlusPlusToken | SyntaxKind::MinusMinusToken => {
                let non_null = self.check_non_null_type(&operand_type, &operand);
                let ok = self.check_arithmetic_operand_type(&operand, &non_null, AN_ARITHMETIC_OPERAND_MUST_BE_OF_TYPE_ANY_NUMBER_BIGINT_OR_AN_ENUM_TYPE, false);
                if ok {
                    self.check_reference_expression(
                        &operand,
                        THE_OPERAND_OF_AN_INCREMENT_OR_DECREMENT_OPERATOR_MUST_BE_A_VARIABLE_OR_A_PROPERTY_ACCESS,
                        THE_OPERAND_OF_AN_INCREMENT_OR_DECREMENT_OPERATOR_MAY_NOT_BE_AN_OPTIONAL_PROPERTY_ACCESS,
                    );
                }
                self.get_unary_result_type(&operand_type)
            }
            _ => self.error_type(),
        }
    }

    pub fn check_private_identifier_expression(&mut self, node: &Arc<Node>) { ::tsox_core::fntrace::enter("check_private_identifier_expression"); 
        self.check_grammar_private_identifier_expression(node);
        let symbol = self.get_symbol_for_private_identifier_expression(node);
        if let Some(symbol) = symbol {
            self.mark_property_as_referenced(&symbol, None);
        }
    }

    pub fn check_property_accessibility(&mut self, node: &Arc<Node>, is_super: bool, writing: bool, t: &Arc<Type>, prop: &Arc<Symbol>) -> bool { ::tsox_core::fntrace::enter("check_property_accessibility"); 
        self.check_property_accessibility_ex(node, is_super, writing, t, prop, true)
    }

    pub fn check_property_accessibility_ex(&mut self, node: &Arc<Node>, is_super: bool, writing: bool, t: &Arc<Type>, prop: &Arc<Symbol>, report_error: bool) -> bool { ::tsox_core::fntrace::enter("check_property_accessibility_ex"); 
        let mut error_node: Option<Arc<Node>> = None;
        if report_error {
            error_node = match node.kind {
                SyntaxKind::PropertyAccessExpression => node.name().cloned(),
                SyntaxKind::QualifiedName => match &node.data {
                    NodeData::QualifiedName(data) => Some(Arc::clone(&data.right)),
                    _ => Some(Arc::clone(node)),
                },
                SyntaxKind::ImportType => Some(Arc::clone(node)),
                SyntaxKind::BindingElement => get_binding_element_property_name(node),
                _ => node.name().cloned(),
            };
        }
        self.check_property_accessibility_at_location(node, is_super, writing, t, prop, error_node.as_ref())
    }

    pub fn check_property_accessibility_at_location(&mut self, location: &Arc<Node>, is_super: bool, writing: bool, containing_type: &Arc<Type>, prop: &Arc<Symbol>, error_node: Option<&Arc<Node>>) -> bool { ::tsox_core::fntrace::enter("check_property_accessibility_at_location"); 
        let flags = get_declaration_modifier_flags_from_symbol_ex(prop, writing);
        if is_super {
            if flags.contains(ModifierFlags::Abstract) {
                if let Some(error_node) = error_node {
                    let prop_name = self.symbol_to_string(prop);
                    let class_string = match self.get_declaring_class(prop) {
                        Some(declaring_class) => self.type_to_string(&declaring_class),
                        None => String::new(),
                    };
                    self.error_message(error_node, ABSTRACT_METHOD_0IN_CLASS_1CANNOT_BE_ACCESSED_VIA_SUPER_EXPRESSION, &[prop_name, class_string]);
                }
                return false;
            }
            if !flags.contains(ModifierFlags::Static) && prop.declarations.iter().any(|d| is_class_instance_property(d)) {
                if let Some(error_node) = error_node {
                    let prop_name = self.symbol_to_string(prop);
                    self.error_message(error_node, CLASS_FIELD_0DEFINED_BY_THE_PARENT_CLASS_IS_NOT_ACCESSIBLE_IN_THE_CHILD_CLASS_VIA_SUPER, &[prop_name]);
                }
                return false;
            }
        }
        if flags.contains(ModifierFlags::Abstract)
            && self.symbol_has_non_method_declaration(prop)
            && (is_this_property(location)
                || is_this_initialized_object_binding_expression(location)
                || location
                    .parent()
                    .map(|parent| {
                        tsox_frontend::ast::is_object_binding_pattern(&parent)
                            && parent
                                .parent()
                                .map(|grandparent| is_this_initialized_declaration(&grandparent))
                                .unwrap_or(false)
                    })
                    .unwrap_or(false))
        {
            let parent_symbol = self.get_parent_of_symbol(prop);
            if let Some(parent_symbol) = parent_symbol {
                if parent_symbol.flags.contains(SymbolFlags::Class) && self.is_node_used_during_class_initialization(location) {
                    if let Some(error_node) = error_node {
                        let prop_name = self.symbol_to_string(prop);
                        let parent_name = self.symbol_to_string(&parent_symbol);
                        self.error_message(error_node, ABSTRACT_PROPERTY_0IN_CLASS_1CANNOT_BE_ACCESSED_IN_THE_CONSTRUCTOR, &[prop_name, parent_name]);
                    }
                    return false;
                }
            }
        }
        if !flags.contains(ModifierFlags::NonPublicAccessibilityModifier) {
            return true;
        }
        if flags.contains(ModifierFlags::Private) {
            let mut declaring_class_declaration: Option<Arc<Node>> = None;
            if let Some(parent) = self.get_parent_of_symbol(prop) {
                declaring_class_declaration = get_class_like_declaration_of_symbol(&parent);
            }
            let within = declaring_class_declaration
                .as_ref()
                .map(|decl| self.is_node_within_class(location, decl))
                .unwrap_or(false);
            if declaring_class_declaration.is_none() || !within {
                if let Some(error_node) = error_node {
                    let prop_name = self.symbol_to_string(prop);
                    let class_string = match self.get_declaring_class(prop) {
                        Some(class) => self.type_to_string(&class),
                        None => String::new(),
                    };
                    self.error_message(error_node, PROPERTY_0IS_PRIVATE_AND_ONLY_ACCESSIBLE_WITHIN_CLASS_1, &[prop_name, class_string]);
                }
                return false;
            }
            return true;
        }
        if is_super {
            return true;
        }
        let mut enclosing_class: Option<Arc<Type>> = None;
        let mut container = tsox_frontend::ast::get_containing_class(location);
        while let Some(container_node) = &container {
            let symbol = self.get_symbol_of_declaration(container_node);
            if let Some(symbol) = &symbol {
                let class = self.get_declared_type_of_symbol(symbol);
                if self.is_class_derived_from_declaring_classes(&class, prop, writing) {
                    enclosing_class = Some(class);
                    break;
                }
            }
            container = tsox_frontend::ast::get_containing_class(container_node);
        }
        if enclosing_class.is_none() {
            let class = self.get_enclosing_class_from_this_parameter(location);
            if let Some(class) = &class {
                if self.is_class_derived_from_declaring_classes(class, prop, writing) {
                    enclosing_class = Some(Arc::clone(class));
                }
            }
            if flags.contains(ModifierFlags::Static) || enclosing_class.is_none() {
                if let Some(error_node) = error_node {
                    let prop_name = self.symbol_to_string(prop);
                    let class_string = match enclosing_class
                        .clone()
                        .or_else(|| self.get_declaring_class(prop))
                    {
                        Some(class) => self.type_to_string(&class),
                        None => String::new(),
                    };
                    self.error_message(error_node, PROPERTY_0IS_PROTECTED_AND_ONLY_ACCESSIBLE_WITHIN_CLASS_1AND_ITS_SUBCLASSES, &[prop_name, class_string]);
                }
                return false;
            }
        }
        if !writing && flags.contains(ModifierFlags::Static) {
            let location_parent = location.parent();
            let in_static_initializer_or_accessor = location_parent
                .as_ref()
                .map(|parent| {
                    (tsox_frontend::ast::is_property_declaration(parent) || tsox_frontend::ast::is_class_static_block_declaration(parent))
                        && parent
                            .parent()
                            .map(|grandparent| grandparent.kind == SyntaxKind::Constructor)
                            .unwrap_or(false)
                })
                .unwrap_or(false)
                || location_parent
                    .map(|parent| {
                        is_accessibility_modifier_set(get_declaration_modifier_flags_from_symbol_ex(prop, writing))
                            && false
                    })
                    .unwrap_or(false);
            let _ = in_static_initializer_or_accessor;
        }
        true
    }

    pub fn check_qualified_name(&mut self, node: &Arc<Node>, check_mode: CheckMode) -> Arc<Type> { ::tsox_core::fntrace::enter("check_qualified_name"); 
        let (left, right) = match &node.data {
            NodeData::QualifiedName(data) => (Arc::clone(&data.left), Arc::clone(&data.right)),
            _ => (Arc::clone(node), Arc::clone(node)),
        };
        let left_type;
        if is_part_of_type_query(node) && tsox_frontend::ast::is_this_identifier(Some(left.as_ref())) {
            let t = self.check_this_expression(&left);
            left_type = self.check_non_null_type(&t, &left);
        } else {
            left_type = self.check_non_null_expression(&left);
        }
        self.check_property_access_expression_or_qualified_name(node, &left, &left_type, &right, check_mode, false)
    }


    pub fn check_regular_expression_literal(&mut self, node: &Arc<Node>) -> Arc<Type> { ::tsox_core::fntrace::enter("check_regular_expression_literal"); 
        if !self.node_links.get_or_default(node).flags.contains(NodeCheckFlags::TypeChecked) {
            self.node_links.get_or_default(node).flags |= NodeCheckFlags::TypeChecked;
            self.check_grammar_regular_expression_literal(node);
        }
        self.global_regexp_type()
    }

    pub fn check_satisfies_expression(&mut self, node: &Arc<Node>) -> Arc<Type> { ::tsox_core::fntrace::enter("check_satisfies_expression"); 
        let type_node = match &node.data {
            NodeData::SatisfiesExpression(d) => Arc::clone(&d.type_node),
            _ => node.type_node().cloned().unwrap_or_else(|| Arc::clone(node)),
        };
        self.check_source_element(&type_node);
        let expression = node.expression().cloned().unwrap_or_else(|| Arc::clone(node));
        let expr_type = self.check_expression_ex(&expression, CheckMode::Normal);
        let target_type = self.get_type_from_type_node(&type_node);
        if self.is_error_type(&target_type) {
            return target_type;
        }
        self.check_type_assignable_to_and_optionally_elaborate(
            &expr_type,
            &target_type,
            Some(node),
            Some(&expression),
            Some(&TYPE_0DOES_NOT_SATISFY_THE_EXPECTED_TYPE_1),
            None,
        );
        expr_type
    }

    pub fn check_shorthand_property_assignment(&mut self, node: &Arc<Node>, in_destructuring_pattern: bool, check_mode: CheckMode) -> Arc<Type> { ::tsox_core::fntrace::enter("check_shorthand_property_assignment"); 
        let expr = if !in_destructuring_pattern {
            match &node.data {
                NodeData::ShorthandPropertyAssignment(data) => data.object_assignment_initializer.clone(),
                _ => None,
            }
        } else {
            None
        };
        let expr = expr.unwrap_or_else(|| node.name().cloned().unwrap_or_else(|| Arc::clone(node)));
        let expression_type = self.check_expression_for_mutable_location(&expr, check_mode);
        if let Some(type_node) = node.type_node() {
            let t = self.get_type_from_type_node(type_node);
            self.check_type_assignable_to_and_optionally_elaborate(&expression_type, &t, Some(node), Some(&expr), None, None);
            return t;
        }
        expression_type
    }

    pub fn check_synthetic_expression(&mut self, node: &Arc<Node>) -> Arc<Type> { ::tsox_core::fntrace::enter("check_synthetic_expression"); 
        let is_spread = match &node.data {
            NodeData::SyntheticExpression(data) => data.is_spread,
            _ => false,
        };
        let t = self.get_type_of_node(node);
        if is_spread {
            let number_type = self.number_type();
            return self.get_indexed_access_type(&t, &number_type);
        }
        t
    }

    pub fn check_tagged_template_expression(&mut self, node: &Arc<Node>) -> Arc<Type> { ::tsox_core::fntrace::enter("check_tagged_template_expression"); 
        if !self.check_grammar_tagged_template_chain(node) {
            if let Some(type_arguments) = tsox_frontend::ast::mig::m3c::type_argument_list(node) {
                self.check_grammar_type_arguments(node, type_arguments);
            }
        }
        let signature = self.get_resolved_signature(node);
        if let Some(sig) = &signature {
            self.check_deprecated_signature(sig, node);
        }
        signature
            .as_ref()
            .and_then(|sig| self.get_return_type_of_signature(sig))
            .unwrap_or_else(|| self.unknown_type())
    }

    pub fn check_template_expression(&mut self, node: &Arc<Node>) -> Arc<Type> { ::tsox_core::fntrace::enter("check_template_expression"); 
        let (head, template_spans) = match &node.data {
            NodeData::TemplateExpression(data) => (Arc::clone(&data.head), Arc::clone(&data.template_spans)),
            _ => return Arc::clone(&self.string_type()),
        };
        let length = template_spans.len();
        let mut texts: Vec<String> = Vec::with_capacity(length + 1);
        let mut types: Vec<Arc<Type>> = Vec::with_capacity(length);
        texts.push(head.text().to_string());
        for span in template_spans.iter() {
            let expression = span.expression().cloned().unwrap_or_else(|| Arc::clone(span));
            let t = self.check_expression_ex(&expression, CheckMode::Normal);
            if self.maybe_type_of_kind_considering_base_constraint(&t, TypeFlags::ES_SYMBOL_LIKE) {
                self.error_message(&expression,IMPLICIT_CONVERSION_OF_A_SYMBOL_TO_A_STRING_WILL_FAIL_AT_RUNTIME_CONSIDER_WRAPPING_THIS_EXPRESSION_IN_STRING, &[]);
            }
            let span_literal = match &span.data {
                NodeData::TemplateSpan(d) => Arc::clone(&d.literal),
                _ => Arc::clone(span),
            };
            texts.push(span_literal.text().to_string());
            let constraint = self.template_constraint_type();
            if self.is_type_assignable_to(&t, &constraint) {
                types.push(t);
            } else {
                types.push(Arc::clone(&self.string_type()));
            }
        }
        let mut evaluated: Option<String> = None;
        let parent = node.parent();
        if !parent.as_ref().map(|p| tsox_frontend::ast::is_tagged_template_expression(p)).unwrap_or(false) {
            let mut entity_fn =
                |expr: &Arc<Node>, loc: Option<&Arc<Node>>| self.evaluate_entity(expr, loc);
            let result = tsox_frontend::evaluator::evaluate_expression(node, Some(node), &mut entity_fn);
            evaluated = result.value.and_then(|v| match v {
                tsox_frontend::evaluator::EvalValue::String(s) => Some(s),
                _ => None,
            });
        }
        if let Some(evaluated) = evaluated {
            let literal = self.get_string_literal_type(evaluated.as_str());
            return self.get_fresh_type_of_literal_type(&literal);
        }
        let contextual = self.get_contextual_type(node, ContextFlags::empty());
        let is_template_contextual = match &contextual {
            Some(contextual_type) => some_type_with_checker(self, contextual_type, &mut |c: &mut Checker, t: &Arc<Type>| c.is_template_literal_contextual_type(t)),
            None => false,
        };
        if self.is_const_context(node) || self.is_template_literal_context(node) || is_template_contextual {
            return self.get_template_literal_type(&texts, &types);
        }
        Arc::clone(&self.string_type())
    }

    pub fn check_this_expression(&mut self, node: &Arc<Node>) -> Arc<Type> { ::tsox_core::fntrace::enter("check_this_expression"); 
        let mut container = get_this_container(node, true, true);
        let mut captured_by_arrow_function = false;
        let mut this_in_computed_property_name = false;
        if container.kind == SyntaxKind::Constructor {
            self.check_this_before_super(node, &container, X_SUPER_MUST_BE_CALLED_BEFORE_ACCESSING_THIS_IN_THE_CONSTRUCTOR_OF_A_DERIVED_CLASS);
        }
        loop {
            if container.kind == SyntaxKind::ArrowFunction {
                container = get_this_container(&container, false, !this_in_computed_property_name);
                captured_by_arrow_function = true;
            }
            if container.kind == SyntaxKind::ComputedPropertyName {
                container = get_this_container(&container, !captured_by_arrow_function, false);
                this_in_computed_property_name = true;
                continue;
            }
            break;
        }
        self.check_this_in_static_class_field_initializer_in_decorated_class(node, &container);
        if self.this_location_errors_reported.insert(node.id()) {
            if this_in_computed_property_name {
                self.diagnostics.add(tsox_frontend::ast::Diagnostic::new(
                    self.current_file.clone(),
                    node.loc,
                    X_THIS_CANNOT_BE_REFERENCED_IN_A_COMPUTED_PROPERTY_NAME,
                    Vec::new(),
                ));
            } else {
                match container.kind {
                    SyntaxKind::ModuleDeclaration => {
                        self.diagnostics.add(tsox_frontend::ast::Diagnostic::new(
                            self.current_file.clone(),
                            node.loc,
                            X_THIS_CANNOT_BE_REFERENCED_IN_A_MODULE_OR_NAMESPACE_BODY,
                            Vec::new(),
                        ));
                    }
                    SyntaxKind::EnumDeclaration => {
                        self.diagnostics.add(tsox_frontend::ast::Diagnostic::new(
                            self.current_file.clone(),
                            node.loc,
                            X_THIS_CANNOT_BE_REFERENCED_IN_CURRENT_LOCATION,
                            Vec::new(),
                        ));
                    }
                    _ => {}
                }
            }
        }
        let t = self.try_get_this_type_at_ex(node, true, Some(&container));
        if self.no_implicit_this {
            let global_this_type = match self.global_this_symbol.clone() {
                Some(symbol) => self.get_type_of_symbol(&symbol),
                None => self.error_type(),
            };
            let is_global_this = t.as_ref().map(|t| Arc::ptr_eq(t, &global_this_type)).unwrap_or(false);
            if is_global_this && captured_by_arrow_function {
                self.error_message(node,THE_CONTAINING_ARROW_FUNCTION_CAPTURES_THE_GLOBAL_VALUE_OF_THIS, &[]);
            } else if t.is_none() {
                self.error_message(node,X_THIS_IMPLICITLY_HAS_TYPE_ANY_BECAUSE_IT_DOES_NOT_HAVE_A_TYPE_ANNOTATION, &[]);
                if container.kind != SyntaxKind::SourceFile {
                    let outside_this = self.try_get_this_type_at(&container);
                    if let Some(outside_this) = outside_this {
                        if !Arc::ptr_eq(&outside_this, &global_this_type) {
                            let related = create_diagnostic_for_node_message(&container,AN_OUTER_VALUE_OF_THIS_IS_SHADOWED_BY_THIS_CONTAINER, &[]);
                            self.add_related_info(node, (*related).clone());
                        }
                    }
                }
            }
        }
        t.unwrap_or_else(|| self.any_type())
    }

    pub fn check_this_in_static_class_field_initializer_in_decorated_class(&mut self, this_expression: &Arc<Node>, container: &Arc<Node>) { ::tsox_core::fntrace::enter("check_this_in_static_class_field_initializer_in_decorated_class"); 
        if container.kind == SyntaxKind::PropertyDeclaration
            && tsox_frontend::ast::has_static_modifier(container)
            && self.legacy_decorators
        {
            if let Some(initializer) = tsox_frontend::ast::mig::m3b::initializer(container) {
                let parent = container.parent();
                let parent_has_decorators = parent
                    .map(|p| has_decorators(&p))
                    .unwrap_or(false);
                if initializer.loc.contains_inclusive(this_expression.loc.pos as usize) && parent_has_decorators {
                    self.error_message(this_expression,CANNOT_USE_THIS_IN_A_STATIC_PROPERTY_INITIALIZER_OF_A_DECORATED_CLASS, &[]);
                }
            }
        }
    }

    pub fn check_truthiness_expression(&mut self, node: &Arc<Node>, check_mode: CheckMode) -> Arc<Type> { ::tsox_core::fntrace::enter("check_truthiness_expression"); 
        let t = self.check_expression_ex(node, check_mode);
        self.check_truthiness_of_type(node);
        t
    }
}

pub(crate) fn jsnum_negate(value: tsox_core::jsnum::Number) -> tsox_core::jsnum::Number { ::tsox_core::fntrace::enter("jsnum_negate"); 
    tsox_core::jsnum::Number(-value.0)
}
