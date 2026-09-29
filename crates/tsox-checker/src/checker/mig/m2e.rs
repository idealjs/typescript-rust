#![allow(unused_imports)]
#![allow(dead_code)]

use crate::checker::checker::*;
use crate::checker::types::*;
use std::sync::Arc;
use tsox_core::diagnostics::{
    AN_IMPORT_ATTRIBUTES_PROPERTY_CANNOT_BE_OPTIONAL, AN_IMPORT_ATTRIBUTES_PROPERTY_MUST_HAVE_A_STRING_LITERAL_OR_IDENTIFIER_NAME,
    AN_IMPORT_ATTRIBUTES_PROPERTY_MUST_HAVE_A_STRING_LITERAL_TYPE_ANNOTATION, AN_IMPORT_ATTRIBUTES_PROPERTY_MUST_HAVE_A_TYPE_ANNOTATION,
    AN_IMPORT_ATTRIBUTES_TYPE_MAY_ONLY_CONTAIN_PROPERTY_SIGNATURES, X_0_IS_NOT_A_VALID_KEY_FOR_AN_IMPORT_ATTRIBUTES_TYPE,
};
use tsox_frontend::ast::{
    is_binding_element, is_declaration_node, is_element_access_expression, is_entity_name_expression,
    is_enum_member, is_identifier, is_string_literal_like, is_string_literal_like_type, is_string_or_numeric_literal_like,
    Node, NodeData, Symbol, SymbolFlags, SyntaxKind,
};

#[path = "r19k3_defs.rs"]
pub mod r19k3_defs;
pub(crate) use r19k3_defs::{
    any_to_string, get_candidate_variable_declaration_initializer, has_only_expression_initializer,
    non_dotted_name_cache_key, try_get_text_of_property_name, NodeAccessExtR19k3,
};
use super::m2d_3::{FlowState, FlowType};
use super::wc3::NodeAccessExt;

impl Checker {
    pub(crate) fn narrow_type_by_literal_expression(
        &mut self,
        t: &Arc<Type>,
        literal: &Arc<Node>,
        assume_true: bool,
    ) -> Arc<Type> {
        if assume_true {
            return self.narrow_type_by_type_name(t, &literal.text());
        }
        let facts = TypeFacts::from_bits_truncate(Self::typeof_ne_facts_of_witness(&literal.text()));
        self.get_adjusted_type_with_facts(t, facts)
    }

    pub(crate) fn narrow_type_by_private_identifier_in_in_expression(
        &mut self,
        f: &FlowState,
        t: &Arc<Type>,
        expr: &Arc<Node>,
        assume_true: bool,
    ) -> Arc<Type> {
        let right = expr.right();
        let target = self.get_reference_candidate(&right);
        if !self.is_matching_reference(f.reference.as_ref().unwrap(), &target) {
            return Arc::clone(t);
        }
        let symbol = match self.get_symbol_for_private_identifier_expression(&expr.left()) {
            Some(symbol) => symbol,
            None => return Arc::clone(t),
        };
        let class_symbol = symbol.parent().unwrap();
        let target_type = if symbol
            .value_declaration
            .as_ref()
            .is_some_and(|d| tsox_frontend::ast::has_static_modifier(d))
        {
            self.get_type_of_symbol(&class_symbol)
        } else {
            self.get_declared_type_of_symbol(&class_symbol)
        };
        self.get_narrowed_type(t, &target_type, assume_true, true)
    }

    pub(crate) fn narrow_type_by_switch_optional_chain_containment(
        &mut self,
        t: &Arc<Type>,
        data: &Arc<Node>,
        clause_check: &dyn Fn(&Arc<Type>) -> bool,
    ) -> Arc<Type> {
        let clause_start = data.as_flow_switch_clause_data().clause_start;
        let clause_end = data.as_flow_switch_clause_data().clause_end;
        let switch_statement = Arc::clone(&data.as_flow_switch_clause_data().switch_statement);
        let every_clause_checks = clause_start != clause_end
            && self
                .get_switch_clause_types(&switch_statement)[clause_start as usize..clause_end as usize]
                .iter()
                .all(clause_check);
        if every_clause_checks {
            return self.get_type_with_facts(t, TypeFacts::NE_UNDEFINED_OR_NULL);
        }
        Arc::clone(t)
    }

    pub(crate) fn narrow_type_by_typeof(
        &mut self,
        f: &FlowState,
        t: &Arc<Type>,
        type_of_expr: &Arc<Node>,
        operator: SyntaxKind,
        literal: &Arc<Node>,
        assume_true: bool,
    ) -> Arc<Type> {
        let mut assume_true = assume_true;
        if operator == SyntaxKind::ExclamationEqualsToken || operator == SyntaxKind::ExclamationEqualsEqualsToken {
            assume_true = !assume_true;
        }
        let expression = type_of_expr.expression().unwrap();
        let target = self.get_reference_candidate(expression);
        if !self.is_matching_reference(f.reference.as_ref().unwrap(), &target) {
            if self.strict_null_checks
                && self.optional_chain_contains_reference(&target, f.reference.as_ref().unwrap())
                && assume_true == (literal.text() != "undefined")
            {
                let t = self.get_adjusted_type_with_facts(t, TypeFacts::NE_UNDEFINED_OR_NULL);
                if let Some(property_access) = self.get_discriminant_property_access(f, &target, &t) {
                    return self.narrow_type_by_discriminant(
                        &t,
                        &property_access,
                        &|checker, t| checker.narrow_type_by_literal_expression(t, literal, assume_true),
                    );
                }
                return t;
            }
            if let Some(property_access) = self.get_discriminant_property_access(f, &target, t) {
                return self.narrow_type_by_discriminant(
                    t,
                    &property_access,
                    &|checker, t| checker.narrow_type_by_literal_expression(t, literal, assume_true),
                );
            }
            return Arc::clone(t);
        }
        self.narrow_type_by_literal_expression(t, literal, assume_true)
    }

    pub(crate) fn new_flow_type(&self, t: &Arc<Type>, incomplete: bool) -> FlowType {
        let t = if incomplete && t.flags.contains(TypeFlags::NEVER) {
            self.silent_never_type()
        } else {
            Arc::clone(t)
        };
        FlowType { t: Some(t), incomplete }
    }

    pub(crate) fn try_get_element_access_expression_name(&mut self, node: &Arc<Node>) -> Option<String> {
        let argument = node.as_element_access_expression().argument_expression.clone();
        if is_string_or_numeric_literal_like(&argument) {
            return Some(argument.text().to_string());
        }
        if is_entity_name_expression(&argument) {
            return self.try_get_name_from_entity_name_expression(&argument);
        }
        None
    }

    pub(crate) fn try_get_name_from_entity_name_expression(&mut self, node: &Arc<Node>) -> Option<String> {
        let symbol = self.resolve_entity_name(node, SymbolFlags::VALUE, true, false, None)?;
        if !(self.is_constant_variable(&symbol) || symbol.flags.contains(SymbolFlags::EnumMember)) {
            return None;
        }
        let declaration = symbol.value_declaration.as_ref()?;
        let t = self.try_get_type_from_type_node(declaration);
        if let Some(t) = t {
            if let Some(name) = try_get_name_from_type(&t) {
                return Some(name);
            }
        }
        if has_only_expression_initializer(declaration)
            && !is_binding_element(declaration)
            && self.is_block_scoped_name_declared_before_use(declaration, node)
        {
            if let Some(initializer) = declaration.initializer() {
                let initializer_type = self.check_expression_ex(&initializer, CheckMode::Normal);
                return try_get_name_from_type(&initializer_type);
            } else if is_enum_member(declaration) {
                return try_get_text_of_property_name(declaration.name().unwrap());
            }
        }
        None
    }

    pub(crate) fn write_flow_cache_key(
        &mut self,
        b: &mut KeyBuilder,
        node: &Arc<Node>,
        declared_type: &Arc<Type>,
        initial_type: Option<&Arc<Type>>,
        flow_container: Option<&Arc<Node>>,
    ) -> bool {
        match node.kind {
            SyntaxKind::Identifier | SyntaxKind::ThisKeyword => {
                if node.kind == SyntaxKind::Identifier && !tsox_frontend::ast::is_this_in_type_query(node) {
                    let Some(symbol) = self.get_resolved_symbol(node) else {
                        return false;
                    };
                    if self.unknown_symbol.as_ref().is_some_and(|u| Arc::ptr_eq(&symbol, u)) {
                        return false;
                    }
                    b.write_symbol(&symbol);
                }
                b.write_byte(b':');
                b.write_type(declared_type);
                if let Some(initial_type) = initial_type {
                    if !Arc::ptr_eq(initial_type, declared_type) {
                        b.write_byte(b'=');
                        b.write_type(initial_type);
                    }
                }
                if let Some(flow_container) = flow_container {
                    b.write_byte(b'@');
                    b.write_node(Some(flow_container));
                }
                true
            }
            SyntaxKind::NonNullExpression | SyntaxKind::ParenthesizedExpression => {
                self.write_flow_cache_key(b, node.expression().unwrap(), declared_type, initial_type, flow_container)
            }
            SyntaxKind::QualifiedName => {
                let left = Arc::clone(&node.as_qualified_name().left);
                if !self.write_flow_cache_key(b, &left, declared_type, initial_type, flow_container) {
                    return false;
                }
                b.write_byte(b'.');
                b.write_string(&node.as_qualified_name().right.text());
                true
            }
            SyntaxKind::PropertyAccessExpression | SyntaxKind::ElementAccessExpression => {
                if let Some(prop_name) = self.get_accessed_property_name(node) {
                    let expression = node.expression().unwrap();
                    if !self.write_flow_cache_key(b, expression, declared_type, initial_type, flow_container) {
                        return false;
                    }
                    b.write_byte(b'.');
                    b.write_string(&prop_name);
                    return true;
                }
                if is_element_access_expression(node) {
                    let argument = node.as_element_access_expression().argument_expression.clone();
                    if is_identifier(&argument) {
                        let Some(symbol) = self.get_resolved_symbol(&argument) else {
                            return false;
                        };
                        if self.is_constant_variable(&symbol)
                            || Checker::is_parameter_or_mutable_local_variable(self, &symbol)
                                && !self.is_symbol_assigned(&symbol)
                        {
                            let expression = node.expression().unwrap();
                            if !self.write_flow_cache_key(b, expression, declared_type, initial_type, flow_container) {
                                return false;
                            }
                            b.write_string(".@");
                            b.write_symbol(&symbol);
                            return true;
                        }
                    }
                }
                false
            }
            SyntaxKind::ObjectBindingPattern
            | SyntaxKind::ArrayBindingPattern
            | SyntaxKind::FunctionDeclaration
            | SyntaxKind::FunctionExpression
            | SyntaxKind::ArrowFunction
            | SyntaxKind::MethodDeclaration => {
                b.write_node(Some(node));
                b.write_byte(b'#');
                b.write_type(declared_type);
                true
            }
            _ => false,
        }
    }

    pub(crate) fn check_grammar_import_attributes_type(&mut self, attributes: &Arc<Node>) -> bool {
        let members = &attributes.as_type_literal_node().members.nodes;
        if members.is_empty() {
            return false;
        }
        for member in members {
            if member.kind != SyntaxKind::PropertySignature {
                self.grammar_error_on_node(member, &AN_IMPORT_ATTRIBUTES_TYPE_MAY_ONLY_CONTAIN_PROPERTY_SIGNATURES);
                return true;
            }
            let NodeData::PropertySignatureDeclaration(property_signature) = &member.data else {
                return true;
            };
            if member.type_node().is_none() {
                self.grammar_error_on_node(member, &AN_IMPORT_ATTRIBUTES_PROPERTY_MUST_HAVE_A_TYPE_ANNOTATION);
                return true;
            }
            if property_signature.postfix_token.is_some() {
                self.grammar_error_on_node(member, &AN_IMPORT_ATTRIBUTES_PROPERTY_CANNOT_BE_OPTIONAL);
                return true;
            }
            let name = Arc::clone(&property_signature.name);
            if !(is_string_literal_like(&name) || is_identifier(&name)) {
                self.grammar_error_on_node(&name, &AN_IMPORT_ATTRIBUTES_PROPERTY_MUST_HAVE_A_STRING_LITERAL_OR_IDENTIFIER_NAME);
                return true;
            }
            if name.text() == "resolution-mode" {
                self.grammar_error_on_node_with_args(
                    &name,
                    &X_0_IS_NOT_A_VALID_KEY_FOR_AN_IMPORT_ATTRIBUTES_TYPE,
                    &[name.text().to_string()],
                );
                return true;
            }
            let type_node = Arc::clone(&property_signature.type_node);
            if !is_string_literal_like_type(&type_node) {
                self.grammar_error_on_node(&type_node, &AN_IMPORT_ATTRIBUTES_PROPERTY_MUST_HAVE_A_STRING_LITERAL_TYPE_ANNOTATION);
                return true;
            }
        }
        false
    }

    pub(crate) fn check_grammar_top_level_elements_for_required_declare_modifier(
        &mut self,
        file: &Arc<Node>,
    ) -> bool {
        for decl in &file.as_source_file().statements.nodes {
            if is_declaration_node(decl) || decl.kind == SyntaxKind::VariableStatement {
                if self.check_grammar_top_level_element_for_required_declare_modifier(decl) {
                    return true;
                }
            }
        }
        false
    }
}

pub(crate) fn get_identifier_from_entity_name_expression(node: &Arc<Node>) -> Option<Arc<Node>> {
    match node.kind {
        SyntaxKind::Identifier => Some(Arc::clone(node)),
        SyntaxKind::PropertyAccessExpression => Some(Arc::clone(&node.as_property_access_expression().name)),
        _ => None,
    }
}

pub(crate) fn is_initializer_big_int_literal_expression(expr: &Arc<Node>) -> bool {
    if expr.kind == SyntaxKind::BigIntLiteral {
        return true;
    }
    if expr.kind == SyntaxKind::PrefixUnaryExpression {
        let unary_expr = expr.as_prefix_unary_expression();
        return unary_expr.operator == SyntaxKind::MinusToken && unary_expr.operand.kind == SyntaxKind::BigIntLiteral;
    }
    false
}

pub(crate) fn is_initializer_string_or_number_literal_expression(expr: &Arc<Node>) -> bool {
    if is_string_or_numeric_literal_like(expr) {
        return true;
    }
    if expr.kind == SyntaxKind::PrefixUnaryExpression {
        let unary_expr = expr.as_prefix_unary_expression();
        return unary_expr.operator == SyntaxKind::MinusToken
            && unary_expr.operand.kind == SyntaxKind::NumericLiteral;
    }
    false
}

pub(crate) fn try_get_name_from_type(t: &Arc<Type>) -> Option<String> {
    if t.flags.contains(TypeFlags::UniqueESSymbol) {
        return Some(t.as_unique_es_symbol_type()?.name.clone());
    }
    if t.flags.contains(TypeFlags::StringLiteral) || t.flags.contains(TypeFlags::NumberLiteral) {
        return Some(any_to_string(&t.as_literal_type()?.value));
    }
    None
}
