#![allow(unused_imports)]

use super::*;

impl Checker {
    pub fn check_grammar_for_invalid_question_mark(
        &mut self,
        _postfix_token: &Arc<Node>,
        _message: &Message,
    ) -> bool {
        false
    }

    pub fn check_grammar_for_invalid_exclamation_token(
        &mut self,
        _postfix_token: &Arc<Node>,
        _message: &Message,
    ) -> bool {
        false
    }

    pub fn check_grammar_object_literal_expression(
        &mut self,
        _node: &Arc<Node>,
        _in_destructuring: bool,
    ) -> bool {
        false
    }

    pub fn check_grammar_for_in_or_for_of_statement(&mut self, _node: &Arc<Node>) -> bool {
        false
    }

    pub fn check_grammar_accessor(&mut self, _node: &Arc<Node>) -> bool {
        false
    }

    pub fn does_accessor_have_correct_parameter_count(&mut self, _accessor: &Arc<Node>) -> bool {
        true
    }

    pub fn check_grammar_type_operator_node(&mut self, _node: &Arc<Node>) -> bool {
        false
    }

    pub fn check_grammar_for_invalid_dynamic_name(
        &mut self,
        _node: &Arc<Node>,
        _message: &Message,
    ) -> bool {
        false
    }

    pub fn is_non_bindable_dynamic_name(&self, _node: &Arc<Node>) -> bool {
        false
    }

    pub fn check_grammar_method(&mut self, _node: &Arc<Node>) -> bool {
        false
    }

    pub fn check_grammar_binding_element(&mut self, _node: &Arc<Node>) -> bool {
        false
    }

    pub fn check_grammar_for_es_module_marker_in_binding_name(
        &mut self,
        _name: &Arc<Node>,
    ) -> bool {
        false
    }

    pub fn check_grammar_name_in_let_or_const_declarations(&mut self, _name: &Arc<Node>) -> bool {
        false
    }

    pub fn check_grammar_await_or_await_using(&mut self, _node: &Arc<Node>) -> bool {
        false
    }

    pub fn check_grammar_yield_expression(&mut self, _node: &Arc<Node>) -> bool {
        false
    }

    pub fn check_grammar_for_disallowed_block_scoped_variable_statement(
        &mut self,
        _node: &Arc<Node>,
    ) -> bool {
        false
    }

    pub fn container_allows_block_scoped_variable(&self, _parent: &Arc<Node>) -> bool {
        false
    }

    pub fn check_grammar_meta_property(&mut self, _node: &Arc<Node>) -> bool {
        false
    }

    pub fn check_grammar_constructor_type_parameters(&mut self, _node: &Arc<Node>) -> bool {
        false
    }

    pub fn check_grammar_constructor_type_annotation(&mut self, _node: &Arc<Node>) -> bool {
        false
    }

    pub fn check_grammar_property(&mut self, _node: &Arc<Node>) -> bool {
        false
    }

    pub fn check_ambient_initializer(&mut self, _node: &Arc<Node>) -> bool {
        false
    }

    pub fn is_initializer_simple_literal_enum_reference(&mut self, _expr: &Arc<Node>) -> bool {
        false
    }

    pub fn check_grammar_top_level_element_for_required_declare_modifier(
        &mut self,
        _node: &Arc<Node>,
    ) -> bool {
        false
    }

    pub fn check_grammar_top_level_elements_for_required_declare_modifier(
        &mut self,
        _file: &Arc<crate::ast::SourceFile>,
    ) -> bool {
        false
    }

    pub fn check_grammar_source_file(&mut self, _node: &Arc<crate::ast::SourceFile>) -> bool {
        false
    }

    pub fn check_grammar_statement_in_ambient_context(&mut self, _node: &Arc<Node>) -> bool {
        false
    }

    pub fn check_grammar_numeric_literal(&mut self, _node: &Arc<Node>) {}

    pub fn check_grammar_big_int_literal(&mut self, _node: &Arc<Node>) -> bool {
        false
    }

    pub fn check_grammar_import_clause(&mut self, _node: &Arc<Node>) -> bool {
        false
    }

    pub fn check_grammar_type_only_named_imports_or_exports(
        &mut self,
        _named_bindings: &Arc<Node>,
    ) -> bool {
        false
    }

    pub fn check_grammar_import_call_expression(&mut self, _node: &Arc<Node>) -> bool {
        false
    }
}

pub(crate) fn is_comma_sequence(node: &Arc<Node>) -> bool {
    if node.kind != SyntaxKind::BinaryExpression {
        return false;
    }
    match &node.data {
        NodeData::BinaryExpression(data) => data.operator_token.kind == SyntaxKind::CommaToken,
        _ => false,
    }
}

pub(crate) fn get_identifier_from_entity_name_expression(node: &Arc<Node>) -> Option<Arc<Node>> {
    match node.kind {
        SyntaxKind::Identifier => Some(Arc::clone(node)),
        SyntaxKind::PropertyAccessExpression => None,
        _ => None,
    }
}

pub(crate) fn is_initializer_string_or_number_literal_expression(expr: &Arc<Node>) -> bool {
    matches!(
        expr.kind,
        SyntaxKind::StringLiteral | SyntaxKind::NumericLiteral
    )
}

pub(crate) fn is_initializer_big_int_literal_expression(expr: &Arc<Node>) -> bool {
    if expr.kind == SyntaxKind::BigIntLiteral {
        return true;
    }

    false
}
