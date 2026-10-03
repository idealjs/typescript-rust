use std::sync::Arc;

use super::m4o::ListFormat;
use super::m4o_2::{CommentState, EmitContext, Printer, WriteKind};
use super::m4t_3::positions_are_on_same_line;
use super::m4t_4::{
    is_new_expression_without_arguments, mixing_binary_operators_requires_parentheses,
};
use crate::ast::mig::m3e_3::get_binary_operator_precedence;
use crate::ast::mig::m3e_3::{
    get_leftmost_expression, OperatorPrecedence, TypePrecedence, OPERATOR_PRECEDENCE_COALESCE,
    OPERATOR_PRECEDENCE_DISALLOW_COMMA, OPERATOR_PRECEDENCE_HIGHEST,
    OPERATOR_PRECEDENCE_LOWEST, TYPE_PRECEDENCE_LOWEST,
};
use crate::ast::node_data_generated::{is_block, is_object_literal_expression};
use crate::ast::node_node::Node;
use crate::ast::node_node_list::{ModifierList, NodeList};
use crate::ast::syntax_kind_generated::SyntaxKind;
use crate::ast::utilities_expressions::{
    is_expression, is_optional_chain, skip_partially_emitted_expressions_arc,
};
use crate::ast::utilities_synthesized::node_is_synthesized;
use crate::ast::node_data_generated::NodeData;
use tsox_core::core::text::TextRange;

pub(crate) struct GetLiteralTextFlags(pub u32);

impl GetLiteralTextFlags {
    pub const NONE: GetLiteralTextFlags = GetLiteralTextFlags(0);
}

pub(crate) type EmitFlags = u32;

pub(crate) const EF_NONE: EmitFlags = 0;
pub(crate) const EF_HELPER_NAME: EmitFlags = 1 << 10;

pub(crate) const ARRAY_BINDING_PATTERN_ELEMENTS: ListFormat =
    ListFormat((1 << 6) | (1 << 4) | (1 << 9) | (1 << 19));
pub(crate) const ARRAY_LITERAL_EXPRESSION_ELEMENTS: ListFormat =
    ListFormat((1 << 1) | (1 << 4) | (1 << 9) | (1 << 6) | (1 << 7) | (1 << 13));
pub(crate) const CALL_EXPRESSION_ARGUMENTS: ListFormat =
    ListFormat((1 << 4) | (1 << 9) | (1 << 11));

pub(crate) struct TokenEmitFlags;

impl TokenEmitFlags {
    pub const NO_SOURCE_MAPS: u32 = (1 << 2) | (1 << 3);
}


impl Printer {
    pub fn emit_big_int_literal(&mut self, node: &Arc<Node>) { ::tsox_core::fntrace::enter("emit_big_int_literal"); 
        let state = self.enter_node(node);
        self.emit_literal(node, GetLiteralTextFlags::NONE);
        self.exit_node(node, state);
    }

    pub fn emit_binding_identifier(&mut self, node: &Arc<Node>) { ::tsox_core::fntrace::enter("emit_binding_identifier"); 
        let mut node = node.clone();
        if self.unique_helper_names.is_some()
            && (self.emit_context.emit_flags(&node) & EF_HELPER_NAME) != EF_NONE
        {
            let helper_name = self.get_unique_helper_name(node.text());
            self.emit_context
                .assign_comment_and_source_map_ranges(&helper_name, &node);
            node = helper_name;
        }

        let state = self.enter_node(&node);
        self.emit_identifier_text(&node);
        self.exit_node(&node, state);
    }

    pub fn emit_computed_property_name(&mut self, node: &Arc<Node>) { ::tsox_core::fntrace::enter("emit_computed_property_name"); 
        let state = self.enter_node(node);
        self.write_punctuation("[");
        let expression = node.expression().expect("computed property name requires expression");
        self.emit_expression(expression, OPERATOR_PRECEDENCE_DISALLOW_COMMA);
        self.write_punctuation("]");
        self.exit_node(node, state);
    }

    pub fn emit_binding_name(&mut self, node: Option<&Arc<Node>>) { ::tsox_core::fntrace::enter("emit_binding_name"); 
        let Some(node) = node else {
            return;
        };

        match node.kind {
            SyntaxKind::Identifier => self.emit_binding_identifier(node),
            SyntaxKind::ObjectBindingPattern => self.emit_object_binding_pattern(node),
            SyntaxKind::ArrayBindingPattern => self.emit_array_binding_pattern(node),
            _ => panic!("unexpected BindingName: {:?}", node.kind),
        }
    }

    pub fn emit_decorator(&mut self, node: &Arc<Node>) { ::tsox_core::fntrace::enter("emit_decorator"); 
        let state = self.enter_node(node);
        self.write_punctuation("@");
        let expression = node.expression().expect("decorator requires expression");
        self.emit_expression(expression, OperatorPrecedence::LeftHandSide);
        self.exit_node(node, state);
    }

    pub fn emit_class_static_block_declaration(&mut self, node: &Arc<Node>) { ::tsox_core::fntrace::enter("emit_class_static_block_declaration"); 
        let state = self.enter_node(node);
        self.write_keyword("static");
        self.push_name_generation_scope(node);
        self.emit_function_body_node(node.body());
        self.pop_name_generation_scope(node);
        self.exit_node(node, state);
    }

    pub fn emit_constructor(&mut self, node: &Arc<Node>) { ::tsox_core::fntrace::enter("emit_constructor"); 
        let state = self.enter_node(node);
        self.emit_modifier_list(node, node.modifiers(), false);
        self.write_keyword("constructor");
        let indented = self.should_emit_indented(node);
        self.increase_indent_if(indented);
        self.push_name_generation_scope(node);
        self.emit_signature(node);
        self.emit_function_body_node(node.body());
        self.pop_name_generation_scope(node);
        self.decrease_indent_if(indented);
        self.exit_node(node, state);
    }

    pub fn emit_accessor_declaration(&mut self, token: SyntaxKind, node: &Arc<Node>) { ::tsox_core::fntrace::enter("emit_accessor_declaration"); 
        let state = self.enter_node(node);
        let pos = self.emit_modifier_list(node, node.modifiers(), true);
        self.emit_token(token, pos, WriteKind::Keyword, node);
        self.write_space();
        self.emit_property_name(node.name());
        let indented = self.should_emit_indented(node);
        self.increase_indent_if(indented);
        self.push_name_generation_scope(node);
        self.emit_signature(node);
        self.emit_function_body_node(node.body());
        self.pop_name_generation_scope(node);
        self.decrease_indent_if(indented);
        self.exit_node(node, state);
    }

    pub fn emit_call_signature(&mut self, node: &Arc<Node>) { ::tsox_core::fntrace::enter("emit_call_signature"); 
        let state = self.enter_node(node);
        let indented = self.should_emit_indented(node);
        self.increase_indent_if(indented);
        self.push_name_generation_scope(node);
        self.emit_signature(node);
        self.write_trailing_semicolon();
        self.pop_name_generation_scope(node);
        self.decrease_indent_if(indented);
        self.exit_node(node, state);
    }

    pub fn emit_construct_signature(&mut self, node: &Arc<Node>) { ::tsox_core::fntrace::enter("emit_construct_signature"); 
        let state = self.enter_node(node);
        self.write_keyword("new");
        self.write_space();
        let indented = self.should_emit_indented(node);
        self.increase_indent_if(indented);
        self.push_name_generation_scope(node);
        self.emit_signature(node);
        self.write_trailing_semicolon();
        self.pop_name_generation_scope(node);
        self.decrease_indent_if(indented);
        self.exit_node(node, state);
    }

    pub fn emit_class_element(&mut self, node: &Arc<Node>) { ::tsox_core::fntrace::enter("emit_class_element"); 
        match node.kind {
            SyntaxKind::PropertyDeclaration => self.emit_property_declaration(node),
            SyntaxKind::MethodDeclaration => self.emit_method_declaration(node),
            SyntaxKind::ClassStaticBlockDeclaration => self.emit_class_static_block_declaration(node),
            SyntaxKind::Constructor => self.emit_constructor(node),
            SyntaxKind::GetAccessor => self.emit_accessor_declaration(SyntaxKind::GetKeyword, node),
            SyntaxKind::SetAccessor => self.emit_accessor_declaration(SyntaxKind::SetKeyword, node),
            SyntaxKind::IndexSignature => self.emit_index_signature(node),
            SyntaxKind::SemicolonClassElement => self.emit_semicolon_class_element(node),
            SyntaxKind::NotEmittedStatement => self.emit_not_emitted_statement(node),
            SyntaxKind::JSTypeAliasDeclaration => self.emit_type_alias_declaration(node),
            _ => panic!("unexpected ClassElement: {:?}", node.kind),
        }
    }

    pub fn emit_constructor_type(&mut self, node: &Arc<Node>) { ::tsox_core::fntrace::enter("emit_constructor_type"); 
        let state = self.enter_node(node);
        self.emit_modifier_list(node, node.modifiers(), false);
        self.write_keyword("new");
        self.write_space();
        let indented = self.should_emit_indented(node);
        self.increase_indent_if(indented);
        self.push_name_generation_scope(node);
        self.emit_type_parameters(node, node_list_slice(node.type_parameters()));
        self.emit_parameters(node, node_list_slice(node.parameters()));
        self.write_space();
        self.emit_return_type(node.type_());
        self.pop_name_generation_scope(node);
        self.decrease_indent_if(indented);
        self.exit_node(node, state);
    }

    pub fn emit_array_type(&mut self, node: &Arc<Node>) { ::tsox_core::fntrace::enter("emit_array_type"); 
        let state = self.enter_node(node);
        self.emit_postfix_type_operand(node.element_type(), node);
        self.write_punctuation("[");
        self.write_punctuation("]");
        self.exit_node(node, state);
    }

    pub fn emit_conditional_type(&mut self, node: &Arc<Node>) { ::tsox_core::fntrace::enter("emit_conditional_type"); 
        let state = self.enter_node(node);
        self.emit_type_node(node.check_type(), TypePrecedence::Union);
        self.write_space();
        self.write_keyword("extends");
        self.write_space();
        self.emit_type_node_in_extends(node.extends_type());
        self.write_space();
        self.write_punctuation("?");
        self.write_space();
        self.emit_type_node_outside_extends(node.true_type());
        self.write_space();
        self.write_punctuation(":");
        self.write_space();
        self.emit_type_node_outside_extends(node.false_type());
        self.exit_node(node, state);
    }

    pub fn emit_array_binding_pattern(&mut self, node: &Arc<Node>) { ::tsox_core::fntrace::enter("emit_array_binding_pattern"); 
        let state = self.enter_node(node);
        self.write_punctuation("[");
        self.emit_list(
            Printer::emit_binding_element_node,
            node,
            node_list_slice(node.elements()),
            ARRAY_BINDING_PATTERN_ELEMENTS,
        );
        self.write_punctuation("]");
        self.exit_node(node, state);
    }

    pub fn emit_binding_element(&mut self, node: &Arc<Node>) { ::tsox_core::fntrace::enter("emit_binding_element"); 
        let state = self.enter_node(node);
        self.emit_token_node(node.dot_dot_dot_token());
        if let Some(property_name) = node.property_name() {
            self.emit_property_name(Some(&property_name));
            self.write_punctuation(":");
            self.write_space();
        }
        if let Some(name) = node.name() {
            let name_end = name.end();
            self.emit_binding_name(Some(&name));
            self.emit_initializer(node.initializer(), name_end, node);
        }
        self.exit_node(node, state);
    }

    pub fn emit_binding_element_node(&mut self, node: &Arc<Node>) { ::tsox_core::fntrace::enter("emit_binding_element_node"); 
        self.emit_binding_element(node);
    }

    pub fn emit_array_literal_expression_element(&mut self, node: &Arc<Node>) { ::tsox_core::fntrace::enter("emit_array_literal_expression_element"); 
        self.emit_expression(node, OperatorPrecedence::Spread);
    }

    pub fn emit_array_literal_expression(&mut self, node: &Arc<Node>) { ::tsox_core::fntrace::enter("emit_array_literal_expression"); 
        let state = self.enter_node(node);
        self.emit_list(
            Printer::emit_array_literal_expression_element,
            node,
            node_list_slice(node.elements()),
            ARRAY_LITERAL_EXPRESSION_ELEMENTS
                | if node.multi_line() {
                    ListFormat::PREFER_NEW_LINE
                } else {
                    ListFormat::NONE
                },
        );
        self.exit_node(node, state);
    }

    pub fn emit_argument(&mut self, node: &Arc<Node>) { ::tsox_core::fntrace::enter("emit_argument"); 
        self.emit_expression(node, OperatorPrecedence::Spread);
    }

    pub fn emit_callee(&mut self, callee: &Arc<Node>, parent_node: &Arc<Node>) { ::tsox_core::fntrace::enter("emit_callee"); 
        if self.should_emit_indirect_call(parent_node) {
            self.write_punctuation("(");
            self.write_literal("0");
            self.write_punctuation(",");
            self.write_space();
            self.emit_expression(callee, OperatorPrecedence::Comma);
            self.write_punctuation(")");
        } else if parent_node.kind == SyntaxKind::CallExpression
            && is_new_expression_without_arguments(&skip_partially_emitted_expressions_arc(callee))
        {
            self.emit_expression(callee, OperatorPrecedence::Parentheses);
        } else {
            self.emit_expression(
                callee,
                if is_optional_chain(parent_node) {
                    OperatorPrecedence::OptionalChain
                } else {
                    OperatorPrecedence::Member
                },
            );
        }
    }

    pub fn emit_call_expression(&mut self, node: &Arc<Node>) { ::tsox_core::fntrace::enter("emit_call_expression"); 
        let state = self.enter_node(node);
        let expression = node.expression().expect("call expression requires expression");
        self.emit_callee(expression, node);
        self.emit_token_node(node.question_dot_token());
        self.emit_type_arguments(node, node.type_arguments());
        self.emit_list(
            Printer::emit_argument,
            node,
            node_list_slice(node.arguments()),
            CALL_EXPRESSION_ARGUMENTS,
        );
        self.exit_node(node, state);
    }

    pub fn emit_concise_body(&mut self, node: &Arc<Node>) { ::tsox_core::fntrace::enter("emit_concise_body"); 
        if is_block(node) {
            self.emit_function_body(node);
        } else if is_object_literal_expression(&get_leftmost_expression(node, false)) {
            self.write_punctuation("(");
            self.emit_expression(node, OPERATOR_PRECEDENCE_LOWEST);
            self.write_punctuation(")");
        } else if is_expression(node) {
            self.emit_expression(node, OperatorPrecedence::Yield);
        } else {
            panic!("unexpected ConciseBody: {:?}", node.kind);
        }
    }

    pub fn emit_arrow_function(&mut self, node: &Arc<Node>) { ::tsox_core::fntrace::enter("emit_arrow_function"); 
        let state = self.enter_node(node);
        self.emit_modifier_list(node, node.modifiers(), false);
        let indented = self.should_emit_indented(node);
        self.increase_indent_if(indented);
        self.push_name_generation_scope(node);
        self.emit_type_parameters(node, node_list_slice(node.type_parameters()));
        self.emit_parameters_for_arrow(node, node_list_slice(node.parameters()));
        self.emit_type_annotation(node.type_());
        self.write_space();
        self.emit_token_node(Some(node.equals_greater_than_token()));
        self.write_space();
        let body = node.body().expect("arrow function requires body");
        self.emit_concise_body(body);
        self.pop_name_generation_scope(node);
        self.decrease_indent_if(indented);
        self.exit_node(node, state);
    }

    pub fn emit_delete_expression(&mut self, node: &Arc<Node>) { ::tsox_core::fntrace::enter("emit_delete_expression"); 
        let state = self.enter_node(node);
        self.emit_token(SyntaxKind::DeleteKeyword, node.pos(), WriteKind::Keyword, node);
        self.write_space();
        let expression = node.expression().expect("unary expression requires expression");
        self.emit_expression(expression, OperatorPrecedence::Unary);
        self.exit_node(node, state);
    }

    pub fn emit_await_expression(&mut self, node: &Arc<Node>) { ::tsox_core::fntrace::enter("emit_await_expression"); 
        let state = self.enter_node(node);
        self.emit_token(SyntaxKind::AwaitKeyword, node.pos(), WriteKind::Keyword, node);
        self.write_space();
        let expression = node.expression().expect("unary expression requires expression");
        self.emit_expression(expression, OperatorPrecedence::Unary);
        self.exit_node(node, state);
    }

    pub fn emit_binary_expression(&mut self, node: &Arc<Node>) { ::tsox_core::fntrace::enter("emit_binary_expression"); 
        let (mut left_prec, mut right_prec) = self.get_binary_expression_precedence(node);
        let emitted_left = skip_partially_emitted_expressions_arc(&node.left());
        if node_is_synthesized(&emitted_left)
            && emitted_left.kind == SyntaxKind::BinaryExpression
            && mixing_binary_operators_requires_parentheses(
                node.operator_token().kind,
                emitted_left.operator_token().kind,
            )
        {
            left_prec = OPERATOR_PRECEDENCE_HIGHEST;
        }
        let emitted_right = skip_partially_emitted_expressions_arc(&node.right());
        if node_is_synthesized(&emitted_right)
            && emitted_right.kind == SyntaxKind::BinaryExpression
            && mixing_binary_operators_requires_parentheses(
                node.operator_token().kind,
                emitted_right.operator_token().kind,
            )
        {
            right_prec = OPERATOR_PRECEDENCE_HIGHEST;
        }
        let state = self.enter_node(node);
        self.emit_expression(node.left(), left_prec);
        let operator_token = node.operator_token();
        let lines_before_operator =
            self.get_lines_between_nodes(node, node.left(), operator_token);
        let lines_after_operator =
            self.get_lines_between_nodes(node, operator_token, node.right());
        self.write_lines_and_indent(
            lines_before_operator,
            operator_token.kind != SyntaxKind::CommaToken,
        );
        self.emit_token_node_ex(operator_token, TokenEmitFlags::NO_SOURCE_MAPS);
        self.write_lines_and_indent(lines_after_operator, true);
        self.emit_expression(node.right(), right_prec);
        self.decrease_indent_if(lines_after_operator > 0);
        self.decrease_indent_if(lines_before_operator > 0);
        self.exit_node(node, state);
    }

    pub fn emit_conditional_expression(&mut self, node: &Arc<Node>) { ::tsox_core::fntrace::enter("emit_conditional_expression"); 
        let state = self.enter_node(node);
        let question_token = node.question_token();
        let when_true = node.when_true();
        let colon_token = node.colon_token();
        let lines_before_question = self.get_lines_between_nodes(node, &node.condition(), question_token);
        let lines_after_question = self.get_lines_between_nodes(node, &question_token, when_true);
        let lines_before_colon = self.get_lines_between_nodes(node, &when_true, colon_token);
        let lines_after_colon = self.get_lines_between_nodes(node, &colon_token, &node.when_false());
        self.emit_short_circuit_expression(node.condition());
        self.write_lines_and_indent(lines_before_question, true);
        self.emit_punctuation_node(question_token);
        self.write_lines_and_indent(lines_after_question, true);
        self.emit_expression(when_true, OperatorPrecedence::Yield);
        self.decrease_indent_if(lines_after_question > 0);
        self.decrease_indent_if(lines_before_question > 0);
        self.write_lines_and_indent(lines_before_colon, true);
        self.emit_punctuation_node(colon_token);
        self.write_lines_and_indent(lines_after_colon, true);
        self.emit_expression(node.when_false(), OperatorPrecedence::Yield);
        self.decrease_indent_if(lines_after_colon > 0);
        self.decrease_indent_if(lines_before_colon > 0);
        self.exit_node(node, state);
    }
}

impl Node {
    pub fn body(&self) -> Option<&Arc<Node>> { ::tsox_core::fntrace::enter("body"); 
        match &self.data {
            NodeData::FunctionDeclaration(d) => d.body.as_ref(),
            NodeData::FunctionExpression(d) => Some(&d.body),
            NodeData::MethodDeclaration(d) => d.body.as_ref(),
            NodeData::ConstructorDeclaration(d) => d.body.as_ref(),
            NodeData::GetAccessorDeclaration(d) => d.body.as_ref(),
            NodeData::SetAccessorDeclaration(d) => d.body.as_ref(),
            NodeData::ArrowFunction(d) => Some(&d.body),
            NodeData::ClassStaticBlockDeclaration(d) => Some(&d.body),
            _ => None,
        }
    }

    pub fn type_parameters(&self) -> Option<&Arc<NodeList>> { ::tsox_core::fntrace::enter("type_parameters"); 
        match &self.data {
            NodeData::FunctionDeclaration(d) => d.type_parameters.as_ref(),
            NodeData::FunctionExpression(d) => d.type_parameters.as_ref(),
            NodeData::MethodDeclaration(d) => d.type_parameters.as_ref(),
            NodeData::ConstructorDeclaration(d) => d.type_parameters.as_ref(),
            NodeData::ConstructorTypeNode(d) => d.type_parameters.as_ref(),
            NodeData::ArrowFunction(d) => d.type_parameters.as_ref(),
            _ => None,
        }
    }

    pub fn parameters(&self) -> Option<&Arc<NodeList>> { ::tsox_core::fntrace::enter("parameters"); 
        match &self.data {
            NodeData::FunctionDeclaration(d) => Some(&d.parameters),
            NodeData::FunctionExpression(d) => Some(&d.parameters),
            NodeData::MethodDeclaration(d) => Some(&d.parameters),
            NodeData::ConstructorDeclaration(d) => Some(&d.parameters),
            NodeData::ConstructorTypeNode(d) => Some(&d.parameters),
            NodeData::ArrowFunction(d) => Some(&d.parameters),
            _ => None,
        }
    }

    pub fn type_(&self) -> Option<&Arc<Node>> { ::tsox_core::fntrace::enter("type_"); 
        crate::ast::node_data_generated::node_type(self)
    }

    pub fn element_type(&self) -> &Arc<Node> { ::tsox_core::fntrace::enter("element_type"); 
        match &self.data {
            NodeData::ArrayTypeNode(d) => &d.element_type,
            _ => unreachable!("element_type on non-array type"),
        }
    }

    pub fn check_type(&self) -> &Arc<Node> { ::tsox_core::fntrace::enter("check_type"); 
        match &self.data {
            NodeData::ConditionalTypeNode(d) => &d.check_type,
            _ => unreachable!("check_type on non-conditional type"),
        }
    }

    pub fn extends_type(&self) -> &Arc<Node> { ::tsox_core::fntrace::enter("extends_type"); 
        match &self.data {
            NodeData::ConditionalTypeNode(d) => &d.extends_type,
            _ => unreachable!("extends_type on non-conditional type"),
        }
    }

    pub fn true_type(&self) -> &Arc<Node> { ::tsox_core::fntrace::enter("true_type"); 
        match &self.data {
            NodeData::ConditionalTypeNode(d) => &d.true_type,
            _ => unreachable!("true_type on non-conditional type"),
        }
    }

    pub fn false_type(&self) -> &Arc<Node> { ::tsox_core::fntrace::enter("false_type"); 
        match &self.data {
            NodeData::ConditionalTypeNode(d) => &d.false_type,
            _ => unreachable!("false_type on non-conditional type"),
        }
    }

    pub fn elements(&self) -> Option<&Arc<NodeList>> { ::tsox_core::fntrace::enter("elements"); 
        match &self.data {
            NodeData::BindingPattern(d) => Some(&d.elements),
            NodeData::ArrayLiteralExpression(d) => Some(&d.elements),
            NodeData::JsxAttributes(d) => Some(&d.properties),
            _ => None,
        }
    }

    pub fn dot_dot_dot_token(&self) -> Option<&Arc<Node>> { ::tsox_core::fntrace::enter("dot_dot_dot_token"); 
        match &self.data {
            NodeData::BindingElement(d) => d.dot_dot_dot_token.as_ref(),
            NodeData::ParameterDeclaration(d) => d.dot_dot_dot_token.as_ref(),
            _ => None,
        }
    }

    pub fn property_name(&self) -> Option<&Arc<Node>> { ::tsox_core::fntrace::enter("property_name"); 
        match &self.data {
            NodeData::BindingElement(d) => d.property_name.as_ref(),
            NodeData::ExportSpecifier(d) => d.property_name.as_ref(),
            NodeData::ImportSpecifier(d) => d.property_name.as_ref(),
            _ => None,
        }
    }

    pub fn initializer(&self) -> Option<&Arc<Node>> { ::tsox_core::fntrace::enter("initializer"); 
        match &self.data {
            NodeData::BindingElement(d) => d.initializer.as_ref(),
            NodeData::VariableDeclaration(d) => d.initializer.as_ref(),
            NodeData::PropertyDeclaration(d) => d.initializer.as_ref(),
            NodeData::PropertyAssignment(d) => Some(&d.initializer),
            _ => None,
        }
    }

    pub fn multi_line(&self) -> bool { ::tsox_core::fntrace::enter("multi_line"); 
        match &self.data {
            NodeData::ArrayLiteralExpression(d) => d.multi_line,
            _ => false,
        }
    }

    pub fn question_dot_token(&self) -> Option<&Arc<Node>> { ::tsox_core::fntrace::enter("question_dot_token"); 
        match &self.data {
            NodeData::CallExpression(d) => d.question_dot_token.as_ref(),
            NodeData::PropertyAccessExpression(d) => d.question_dot_token.as_ref(),
            _ => None,
        }
    }

    pub fn type_arguments(&self) -> Option<&Arc<NodeList>> { ::tsox_core::fntrace::enter("type_arguments"); 
        match &self.data {
            NodeData::CallExpression(d) => d.type_arguments.as_ref(),
            NodeData::NewExpression(d) => d.type_arguments.as_ref(),
            NodeData::JsxOpeningElement(d) => d.type_arguments.as_ref(),
            NodeData::JsxSelfClosingElement(d) => d.type_arguments.as_ref(),
            NodeData::TaggedTemplateExpression(d) => d.type_arguments.as_ref(),
            NodeData::ExpressionWithTypeArguments(d) => d.type_arguments.as_ref(),
            _ => None,
        }
    }

    pub fn arguments(&self) -> Option<&Arc<NodeList>> { ::tsox_core::fntrace::enter("arguments"); 
        match &self.data {
            NodeData::CallExpression(d) => Some(&d.arguments),
            NodeData::NewExpression(d) => d.arguments.as_ref(),
            _ => None,
        }
    }

    pub fn left(&self) -> &Arc<Node> { ::tsox_core::fntrace::enter("left"); 
        match &self.data {
            NodeData::BinaryExpression(d) => &d.left,
            _ => unreachable!("left on non-binary expression"),
        }
    }

    pub fn right(&self) -> &Arc<Node> { ::tsox_core::fntrace::enter("right"); 
        match &self.data {
            NodeData::BinaryExpression(d) => &d.right,
            _ => unreachable!("right on non-binary expression"),
        }
    }

    pub fn operator_token(&self) -> &Arc<Node> { ::tsox_core::fntrace::enter("operator_token"); 
        match &self.data {
            NodeData::BinaryExpression(d) => &d.operator_token,
            _ => unreachable!("operator_token on non-binary expression"),
        }
    }

    pub fn question_token(&self) -> &Arc<Node> { ::tsox_core::fntrace::enter("question_token"); 
        match &self.data {
            NodeData::ConditionalExpression(d) => &d.question_token,
            _ => unreachable!("question_token on non-conditional expression"),
        }
    }

    pub fn when_true(&self) -> &Arc<Node> { ::tsox_core::fntrace::enter("when_true"); 
        match &self.data {
            NodeData::ConditionalExpression(d) => &d.when_true,
            _ => unreachable!("when_true on non-conditional expression"),
        }
    }

    pub fn colon_token(&self) -> &Arc<Node> { ::tsox_core::fntrace::enter("colon_token"); 
        match &self.data {
            NodeData::ConditionalExpression(d) => &d.colon_token,
            _ => unreachable!("colon_token on non-conditional expression"),
        }
    }

    pub fn condition(&self) -> &Arc<Node> { ::tsox_core::fntrace::enter("condition"); 
        match &self.data {
            NodeData::ConditionalExpression(d) => &d.condition,
            _ => unreachable!("condition on non-conditional expression"),
        }
    }

    pub fn when_false(&self) -> &Arc<Node> { ::tsox_core::fntrace::enter("when_false"); 
        match &self.data {
            NodeData::ConditionalExpression(d) => &d.when_false,
            _ => unreachable!("when_false on non-conditional expression"),
        }
    }

    pub fn equals_greater_than_token(&self) -> &Arc<Node> { ::tsox_core::fntrace::enter("equals_greater_than_token"); 
        match &self.data {
            NodeData::ArrowFunction(d) => &d.equals_greater_than_token,
            _ => unreachable!("equals_greater_than_token on non-arrow function"),
        }
    }
}

pub(crate) struct PrinterState {
    pub(crate) comment_state: Option<CommentState>,
}

impl Default for PrinterState {
    fn default() -> Self { ::tsox_core::fntrace::enter("default"); 
        PrinterState { comment_state: None }
    }
}

fn node_list_slice(list: Option<&Arc<NodeList>>) -> &[Arc<Node>] { ::tsox_core::fntrace::enter("node_list_slice"); 
    list.map(|l| l.nodes.as_slice()).unwrap_or(&[])
}

impl EmitContext {
    pub fn emit_flags(&self, _node: &Arc<Node>) -> u32 { ::tsox_core::fntrace::enter("emit_flags"); 
        0
    }

    pub fn comment_range(&self, node: &Arc<Node>) -> TextRange { ::tsox_core::fntrace::enter("comment_range"); 
        node.loc
    }

    pub fn get_type_node(&self, _node: &Arc<Node>) -> Option<Arc<Node>> { ::tsox_core::fntrace::enter("get_type_node"); 
        None
    }

    pub fn parse_node(&self, _node: &Arc<Node>) -> Option<Arc<Node>> { ::tsox_core::fntrace::enter("parse_node"); 
        None
    }

    pub fn assign_comment_and_source_map_ranges(&self, _node: &Arc<Node>, _source: &Arc<Node>) { ::tsox_core::fntrace::enter("assign_comment_and_source_map_ranges"); }
}

impl Printer {
    pub fn enter_node(&mut self, node: &Arc<Node>) -> PrinterState { ::tsox_core::fntrace::enter("enter_node"); 
        let mut state = PrinterState::default();
        if let Some(on_before_emit_node) = &self.print_handlers.on_before_emit_node {
            on_before_emit_node(Some(node));
        }
        state.comment_state = self.emit_comments_before_node(node);
        state
    }

    pub fn exit_node(&mut self, node: &Arc<Node>, previous_state: PrinterState) { ::tsox_core::fntrace::enter("exit_node"); 
        self.emit_comments_after_node(node, previous_state.comment_state);
        if let Some(on_after_emit_node) = &self.print_handlers.on_after_emit_node {
            on_after_emit_node(Some(node));
        }
    }

    pub fn enter_token(
        &mut self,
        token: SyntaxKind,
        pos: usize,
        context_node: &Arc<Node>,
        flags: super::m4o::TokenEmitFlags,
    ) -> (PrinterState, usize) { ::tsox_core::fntrace::enter("enter_token"); 
        let mut state = PrinterState::default();
        if let Some(on_before_emit_token) = &self.print_handlers.on_before_emit_token {
            on_before_emit_token(Some(context_node));
        }
        let (comment_state, pos) = self.emit_comments_before_token(token, pos, context_node, flags);
        state.comment_state = comment_state;
        (state, pos)
    }

    pub fn exit_token(
        &mut self,
        token: SyntaxKind,
        pos: usize,
        context_node: &Arc<Node>,
        previous_state: PrinterState,
    ) { ::tsox_core::fntrace::enter("exit_token"); 
        self.emit_comments_after_token(token, pos, context_node, previous_state.comment_state);
        if let Some(on_after_emit_token) = &self.print_handlers.on_after_emit_token {
            on_after_emit_token(Some(context_node));
        }
    }

    pub fn write_as(&mut self, text: &str, _write_kind: WriteKind) { ::tsox_core::fntrace::enter("write_as"); 
        if let Some(writer) = self.writer.as_ref() {
            writer.write(text);
        }
    }

    pub fn write_space(&mut self) { ::tsox_core::fntrace::enter("write_space"); 
        self.write_as(" ", WriteKind::None);
    }

    pub fn write_line(&mut self) { ::tsox_core::fntrace::enter("write_line"); 
        if let Some(writer) = self.writer.as_ref() {
            writer.write_line("");
        }
    }

    pub fn write_punctuation(&mut self, text: &str) { ::tsox_core::fntrace::enter("write_punctuation"); 
        self.write_as(text, WriteKind::Punctuation);
    }

    pub fn write_keyword(&mut self, text: &str) { ::tsox_core::fntrace::enter("write_keyword"); 
        self.write_as(text, WriteKind::Keyword);
    }

    pub fn write_literal(&mut self, text: &str) { ::tsox_core::fntrace::enter("write_literal"); 
        self.write_as(text, WriteKind::Literal);
    }

    pub fn write_lines_and_indent(&mut self, line_count: i32, write_space_if_not_indenting: bool) { ::tsox_core::fntrace::enter("write_lines_and_indent"); 
        if line_count > 0 {
            self.increase_indent_if(true);
            for _ in 0..line_count {
                self.write_line();
            }
        } else if write_space_if_not_indenting {
            self.write_space();
        }
    }

    pub fn write_trailing_semicolon(&mut self) { ::tsox_core::fntrace::enter("write_trailing_semicolon"); 
        self.write_token_text(SyntaxKind::SemicolonToken, WriteKind::Punctuation, usize::MAX);
    }

    pub fn increase_indent_if(&mut self, indent_requested: bool) { ::tsox_core::fntrace::enter("increase_indent_if"); 
        if indent_requested {
            if let Some(writer) = self.writer.as_ref() {
                writer.increase_indent();
            }
        }
    }

    pub fn write_token_text(
        &mut self,
        token: SyntaxKind,
        write_kind: WriteKind,
        pos: usize,
    ) -> usize { ::tsox_core::fntrace::enter("write_token_text"); 
        let token_string = crate::scanner::token_to_string(token);
        self.write_as(token_string, write_kind);
        pos + token_string.len()
    }

    pub fn emit_token(
        &mut self,
        token: SyntaxKind,
        pos: usize,
        write_kind: WriteKind,
        context_node: &Arc<Node>,
    ) -> usize { ::tsox_core::fntrace::enter("emit_token"); 
        self.emit_token_ex(token, pos, write_kind, context_node, super::m4o::TokenEmitFlags::NONE)
    }

    pub fn emit_token_ex(
        &mut self,
        token: SyntaxKind,
        pos: usize,
        write_kind: WriteKind,
        context_node: &Arc<Node>,
        flags: super::m4o::TokenEmitFlags,
    ) -> usize { ::tsox_core::fntrace::enter("emit_token_ex"); 
        let (state, pos) = self.enter_token(token, pos, context_node, flags);
        let pos = self.write_token_text(token, write_kind, pos);
        self.exit_token(token, pos, context_node, state);
        pos
    }

    pub fn emit_token_node(&mut self, node: Option<&Arc<Node>>) { ::tsox_core::fntrace::enter("emit_token_node"); 
        let Some(node) = node else {
            return;
        };
        self.emit_token_node_ex(node, 0);
    }

    pub fn emit_token_node_ex(&mut self, node: &Arc<Node>, _flags: u32) { ::tsox_core::fntrace::enter("emit_token_node_ex"); 
        let state = self.enter_node(node);
        let write_kind = if crate::ast::node_data_generated::is_keyword_kind(node.kind) {
            WriteKind::Keyword
        } else if crate::ast::node_data_generated::is_punctuation_kind(node.kind) {
            WriteKind::Punctuation
        } else {
            WriteKind::None
        };
        self.write_token_text(node.kind, write_kind, node.pos());
        self.exit_node(node, state);
    }

    pub fn emit_punctuation_node(&mut self, node: &Arc<Node>) { ::tsox_core::fntrace::enter("emit_punctuation_node"); 
        self.emit_token_node_ex(node, 0);
    }

    pub fn emit_literal(&mut self, node: &Arc<Node>, _flags: GetLiteralTextFlags) { ::tsox_core::fntrace::enter("emit_literal"); 
        self.write_as(node.text(), WriteKind::Literal);
    }

    pub fn emit_identifier_name(&mut self, node: &Arc<Node>) { ::tsox_core::fntrace::enter("emit_identifier_name"); 
        self.write_as(node.text(), WriteKind::None);
    }

    pub fn emit_identifier_text(&mut self, node: &Arc<Node>) { ::tsox_core::fntrace::enter("emit_identifier_text"); 
        self.write_as(node.text(), WriteKind::None);
    }

    pub fn emit_label_identifier(&mut self, node: &Arc<Node>) { ::tsox_core::fntrace::enter("emit_label_identifier"); 
        self.emit_identifier_text(node);
    }

    pub fn emit_property_name(&mut self, node: Option<&Arc<Node>>) { ::tsox_core::fntrace::enter("emit_property_name"); 
        let Some(node) = node else {
            return;
        };
        match node.kind {
            SyntaxKind::ComputedPropertyName => self.emit_computed_property_name(node),
            SyntaxKind::PrivateIdentifier => self.emit_identifier_name(node),
            _ => self.emit_literal(node, GetLiteralTextFlags::NONE),
        }
    }

    pub fn emit_expression(&mut self, node: &Arc<Node>, precedence: OperatorPrecedence) { ::tsox_core::fntrace::enter("emit_expression"); 
        let _ = precedence;
        match node.kind {
            SyntaxKind::BinaryExpression => self.emit_binary_expression(node),
            SyntaxKind::ConditionalExpression => self.emit_conditional_expression(node),
            SyntaxKind::ArrowFunction => self.emit_arrow_function(node),
            SyntaxKind::CallExpression => self.emit_call_expression(node),
            SyntaxKind::NewExpression => {
                let expression = node.expression().expect("new expression requires expression");
                self.emit_callee(expression, node);
                self.emit_type_arguments(node, node.type_arguments());
                self.emit_list(Printer::emit_argument, node, node_list_slice(node.arguments()), CALL_EXPRESSION_ARGUMENTS);
            }
            SyntaxKind::ArrayLiteralExpression => self.emit_array_literal_expression(node),
            SyntaxKind::ClassExpression => self.emit_class_expression(node),
            SyntaxKind::AsExpression => self.emit_as_expression(node),
            SyntaxKind::DeleteExpression => self.emit_delete_expression(node),
            SyntaxKind::AwaitExpression => self.emit_await_expression(node),
            SyntaxKind::ComputedPropertyName => self.emit_computed_property_name(node),
            _ => self.write_as(node.text(), WriteKind::None),
        }
    }

    pub fn emit_short_circuit_expression(&mut self, node: &Arc<Node>) { ::tsox_core::fntrace::enter("emit_short_circuit_expression"); 
        match node.kind {
            SyntaxKind::BinaryExpression => self.emit_binary_expression(node),
            SyntaxKind::ConditionalExpression => self.emit_conditional_expression(node),
            _ => self.emit_expression(node, OPERATOR_PRECEDENCE_COALESCE),
        }
    }

    pub fn get_binary_expression_precedence(
        &self,
        node: &Arc<Node>,
    ) -> (OperatorPrecedence, OperatorPrecedence) { ::tsox_core::fntrace::enter("get_binary_expression_precedence"); 
        let NodeData::BinaryExpression(d) = &node.data else {
            return (OperatorPrecedence::Invalid, OperatorPrecedence::Invalid);
        };
        let precedence = get_binary_operator_precedence(d.operator_token.kind);
        (precedence, precedence)
    }

    pub fn emit_type_node(&mut self, node: &Arc<Node>, precedence: TypePrecedence) { ::tsox_core::fntrace::enter("emit_type_node"); 
        let _ = precedence;
        match node.kind {
            SyntaxKind::ArrayType => self.emit_array_type(node),
            SyntaxKind::ConditionalType => self.emit_conditional_type(node),
            SyntaxKind::ConstructorType => self.emit_constructor_type(node),
            _ => self.write_as(node.text(), WriteKind::None),
        }
    }

    pub fn emit_type_node_outside_extends(&mut self, node: &Arc<Node>) { ::tsox_core::fntrace::enter("emit_type_node_outside_extends"); 
        self.emit_type_node(node, TYPE_PRECEDENCE_LOWEST);
    }

    pub fn emit_type_node_in_extends(&mut self, node: &Arc<Node>) { ::tsox_core::fntrace::enter("emit_type_node_in_extends"); 
        self.emit_type_node(node, TYPE_PRECEDENCE_LOWEST);
    }

    pub fn emit_postfix_type_operand(&mut self, operand: &Arc<Node>, parent_node: &Arc<Node>) { ::tsox_core::fntrace::enter("emit_postfix_type_operand"); 
        let _ = parent_node;
        self.emit_type_node_outside_extends(operand);
    }

    pub fn emit_type_annotation(&mut self, node: Option<&Arc<Node>>) { ::tsox_core::fntrace::enter("emit_type_annotation"); 
        let Some(node) = node else {
            return;
        };
        self.write_punctuation(":");
        self.write_space();
        self.emit_type_node(node, TYPE_PRECEDENCE_LOWEST);
    }

    pub fn emit_return_type(&mut self, node: Option<&Arc<Node>>) { ::tsox_core::fntrace::enter("emit_return_type"); 
        let Some(node) = node else {
            return;
        };
        self.write_space();
        self.write_punctuation(":");
        self.write_space();
        self.emit_type_node(node, TYPE_PRECEDENCE_LOWEST);
    }

    pub fn emit_type_arguments(&mut self, context_node: &Arc<Node>, nodes: Option<&Arc<NodeList>>) { ::tsox_core::fntrace::enter("emit_type_arguments"); 
        let _ = context_node;
        let Some(nodes) = nodes else {
            return;
        };
        if nodes.nodes.is_empty() {
            return;
        }
        self.write_punctuation("<");
        for (i, node) in nodes.nodes.iter().enumerate() {
            if i > 0 {
                self.write_punctuation(",");
                self.write_space();
            }
            self.emit_type_node_outside_extends(node);
        }
        self.write_punctuation(">");
    }

    pub fn emit_type_parameters(&mut self, context_node: &Arc<Node>, nodes: &[Arc<Node>]) { ::tsox_core::fntrace::enter("emit_type_parameters"); 
        let _ = context_node;
        if nodes.is_empty() {
            return;
        }
        self.write_punctuation("<");
        for (i, node) in nodes.iter().enumerate() {
            if i > 0 {
                self.write_punctuation(",");
                self.write_space();
            }
            if let Some(name) = node.name() {
                self.emit_identifier_name(name);
            }
        }
        self.write_punctuation(">");
    }

    pub fn emit_parameters(&mut self, context_node: &Arc<Node>, nodes: &[Arc<Node>]) { ::tsox_core::fntrace::enter("emit_parameters"); 
        let _ = context_node;
        self.write_punctuation("(");
        for (i, node) in nodes.iter().enumerate() {
            if i > 0 {
                self.write_punctuation(",");
                self.write_space();
            }
            self.emit_parameter(node);
        }
        self.write_punctuation(")");
    }

    pub fn emit_parameters_for_arrow(&mut self, context_node: &Arc<Node>, nodes: &[Arc<Node>]) { ::tsox_core::fntrace::enter("emit_parameters_for_arrow"); 
        self.emit_parameters(context_node, nodes);
    }

    pub fn emit_parameter(&mut self, node: &Arc<Node>) { ::tsox_core::fntrace::enter("emit_parameter"); 
        self.emit_token_node(node.dot_dot_dot_token());
        if let Some(name) = node.name() {
            self.emit_binding_name(Some(name));
        }
        self.emit_type_annotation(node.type_());
        let name_end = node.name().map(|n| n.end()).unwrap_or_else(|| node.pos());
        self.emit_initializer(node.initializer(), name_end, node);
    }

    pub fn emit_initializer(
        &mut self,
        initializer: Option<&Arc<Node>>,
        equals_pos: usize,
        context_node: &Arc<Node>,
    ) { ::tsox_core::fntrace::enter("emit_initializer"); 
        let _ = equals_pos;
        let _ = context_node;
        if let Some(initializer) = initializer {
            self.write_space();
            self.write_punctuation("=");
            self.write_space();
            self.emit_expression(initializer, OperatorPrecedence::Assignment);
        }
    }

    pub fn emit_modifier_list(
        &mut self,
        node: &Arc<Node>,
        modifiers: Option<&Arc<ModifierList>>,
        _ensure_verbal_keyword: bool,
    ) -> usize { ::tsox_core::fntrace::enter("emit_modifier_list"); 
        let Some(modifiers) = modifiers else {
            return node.pos();
        };
        for modifier in &modifiers.list.nodes {
            self.emit_token_node(Some(modifier));
        }
        modifiers
            .list
            .nodes
            .last()
            .map(|m| m.end())
            .unwrap_or_else(|| node.pos())
    }

    pub fn emit_signature(&mut self, node: &Arc<Node>) { ::tsox_core::fntrace::enter("emit_signature"); 
        if let Some(type_parameters) = node.type_parameters() {
            self.emit_type_parameters(node, &type_parameters.nodes);
        }
        self.emit_parameters(node, node_list_slice(node.parameters()));
        self.emit_return_type(node.type_());
    }

    pub fn emit_function_body(&mut self, node: &Arc<Node>) { ::tsox_core::fntrace::enter("emit_function_body"); 
        self.write_space();
        self.emit_block(node);
    }

    pub fn emit_function_body_node(&mut self, node: Option<&Arc<Node>>) { ::tsox_core::fntrace::enter("emit_function_body_node"); 
        if let Some(node) = node {
            self.emit_function_body(node);
        }
    }

    pub fn emit_list(
        &mut self,
        emit_element: fn(&mut Printer, &Arc<Node>),
        context_node: &Arc<Node>,
        nodes: &[Arc<Node>],
        format: ListFormat,
    ) { ::tsox_core::fntrace::enter("emit_list"); 
        let _ = context_node;
        if nodes.is_empty() {
            if format.contains(ListFormat::MULTI_LINE) {
                self.write_line();
            }
            return;
        }
        if format.contains(ListFormat::INDENTED) {
            self.increase_indent_if(true);
        }
        for (i, node) in nodes.iter().enumerate() {
            if format.contains(ListFormat::MULTI_LINE) {
                self.write_line();
            } else if i > 0 && format.contains(ListFormat::SPACE_BETWEEN_SIBLINGS) {
                self.write_space();
            }
            if i > 0 && format.contains(ListFormat::COMMA_DELIMITED) {
                self.write_punctuation(",");
            }
            emit_element(self, node);
        }
        if format.contains(ListFormat::INDENTED) {
            self.decrease_indent_if(true);
        }
    }

    pub fn should_emit_indented(&self, node: &Arc<Node>) -> bool { ::tsox_core::fntrace::enter("should_emit_indented"); 
        let flags = self.emit_context.emit_flags(node);
        (flags & super::m4o::EmitFlags::INDENTED.0) != 0
            && (flags & super::m4o::EmitFlags::NO_INDENTATION.0) == 0
    }

    pub fn should_emit_on_single_line(&self, node: &Arc<Node>) -> bool { ::tsox_core::fntrace::enter("should_emit_on_single_line"); 
        (self.emit_context.emit_flags(node) & super::m4o::EmitFlags::SINGLE_LINE.0) != 0
    }

    pub fn should_emit_indirect_call(&self, node: &Arc<Node>) -> bool { ::tsox_core::fntrace::enter("should_emit_indirect_call"); 
        (self.emit_context.emit_flags(node) & super::m4o::EmitFlags::INDIRECT_CALL.0) != 0
    }

    pub fn is_empty_block(&mut self, node: &Arc<Node>, statements: &[Arc<Node>]) -> bool { ::tsox_core::fntrace::enter("is_empty_block"); 
        let _ = node;
        !self.options.only_print_jsdoc_style && statements.is_empty()
    }

    pub fn get_lines_between_nodes(
        &mut self,
        parent_node: &Arc<Node>,
        node1: &Arc<Node>,
        node2: &Arc<Node>,
    ) -> i32 { ::tsox_core::fntrace::enter("get_lines_between_nodes"); 
        let _ = parent_node;
        let Some(source_file) = self.current_source_file.clone() else {
            return 0;
        };
        if node_is_synthesized(node1) || node_is_synthesized(node2) {
            return 0;
        }
        if positions_are_on_same_line(node1.loc.end() as i64, node2.loc.pos() as i64, &source_file) {
            0
        } else {
            1
        }
    }

    pub fn push_name_generation_scope(&mut self, _node: &Arc<Node>) { ::tsox_core::fntrace::enter("push_name_generation_scope"); }

    pub fn pop_name_generation_scope(&mut self, _node: &Arc<Node>) { ::tsox_core::fntrace::enter("pop_name_generation_scope"); }

    pub fn generate_name_needed(&mut self, _name: Option<&Arc<Node>>) { ::tsox_core::fntrace::enter("generate_name_needed"); }

    pub fn generate_names(&mut self, _node: &Arc<Node>) { ::tsox_core::fntrace::enter("generate_names"); }

    pub fn generate_all_member_names(&mut self, _members: &[Arc<Node>]) { ::tsox_core::fntrace::enter("generate_all_member_names"); }

    pub fn get_unique_helper_name(&mut self, name: &str) -> Arc<Node> { ::tsox_core::fntrace::enter("get_unique_helper_name"); 
        if let Some(existing) = self
            .unique_helper_names
            .as_ref()
            .and_then(|map| map.get(name))
        {
            return existing.clone();
        }
        let helper_name = Arc::new(Node::new(
            SyntaxKind::Identifier,
            NodeData::Identifier(crate::ast::node_data_generated::IdentifierData {
                text: name.to_string(),
            }),
        ));
        if let Some(map) = &mut self.unique_helper_names {
            map.insert(name.to_string(), helper_name.clone());
        }
        helper_name
    }

    pub fn emit_variable_declaration(&mut self, node: &Arc<Node>) { ::tsox_core::fntrace::enter("emit_variable_declaration"); 
        let state = self.enter_node(node);
        let name_end = node.name().map(|n| n.end());
        if name_end.is_some() {
            self.emit_binding_name(node.name());
        }
        self.emit_type_annotation(node.type_());
        if let Some(initializer) = node.initializer() {
            self.emit_initializer(
                Some(initializer),
                name_end.unwrap_or_else(|| node.pos()),
                node,
            );
        }
        self.exit_node(node, state);
    }

    pub fn emit_property_declaration(&mut self, node: &Arc<Node>) { ::tsox_core::fntrace::enter("emit_property_declaration"); 
        let state = self.enter_node(node);
        self.emit_modifier_list(node, node.modifiers(), true);
        self.emit_property_name(node.name());
        self.emit_type_annotation(node.type_());
        let name_end = node.name().map(|n| n.end()).unwrap_or_else(|| node.pos());
        self.emit_initializer(node.initializer(), name_end, node);
        self.write_trailing_semicolon();
        self.exit_node(node, state);
    }

    pub fn emit_method_declaration(&mut self, node: &Arc<Node>) { ::tsox_core::fntrace::enter("emit_method_declaration"); 
        let state = self.enter_node(node);
        self.emit_modifier_list(node, node.modifiers(), true);
        self.emit_property_name(node.name());
        self.emit_signature(node);
        self.emit_function_body_node(node.body());
        self.exit_node(node, state);
    }

    pub fn emit_index_signature(&mut self, node: &Arc<Node>) { ::tsox_core::fntrace::enter("emit_index_signature"); 
        let state = self.enter_node(node);
        self.emit_modifier_list(node, node.modifiers(), true);
        self.write_punctuation("[");
        self.write_punctuation("]");
        self.emit_type_annotation(node.type_());
        self.write_trailing_semicolon();
        self.exit_node(node, state);
    }

    pub fn emit_semicolon_class_element(&mut self, node: &Arc<Node>) { ::tsox_core::fntrace::enter("emit_semicolon_class_element"); 
        let state = self.enter_node(node);
        self.write_trailing_semicolon();
        self.exit_node(node, state);
    }

    pub fn emit_not_emitted_statement(&mut self, node: &Arc<Node>) { ::tsox_core::fntrace::enter("emit_not_emitted_statement"); 
        let state = self.enter_node(node);
        self.exit_node(node, state);
    }

    pub fn emit_type_alias_declaration(&mut self, node: &Arc<Node>) { ::tsox_core::fntrace::enter("emit_type_alias_declaration"); 
        let state = self.enter_node(node);
        self.write_keyword("type");
        self.write_space();
        if let Some(name) = node.name() {
            self.emit_identifier_name(name);
        }
        if let Some(type_parameters) = node.type_parameters() {
            self.emit_type_parameters(node, &type_parameters.nodes);
        }
        self.write_space();
        self.write_punctuation("=");
        self.write_space();
        if let Some(type_node) = node.type_() {
            self.emit_type_node(type_node, TYPE_PRECEDENCE_LOWEST);
        }
        self.write_trailing_semicolon();
        self.exit_node(node, state);
    }

    pub fn emit_object_binding_pattern(&mut self, node: &Arc<Node>) { ::tsox_core::fntrace::enter("emit_object_binding_pattern"); 
        let state = self.enter_node(node);
        self.write_punctuation("{");
        if let Some(elements) = node.elements() {
            for (i, element) in elements.nodes.iter().enumerate() {
                if i > 0 {
                    self.write_punctuation(",");
                    self.write_space();
                }
                self.emit_binding_element(element);
            }
        }
        self.write_punctuation("}");
        self.exit_node(node, state);
    }

    pub fn emit_statement(&mut self, node: &Arc<Node>) { ::tsox_core::fntrace::enter("emit_statement"); 
        match node.kind {
            SyntaxKind::Block => self.emit_block(node),
            SyntaxKind::ContinueStatement => self.emit_continue_statement(node),
            SyntaxKind::BreakStatement => self.emit_break_statement(node),
            SyntaxKind::DebuggerStatement => self.emit_debugger_statement(node),
            SyntaxKind::EmptyStatement => {}
            _ => self.write_as(node.text(), WriteKind::None),
        }
    }

    pub fn emit_heritage_clause_node(&mut self, node: &Arc<Node>) { ::tsox_core::fntrace::enter("emit_heritage_clause_node"); 
        let state = self.enter_node(node);
        let NodeData::HeritageClause(d) = &node.data else {
            self.exit_node(node, state);
            return;
        };
        self.write_space();
        self.write_token_text(d.token, WriteKind::Keyword, node.pos());
        self.write_space();
        for (i, heritage_type) in d.types.nodes.iter().enumerate() {
            if i > 0 {
                self.write_punctuation(",");
                self.write_space();
            }
            self.emit_expression(heritage_type, OPERATOR_PRECEDENCE_LOWEST);
        }
        self.exit_node(node, state);
    }
}
