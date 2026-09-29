#![allow(unused_imports)]
use std::sync::Arc;
use tsox_core::diagnostics::messages_generated::*;
use tsox_core::diagnostics::Message;
use tsox_frontend::ast::{self, Node, SyntaxKind};
use tsox_frontend::ast::node_data_generated::{
    is_array_literal_expression, is_assignment_operator, is_binary_expression, is_call_expression,
    is_element_access_expression, is_identifier, is_numeric_literal, is_object_literal_expression,
    is_parenthesized_expression, is_property_access_expression, is_tagged_template_expression,
};
use tsox_frontend::ast::{is_compound_assignment, is_declaration_node};
use tsox_frontend::ast::mig::x4ast::get_invoked_expression;
use tsox_frontend::scanner::token_to_string;
use crate::checker::checker_checker::*;
use crate::checker::types_impl_chunk_3::Signature;
use crate::checker::types_type_id::TYPE_FLAGS_ES_SYMBOL_LIKE;
use crate::checker::utilities_is_private_within_ambient::try_get_property_access_or_identifier_to_string;
use crate::checker::mig::m2a::r19k11_defs::R19K11NodeExt;
use crate::checker::mig::m2e::r19k3_defs::NodeAccessExtR19k3;

impl Checker {
    pub fn node_check_flags(&mut self, node: &Arc<Node>, flags: NodeCheckFlags) {
        if let Some(links) = self.node_links.get_mut(node) {
            links.flags |= flags;
        }
    }

    pub fn signature_is_resolving_signature(&self, sig: &Arc<Signature>) -> bool {
        Arc::ptr_eq(sig, &self.resolving_signature())
    }

    pub fn check_for_disallowed_essymbol_operand(
        &mut self,
        left: &Arc<Node>,
        right: &Arc<Node>,
        left_type: &Arc<Type>,
        right_type: &Arc<Type>,
        operator: SyntaxKind,
    ) -> bool {
        let offending_symbol_operand = if self
            .maybe_type_of_kind_considering_base_constraint(left_type, TYPE_FLAGS_ES_SYMBOL_LIKE)
        {
            Some(left)
        } else if self.maybe_type_of_kind_considering_base_constraint(
            right_type,
            TYPE_FLAGS_ES_SYMBOL_LIKE,
        ) {
            Some(right)
        } else {
            None
        };
        if let Some(offending_symbol_operand) = offending_symbol_operand {
            self.error_message(
                offending_symbol_operand,THE_0_OPERATOR_CANNOT_BE_APPLIED_TO_TYPE_SYMBOL,
                &[token_to_string(operator).to_string()],
            );
            return false;
        }
        true
    }

    pub fn check_assignment_operator(
        &mut self,
        left: &Arc<Node>,
        operator: SyntaxKind,
        right: &Arc<Node>,
        left_type: &Arc<Type>,
        right_type: &Arc<Type>,
    ) {
        if !is_assignment_operator(operator) {
            return;
        }
        let mut left_type = Arc::clone(left_type);
        if is_compound_assignment(operator) && is_property_access_expression(left) {
            left_type = self.check_property_access_expression(left, CheckMode::Normal, true);
        }
        if self.check_reference_expression(
            left,
            THE_LEFT_HAND_SIDE_OF_AN_ASSIGNMENT_EXPRESSION_MUST_BE_A_VARIABLE_OR_A_PROPERTY_ACCESS,
            THE_LEFT_HAND_SIDE_OF_AN_ASSIGNMENT_EXPRESSION_MAY_NOT_BE_AN_OPTIONAL_PROPERTY_ACCESS,
        ) {
            self.check_type_assignable_to_and_optionally_elaborate(
                right_type,
                &left_type,
                Some(left),
                Some(right),
                None,
                None,
            );
        }
    }

    pub fn check_deprecated_signature(&mut self, sig: &Arc<Signature>, node: &Arc<Node>) {
        if sig
            .flags
            .contains(SignatureFlags::IsSignatureCandidateForOverloadFailure)
        {
            return;
        }
        if let Some(declaration) = sig.declaration.clone() {
            if self.is_deprecated_declaration(&declaration) {
                if let Some(suggestion_node) = self.get_deprecated_suggestion_node(node) {
                    let name = try_get_property_access_or_identifier_to_string(
                        &get_invoked_expression(node),
                    );
                    let signature_string = self.signature_to_string(sig);
                    self.add_deprecated_suggestion_with_signature(
                        &suggestion_node,
                        &declaration,
                        &name,
                        &signature_string,
                    );
                }
            }
        }
    }

    pub fn class_declaration_extends_null(&mut self, class_decl: &Arc<Node>) -> bool {
        let Some(class_symbol) = self.get_symbol_of_declaration_opt(class_decl) else {
            return false;
        };
        let class_instance_type = self.get_declared_type_of_symbol(&class_symbol);
        match self.get_base_constructor_type_of_class(&class_instance_type) {
            Some(base_constructor_type) => {
                Arc::ptr_eq(&base_constructor_type, &self.null_widening_type())
            }
            None => false,
        }
    }

    pub fn is_indirect_call(&self, node: &Arc<Node>) -> bool {
        let binary = node.as_binary_expression_node();
        let left = &binary.left;
        let right = &binary.right;
        let Some(parent) = node.parent() else {
            return false;
        };
        let Some(grandparent) = parent.parent() else {
            return false;
        };
        is_parenthesized_expression(&parent)
            && is_numeric_literal(left)
            && left.text() == "0"
            && ((is_call_expression(&grandparent)
                && grandparent
                    .expression()
                    .map(|e| Arc::ptr_eq(e, &parent))
                    .unwrap_or(false))
                || is_tagged_template_expression(&grandparent))
            && (is_element_access_expression(right)
                || (is_identifier(right) && right.text() == "eval"))
    }

    pub fn check_destructuring_assignment_for_binary(
        &mut self,
        node: &Arc<Node>,
        source_type: &Arc<Type>,
        check_mode: CheckMode,
        right_is_this: bool,
    ) -> Arc<Type> {
        let mut source_type = Arc::clone(source_type);
        let mut target = Arc::clone(node);
        if is_binary_expression(&target)
            && target.as_binary_expression_node().operator_token.kind == SyntaxKind::EqualsToken
        {
            self.check_binary_expression(&target);
            target = Arc::clone(&target.as_binary_expression_node().left);
            if self.strict_null_checks {
                source_type = self.get_type_with_facts(&source_type, TypeFacts::NE_UNDEFINED);
            }
        }
        if is_object_literal_expression(&target) {
            self.check_object_literal_assignment(&target, &source_type, right_is_this);
            return Arc::clone(&source_type);
        }
        if is_array_literal_expression(&target) {
            self.check_array_literal_assignment(&target, &source_type, check_mode);
            return Arc::clone(&source_type);
        }
        self.check_reference_assignment(&target, &source_type, check_mode);
        Arc::clone(&source_type)
    }

    pub fn get_essymbol_like_type_for_node(&mut self, node: &Arc<Node>) -> Arc<Type> {
        let _ = node;
        self.es_symbol_type()
    }
}
