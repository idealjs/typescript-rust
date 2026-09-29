#![allow(unused_imports)]
#![allow(dead_code)]

use std::sync::Arc;

use super::m4r::*;
use super::m4r_3::NodeR36k16Accessors;
use super::m4r::R39k06NodeAccessors;
use crate::mig::m4p_6::r36k20_jsx_defs::JsxNodeExt;
use crate::mig::m4q::r39k08_defs::EmitContextExtK08;
use tsox_frontend::ast::node_data_generated::{
    is_arrow_function, is_numeric_literal, is_partially_emitted_expression,
    is_type_parameter_declaration, NodeData, PropertyAccessExpressionData,
};
use tsox_frontend::scanner::token_to_string;
use tsox_frontend::scanner::TOKEN_FLAGS_WITH_SPECIFIER;
use tsox_frontend::ast::utilities::is_optional_chain;
use tsox_frontend::ast::mig::m3g_2::is_parse_tree_node;
use tsox_frontend::format::mig::m4o::EmitFlags;
use tsox_frontend::ast::{Node, NodeList, SyntaxKind};

const LF_TEMPLATE_EXPRESSION_SPANS: ListFormat =
    crate::mig::m4q::r33k12_defs::LF_TEMPLATE_EXPRESSION_SPANS;
const LF_IMPORT_ATTRIBUTES: ListFormat = crate::mig::m4q::r33k12_defs::LF_IMPORT_ATTRIBUTES;
const LF_INTERSECTION_TYPE_CONSTITUENTS: ListFormat =
    crate::mig::m4q::r33k12_defs::LF_INTERSECTION_TYPE_CONSTITUENTS;
const TYPE_PRECEDENCE_UNION: TypePrecedence = TypePrecedence::Union;

fn get_type_node_precedence_node(n: &Node) -> TypePrecedence {
    match n.kind {
        SyntaxKind::ConditionalType => TypePrecedence::Conditional,
        SyntaxKind::JSDocOptionalType | SyntaxKind::JSDocVariadicType => TypePrecedence::Jsdoc,
        SyntaxKind::FunctionType | SyntaxKind::ConstructorType => TypePrecedence::Function,
        SyntaxKind::UnionType => TypePrecedence::Union,
        SyntaxKind::IntersectionType => TypePrecedence::Intersection,
        SyntaxKind::TypeOperator => TypePrecedence::TypeOperator,
        SyntaxKind::InferType => {
            let has_constraint = match &n.data {
                NodeData::InferTypeNode(d) => match &d.type_parameter.data {
                    NodeData::TypeParameterDeclaration(tp) => tp.constraint.is_some(),
                    _ => false,
                },
                _ => false,
            };
            if has_constraint {
                TypePrecedence::Function
            } else {
                TypePrecedence::TypeOperator
            }
        }
        SyntaxKind::IndexedAccessType | SyntaxKind::ArrayType | SyntaxKind::OptionalType => {
            TypePrecedence::Postfix
        }
        SyntaxKind::TypeQuery => TypePrecedence::TypeOperator,
        SyntaxKind::AnyKeyword
        | SyntaxKind::UnknownKeyword
        | SyntaxKind::StringKeyword
        | SyntaxKind::NumberKeyword
        | SyntaxKind::BigIntKeyword
        | SyntaxKind::SymbolKeyword
        | SyntaxKind::BooleanKeyword
        | SyntaxKind::UndefinedKeyword
        | SyntaxKind::NeverKeyword
        | SyntaxKind::ObjectKeyword
        | SyntaxKind::IntrinsicKeyword
        | SyntaxKind::VoidKeyword
        | SyntaxKind::JSDocAllType
        | SyntaxKind::JSDocNullableType
        | SyntaxKind::JSDocNonNullableType
        | SyntaxKind::LiteralType
        | SyntaxKind::TypePredicate
        | SyntaxKind::TypeReference
        | SyntaxKind::TypeLiteral
        | SyntaxKind::TupleType
        | SyntaxKind::RestType
        | SyntaxKind::ParenthesizedType
        | SyntaxKind::ThisType
        | SyntaxKind::MappedType
        | SyntaxKind::NamedTupleMember
        | SyntaxKind::TemplateLiteralType
        | SyntaxKind::ImportType
        | SyntaxKind::PropertyAccessExpression
        | SyntaxKind::ExpressionWithTypeArguments => TypePrecedence::NonArray,
        _ => panic!("unhandled TypeNode: {:?}", n.kind),
    }
}

pub trait R36K18MembersExt {
    fn members(&self) -> Option<&Arc<NodeList>>;
}

impl R36K18MembersExt for Node {
    fn members(&self) -> Option<&Arc<NodeList>> {
        match &self.data {
            NodeData::TypeLiteralNode(d) => Some(&d.members),
            NodeData::ClassDeclaration(d) => Some(&d.members),
            NodeData::ClassExpression(d) => Some(&d.members),
            NodeData::InterfaceDeclaration(d) => Some(&d.members),
            NodeData::ModuleDeclaration(d) => d.body.as_ref().map(|body| match &body.data {
                NodeData::ModuleBlock(d) => &d.statements,
                _ => unreachable!("ModuleDeclaration body is not ModuleBlock"),
            }),
            _ => None,
        }
    }
}

pub trait R36K18PrinterExt {
    fn write_operator(&mut self, text: &str);
}

impl R36K18PrinterExt for Printer {
    fn write_operator(&mut self, text: &str) {
        self.write_as(text, WriteKind::Operator);
    }
}

impl Printer {
    pub fn emit_type_parameter(&mut self, node: &Node) {
        let state = self.enter_node(node);
        self.emit_modifier_list(node, node.modifiers().map(|m| &**m), false);
        self.emit_binding_identifier(node.name().expect("TypeParameter name"));
        if let Some(constraint) = node.k06_constraint() {
            self.write_space();
            self.write_keyword("extends");
            self.write_space();
            self.emit_type_node_outside_extends(constraint);
        }
        if let Some(default_type) = node.k06_default_type() {
            self.write_space();
            self.write_operator("=");
            self.write_space();
            self.emit_type_node_outside_extends(default_type);
        }
        self.exit_node(node, state);
    }

    pub fn emit_type_parameter_declaration_node(&mut self, node: &Node) {
        if is_type_parameter_declaration(node) {
            self.emit_type_parameter(node);
        } else {
            self.emit_type_argument(node);
        }
    }

    pub fn emit_type_parameters(&mut self, parent_node: &Node, nodes: Option<&NodeList>) {
        let Some(nodes) = nodes else {
            return;
        };
        let flags = LF_TYPE_PARAMETERS
            | if is_arrow_function(parent_node) {
                LF_ALLOW_TRAILING_COMMA
            } else {
                LF_NONE
            };
        self.emit_list(|p, n| p.emit_type_parameter_declaration_node(n), parent_node, Some(nodes), flags);
    }

    pub fn emit_type_annotation(&mut self, node: Option<&Node>) {
        let Some(node) = node else {
            return;
        };
        self.write_punctuation(":");
        self.write_space();
        self.emit_type_node_outside_extends(node);
    }

    pub fn emit_type_element(&mut self, node: &Node) {
        match node.kind {
            SyntaxKind::PropertySignature => self.emit_property_signature(node),
            SyntaxKind::MethodSignature => self.emit_method_signature(node),
            SyntaxKind::CallSignature => self.emit_call_signature(node),
            SyntaxKind::ConstructSignature => self.emit_construct_signature(node),
            SyntaxKind::GetAccessor => self.emit_get_accessor_declaration(node),
            SyntaxKind::SetAccessor => self.emit_set_accessor_declaration(node),
            SyntaxKind::IndexSignature => self.emit_index_signature(node),
            SyntaxKind::NotEmittedTypeElement => self.emit_not_emitted_type_element(node),
            _ => panic!("unexpected TypeElement: {:?}", node.kind),
        }
    }

    pub fn emit_type_predicate_parameter_name(&mut self, node: &Arc<Node>) {
        match node.kind {
            SyntaxKind::Identifier => self.emit_identifier_reference(node),
            SyntaxKind::ThisType => self.emit_this_type(node),
            _ => panic!("unexpected TypePredicateParameterName: {:?}", node.kind),
        }
    }

    pub fn emit_type_predicate(&mut self, node: &Node) {
        let state = self.enter_node(node);
        if let Some(asserts_modifier) = node.k06_asserts_modifier() {
            self.emit_token_node(Some(asserts_modifier));
            self.write_space();
        }
        let parameter_name = match &node.data {
            NodeData::TypePredicateNode(d) => &d.parameter_name,
            _ => panic!("unexpected TypePredicateNode: {:?}", node.kind),
        };
        self.emit_type_predicate_parameter_name(parameter_name);
        if let Some(ty) = node.ty() {
            self.write_space();
            self.write_keyword("is");
            self.write_space();
            self.emit_type_node_outside_extends(ty);
        }
        self.exit_node(node, state);
    }

    pub fn emit_type_argument(&mut self, node: &Node) {
        self.emit_type_node_outside_extends(node);
    }

    pub fn emit_type_arguments(&mut self, parent_node: &Node, nodes: Option<&NodeList>) {
        let Some(nodes) = nodes else {
            return;
        };
        self.emit_list(
            |p, n| p.emit_type_parameter_declaration_node(n),
            parent_node,
            Some(nodes),
            LF_TYPE_ARGUMENTS,
        );
    }

    pub fn emit_type_reference(&mut self, node: &Node) {
        let state = self.enter_node(node);
        let (type_name, type_arguments) = match &node.data {
            NodeData::TypeReferenceNode(d) => (&d.type_name, d.type_arguments.as_ref()),
            _ => panic!("unexpected TypeReferenceNode: {:?}", node.kind),
        };
        self.emit_entity_name(type_name);
        self.emit_type_arguments(node, type_arguments.map(|l| &**l));
        self.exit_node(node, state);
    }

    pub fn emit_type_query(&mut self, node: &Node) {
        let state = self.enter_node(node);
        self.write_keyword("typeof");
        self.write_space();
        let (expr_name, type_arguments) = match &node.data {
            NodeData::TypeQueryNode(d) => (&d.expr_name, d.type_arguments.as_ref()),
            _ => panic!("unexpected TypeQueryNode: {:?}", node.kind),
        };
        self.emit_entity_name(expr_name);
        self.emit_type_arguments(node, type_arguments.map(|l| &**l));
        self.exit_node(node, state);
    }

    pub fn emit_type_literal(&mut self, node: &Node) {
        let state = self.enter_node(node);
        self.push_name_generation_scope(Some(node));
        self.generate_all_member_names(node.members().map(|l| &**l));
        self.write_punctuation("{");
        let flags = if self.should_emit_on_single_line(node) {
            LF_SINGLE_LINE_TYPE_LITERAL_MEMBERS
        } else {
            LF_MULTI_LINE_TYPE_LITERAL_MEMBERS
        };
        self.emit_list(
            |p, n| p.emit_type_element(n),
            node,
            node.members().map(|l| &**l),
            flags | LF_NO_SPACE_IF_EMPTY,
        );
        self.write_punctuation("}");
        self.pop_name_generation_scope(Some(node));
        self.exit_node(node, state);
    }

    pub fn emit_tuple_element_type(&mut self, node: &Node) {
        self.emit_type_node_outside_extends(node);
    }

    pub fn emit_tuple_type(&mut self, node: &Node) {
        let state = self.enter_node(node);
        self.emit_token(SyntaxKind::OpenBracketToken, node.pos(), WriteKind::Punctuation, node);
        let flags = if self.should_emit_on_single_line(node) {
            LF_SINGLE_LINE_TUPLE_TYPE_ELEMENTS
        } else {
            LF_MULTI_LINE_TUPLE_TYPE_ELEMENTS
        };
        let elements = match &node.data {
            NodeData::TupleTypeNode(d) => &d.elements,
            _ => panic!("unexpected TupleTypeNode: {:?}", node.kind),
        };
        let elements_end = elements.end();
        self.emit_list(
            |p, n| p.emit_tuple_element_type(n),
            node,
            Some(&**elements),
            flags | LF_NO_SPACE_IF_EMPTY,
        );
        self.emit_token(SyntaxKind::CloseBracketToken, elements_end, WriteKind::Punctuation, node);
        self.exit_node(node, state);
    }

    pub fn emit_union_type_constituent(&mut self, node: &Node) {
        self.emit_type_node(node, TYPE_PRECEDENCE_TYPE_OPERATOR);
    }

    pub fn emit_union_type(&mut self, node: &Node) {
        let state = self.enter_node(node);
        self.emit_list(
            |p, n| p.emit_union_type_constituent(n),
            node,
            Some(node.k06_types()),
            LF_UNION_TYPE_CONSTITUENTS,
        );
        self.exit_node(node, state);
    }

    pub fn emit_type_operator(&mut self, node: &Node) {
        let state = self.enter_node(node);
        self.emit_token(node.k06_operator(), node.pos(), WriteKind::Keyword, node);
        self.write_space();
        let precedence = if node.k06_operator() == SyntaxKind::ReadonlyKeyword {
            TYPE_PRECEDENCE_POSTFIX
        } else {
            TYPE_PRECEDENCE_TYPE_OPERATOR
        };
        if let Some(ty) = node.ty() {
            self.emit_type_node(ty, precedence);
        }
        self.exit_node(node, state);
    }

    pub fn emit_type_node_in_extends(&mut self, node: &Node) {
        let saved_in_extends = self.in_extends;
        self.in_extends = true;
        self.emit_type_node_preserving_extends(node, TYPE_PRECEDENCE_LOWEST);
        self.in_extends = saved_in_extends;
    }

    pub fn emit_type_node_outside_extends(&mut self, node: &Node) {
        let saved_in_extends = self.in_extends;
        self.in_extends = false;
        self.emit_type_node_preserving_extends(node, TYPE_PRECEDENCE_LOWEST);
        self.in_extends = saved_in_extends;
    }

    pub fn emit_type_node_preserving_extends(&mut self, node: &Node, precedence: TypePrecedence) {
        self.emit_type_node(node, precedence);
    }

    pub fn emit_type_node(&mut self, node: &Node, precedence: TypePrecedence) {
        let mut precedence = precedence;
        if self.in_extends && precedence <= TYPE_PRECEDENCE_CONDITIONAL {
            precedence = TYPE_PRECEDENCE_FUNCTION;
        }

        let saved_in_extends = self.in_extends;
        let parens = get_type_node_precedence_node(node) < precedence;
        if parens {
            self.in_extends = false;
            self.write_punctuation("(");
        }

        match node.kind {
            SyntaxKind::AnyKeyword
            | SyntaxKind::UnknownKeyword
            | SyntaxKind::NumberKeyword
            | SyntaxKind::BigIntKeyword
            | SyntaxKind::ObjectKeyword
            | SyntaxKind::BooleanKeyword
            | SyntaxKind::StringKeyword
            | SyntaxKind::SymbolKeyword
            | SyntaxKind::VoidKeyword
            | SyntaxKind::UndefinedKeyword
            | SyntaxKind::NeverKeyword
            | SyntaxKind::IntrinsicKeyword => self.emit_keyword_type_node(node),

            SyntaxKind::TypePredicate => self.emit_type_predicate(node),
            SyntaxKind::TypeReference => self.emit_type_reference(node),
            SyntaxKind::FunctionType => self.emit_function_type(node),
            SyntaxKind::ConstructorType => self.emit_constructor_type(node),
            SyntaxKind::TypeQuery => self.emit_type_query(node),
            SyntaxKind::TypeLiteral => self.emit_type_literal(node),
            SyntaxKind::ArrayType => self.emit_array_type(node),
            SyntaxKind::TupleType => self.emit_tuple_type(node),
            SyntaxKind::OptionalType => self.emit_optional_type(node),
            SyntaxKind::RestType => self.emit_rest_type(node),
            SyntaxKind::UnionType => self.emit_union_type(node),
            SyntaxKind::IntersectionType => self.emit_intersection_type(node),
            SyntaxKind::ConditionalType => self.emit_conditional_type(node),
            SyntaxKind::InferType => self.emit_infer_type(node),
            SyntaxKind::ParenthesizedType => self.emit_parenthesized_type(node),
            SyntaxKind::ThisType => self.emit_this_type(node),
            SyntaxKind::TypeOperator => self.emit_type_operator(node),
            SyntaxKind::IndexedAccessType => self.emit_indexed_access_type(node),
            SyntaxKind::MappedType => self.emit_mapped_type(node),
            SyntaxKind::LiteralType => self.emit_literal_type(node),
            SyntaxKind::NamedTupleMember => self.emit_named_tuple_member(node),
            SyntaxKind::TemplateLiteralType => self.emit_template_type(node),
            SyntaxKind::TemplateLiteralTypeSpan => self.emit_template_type_span(node),
            SyntaxKind::ImportType => self.emit_import_type_node(node),

            SyntaxKind::PropertyAccessExpression => self.emit_property_access_expression_node(node),
            SyntaxKind::ExpressionWithTypeArguments => self.emit_expression_with_type_arguments(node),

            SyntaxKind::JSDocAllType => self.emit_jsdoc_all_type(node),
            SyntaxKind::JSDocNonNullableType => self.emit_jsdoc_non_nullable_type(node),
            SyntaxKind::JSDocNullableType => self.emit_jsdoc_nullable_type(node),
            SyntaxKind::JSDocOptionalType => self.emit_jsdoc_optional_type(node),
            SyntaxKind::JSDocVariadicType => self.emit_jsdoc_variadic_type(node),

            _ => panic!("unhandled TypeNode: {:?}", node.kind),
        }

        if parens {
            self.write_punctuation(")");
        }

        self.in_extends = saved_in_extends;
    }

    pub fn may_need_dot_dot_for_property_access(&mut self, expression: &Arc<Node>) -> bool {
        let mut expression = Arc::clone(expression);
        while is_partially_emitted_expression(&expression) {
            let original = self.emit_context.most_original(&expression);
            if Arc::ptr_eq(&original, &expression) {
                break;
            }
            expression = original;
        }
        if is_numeric_literal(&expression) {
            let text = self.get_literal_text_of_node(&expression, None, GET_LITERAL_TEXT_FLAGS_NEVER_ASCII_ESCAPE);
            let token_flags = match &expression.data {
                NodeData::NumericLiteral(d) => d.token_flags,
                NodeData::BigIntLiteral(d) => d.token_flags,
                _ => 0,
            };
            return token_flags & TOKEN_FLAGS_WITH_SPECIFIER == 0
                && !text.contains(token_to_string(SyntaxKind::DotToken))
                && !text.contains('E')
                && !text.contains('e');
        }
        false
    }

    pub fn emit_type_assertion_expression(&mut self, node: &Node) {
        let state = self.enter_node(node);
        self.write_punctuation("<");
        if let Some(ty) = node.ty() {
            self.emit_type_node_outside_extends(ty);
        }
        self.write_punctuation(">");
        self.emit_expression(
            node.expression().expect("TypeAssertion expression"),
            OPERATOR_PRECEDENCE_UPDATE,
        );
        self.exit_node(node, state);
    }

    pub fn emit_type_of_expression(&mut self, node: &Node) {
        let state = self.enter_node(node);
        self.emit_token(SyntaxKind::TypeOfKeyword, node.pos(), WriteKind::Keyword, node);
        self.write_space();
        self.emit_expression(
            node.expression().expect("TypeOfExpression expression"),
            OPERATOR_PRECEDENCE_UNARY,
        );
        self.exit_node(node, state);
    }

    pub fn emit_void_expression(&mut self, node: &Node) {
        let state = self.enter_node(node);
        self.emit_token(SyntaxKind::VoidKeyword, node.pos(), WriteKind::Keyword, node);
        self.write_space();
        self.emit_expression(
            node.expression().expect("VoidExpression expression"),
            OPERATOR_PRECEDENCE_UNARY,
        );
        self.exit_node(node, state);
    }

    pub fn emit_property_access_expression_node(&mut self, node: &Node) {
        let state = self.enter_node(node);
        let data: &PropertyAccessExpressionData = node.property_access_expression();
        let precedence = if is_optional_chain(node) {
            OperatorPrecedence::OptionalChain
        } else {
            OperatorPrecedence::Member
        };
        self.emit_expression(&data.expression, precedence);
        let token = match &data.question_dot_token {
            Some(token) => Arc::clone(token),
            None => {
                let token = Arc::new(Node::with_loc(
                    SyntaxKind::DotToken,
                    NodeData::Token,
                    tsox_core::core::text::TextRange::new(data.expression.end(), data.name.pos()),
                ));
                self.emit_context
                    .add_emit_flags(&token, EmitFlags::NO_SOURCE_MAP);
                token
            }
        };
        let lines_before_dot = self.get_lines_between_nodes(node, &data.expression, &token);
        self.write_line_repeat(lines_before_dot);
        self.increase_indent_if(lines_before_dot > 0);
        let should_emit_dot_dot = token.kind != SyntaxKind::QuestionDotToken
            && self.may_need_dot_dot_for_property_access(&data.expression)
            && !self.writer.has_trailing_comment()
            && !self.writer.has_trailing_whitespace();
        if should_emit_dot_dot {
            self.write_punctuation(".");
        }
        if data.question_dot_token.is_some() {
            self.emit_token_node(Some(&token));
        } else {
            self.emit_token(
                SyntaxKind::DotToken,
                data.expression.end(),
                WriteKind::Punctuation,
                node,
            );
        }
        let lines_after_dot = self.get_lines_between_nodes(node, &token, &data.name);
        self.write_line_repeat(lines_after_dot);
        self.increase_indent_if(lines_after_dot > 0);
        self.emit_member_name(&data.name);
        self.decrease_indent_if(lines_after_dot > 0);
        self.decrease_indent_if(lines_before_dot > 0);
        self.exit_node(node, state);
    }
}

impl Printer {
    pub fn emit_identifier_reference(&mut self, node: &Arc<Node>) {
        let state = self.enter_node(node);
        let text = self.get_text_of_node(node, false);
        self.write(&text);
        self.exit_node(node, state);
    }

    pub fn emit_entity_name(&mut self, node: &Arc<Node>) {
        match node.kind {
            SyntaxKind::Identifier => self.emit_identifier_reference(node),
            SyntaxKind::QualifiedName => self.emit_qualified_name(node),
            SyntaxKind::PropertyAccessExpression => {
                self.emit_expression(node, OPERATOR_PRECEDENCE_DISALLOW_COMMA);
            }
            _ => panic!("unexpected EntityName: {:?}", node.kind),
        }
    }

    pub fn emit_qualified_name(&mut self, node: &Arc<Node>) {
        let state = self.enter_node(node);
        let (left, right) = match &node.data {
            NodeData::QualifiedName(d) => (&d.left, &d.right),
            _ => panic!("unexpected QualifiedName: {:?}", node.kind),
        };
        self.emit_entity_name(left);
        self.write_punctuation(".");
        self.emit_member_name(right);
        self.exit_node(node, state);
    }

    pub fn emit_keyword_type_node(&mut self, node: &Node) {
        self.emit_keyword_node(Some(node));
    }

    pub fn emit_this_type(&mut self, node: &Node) {
        let state = self.enter_node(node);
        self.write_keyword("this");
        self.exit_node(node, state);
    }

    pub fn emit_function_type(&mut self, node: &Node) {
        let state = self.enter_node(node);
        let indented = self.should_emit_indented(node);
        self.increase_indent_if(indented);
        self.push_name_generation_scope(Some(node));
        self.emit_signature(node);
        self.write_space();
        let type_node = match &node.data {
            NodeData::FunctionTypeNode(d) => d.type_node.as_deref(),
            _ => panic!("unexpected FunctionTypeNode: {:?}", node.kind),
        };
        self.emit_return_type(type_node);
        self.pop_name_generation_scope(Some(node));
        self.decrease_indent_if(indented);
        self.exit_node(node, state);
    }

    pub fn emit_constructor_type(&mut self, node: &Node) {
        let state = self.enter_node(node);
        let modifiers = match &node.data {
            NodeData::ConstructorTypeNode(d) => d.modifiers.as_deref(),
            _ => panic!("unexpected ConstructorTypeNode: {:?}", node.kind),
        };
        self.emit_modifier_list(node, modifiers, false);
        self.write_keyword("new");
        self.write_space();
        let indented = self.should_emit_indented(node);
        self.increase_indent_if(indented);
        self.push_name_generation_scope(Some(node));
        self.emit_signature(node);
        self.write_space();
        let type_node = match &node.data {
            NodeData::ConstructorTypeNode(d) => d.type_node.as_deref(),
            _ => panic!("unexpected ConstructorTypeNode: {:?}", node.kind),
        };
        self.emit_return_type(type_node);
        self.pop_name_generation_scope(Some(node));
        self.decrease_indent_if(indented);
        self.exit_node(node, state);
    }

    pub fn emit_postfix_type_operand(&mut self, operand: &Arc<Node>, parent: &Node) {
        if is_parse_tree_node(parent) && operand.kind == SyntaxKind::TypeQuery {
            self.emit_type_node(operand, TYPE_PRECEDENCE_TYPE_OPERATOR);
            return;
        }
        self.emit_type_node(operand, TYPE_PRECEDENCE_POSTFIX);
    }

    pub fn emit_array_type(&mut self, node: &Node) {
        let state = self.enter_node(node);
        let element_type = match &node.data {
            NodeData::ArrayTypeNode(d) => &d.element_type,
            _ => panic!("unexpected ArrayTypeNode: {:?}", node.kind),
        };
        self.emit_postfix_type_operand(element_type, node);
        self.write_punctuation("[");
        self.write_punctuation("]");
        self.exit_node(node, state);
    }

    pub fn emit_optional_type(&mut self, node: &Node) {
        let state = self.enter_node(node);
        let type_node = match &node.data {
            NodeData::OptionalTypeNode(d) => &d.type_node,
            _ => panic!("unexpected OptionalTypeNode: {:?}", node.kind),
        };
        self.emit_postfix_type_operand(type_node, node);
        self.write_punctuation("?");
        self.exit_node(node, state);
    }

    pub fn emit_rest_type(&mut self, node: &Node) {
        let state = self.enter_node(node);
        let type_node = match &node.data {
            NodeData::RestTypeNode(d) => &d.type_node,
            _ => panic!("unexpected RestTypeNode: {:?}", node.kind),
        };
        self.write_punctuation("...");
        self.emit_type_node_outside_extends(type_node);
        self.exit_node(node, state);
    }

    pub fn emit_intersection_type(&mut self, node: &Node) {
        let state = self.enter_node(node);
        self.emit_list(
            |p, n| p.emit_intersection_type_constituent(n),
            node,
            Some(node.k06_types()),
            LF_INTERSECTION_TYPE_CONSTITUENTS,
        );
        self.exit_node(node, state);
    }

    pub fn emit_intersection_type_constituent(&mut self, node: &Node) {
        self.emit_type_node(node, TYPE_PRECEDENCE_TYPE_OPERATOR);
    }

    pub fn emit_conditional_type(&mut self, node: &Node) {
        let state = self.enter_node(node);
        let (check_type, extends_type, true_type, false_type) = match &node.data {
            NodeData::ConditionalTypeNode(d) => (&d.check_type, &d.extends_type, &d.true_type, &d.false_type),
            _ => panic!("unexpected ConditionalTypeNode: {:?}", node.kind),
        };
        self.emit_type_node(check_type, TYPE_PRECEDENCE_UNION);
        self.write_space();
        self.write_keyword("extends");
        self.write_space();
        self.emit_type_node_in_extends(extends_type);
        self.write_space();
        self.write_punctuation("?");
        self.write_space();
        self.emit_type_node_outside_extends(true_type);
        self.write_space();
        self.write_punctuation(":");
        self.write_space();
        self.emit_type_node_outside_extends(false_type);
        self.exit_node(node, state);
    }

    pub fn emit_infer_type(&mut self, node: &Node) {
        let state = self.enter_node(node);
        let type_parameter = match &node.data {
            NodeData::InferTypeNode(d) => &d.type_parameter,
            _ => panic!("unexpected InferTypeNode: {:?}", node.kind),
        };
        self.write_keyword("infer");
        self.write_space();
        self.emit_infer_type_parameter(type_parameter);
        self.exit_node(node, state);
    }

    pub fn emit_infer_type_parameter(&mut self, node: &Arc<Node>) {
        let state = self.enter_node(node);
        let (name, constraint) = match &node.data {
            NodeData::TypeParameterDeclaration(d) => (&d.name, d.constraint.as_deref()),
            _ => panic!("unexpected TypeParameterDeclaration: {:?}", node.kind),
        };
        self.emit_binding_identifier(name);
        if let Some(constraint) = constraint {
            self.write_space();
            self.write_keyword("extends");
            self.write_space();
            self.emit_type_node_in_extends(constraint);
        }
        self.exit_node(node, state);
    }

    pub fn emit_parenthesized_type(&mut self, node: &Node) {
        let state = self.enter_node(node);
        let type_node = match &node.data {
            NodeData::ParenthesizedTypeNode(d) => &d.type_node,
            _ => panic!("unexpected ParenthesizedTypeNode: {:?}", node.kind),
        };
        self.write_punctuation("(");
        self.emit_type_node_outside_extends(type_node);
        self.write_punctuation(")");
        self.exit_node(node, state);
    }

    pub fn emit_indexed_access_type(&mut self, node: &Node) {
        let state = self.enter_node(node);
        let (object_type, index_type) = match &node.data {
            NodeData::IndexedAccessTypeNode(d) => (&d.object_type, &d.index_type),
            _ => panic!("unexpected IndexedAccessTypeNode: {:?}", node.kind),
        };
        self.emit_postfix_type_operand(object_type, node);
        self.write_punctuation("[");
        self.emit_type_node_outside_extends(index_type);
        self.write_punctuation("]");
        self.exit_node(node, state);
    }

    pub fn emit_mapped_type_parameter(&mut self, node: &Arc<Node>) {
        let state = self.enter_node(node);
        let (name, constraint) = match &node.data {
            NodeData::TypeParameterDeclaration(d) => (&d.name, d.constraint.as_deref()),
            _ => panic!("unexpected TypeParameterDeclaration: {:?}", node.kind),
        };
        self.emit_binding_identifier(name);
        self.write_space();
        self.write_keyword("in");
        self.write_space();
        self.emit_type_node_outside_extends(constraint.expect("mapped type constraint"));
        self.exit_node(node, state);
    }

    pub fn emit_mapped_type(&mut self, node: &Node) {
        let state = self.enter_node(node);
        let (readonly_token, type_parameter, name_type, question_token, type_node, members) =
            match &node.data {
                NodeData::MappedTypeNode(d) => (
                    d.readonly_token.as_deref(),
                    &d.type_parameter,
                    d.name_type.as_deref(),
                    d.question_token.as_deref(),
                    d.type_node.as_deref(),
                    &d.members,
                ),
                _ => panic!("unexpected MappedTypeNode: {:?}", node.kind),
            };
        let single_line = self.should_emit_on_single_line(node);
        self.write_punctuation("{");
        if single_line {
            self.write_space();
        } else {
            self.write_line();
            self.increase_indent();
        }
        if let Some(readonly_token) = readonly_token {
            self.emit_token_node(Some(readonly_token));
            if readonly_token.kind != SyntaxKind::ReadonlyKeyword {
                self.write_keyword("readonly");
            }
            self.write_space();
        }
        self.write_punctuation("[");
        self.emit_mapped_type_parameter(type_parameter);
        if let Some(name_type) = name_type {
            self.write_space();
            self.write_keyword("as");
            self.write_space();
            self.emit_type_node_outside_extends(name_type);
        }
        self.write_punctuation("]");
        if let Some(question_token) = question_token {
            self.emit_punctuation_node(Some(question_token));
            if question_token.kind != SyntaxKind::QuestionToken {
                self.write_punctuation("?");
            }
        }
        if let Some(type_node) = type_node {
            self.write_punctuation(":");
            self.write_space();
            self.emit_type_node_outside_extends(type_node);
        }
        self.write_trailing_semicolon();
        if let Some(members) = members {
            if !members.nodes.is_empty() {
                if single_line {
                    self.write_space();
                } else {
                    self.write_line();
                }
                self.emit_list(
                    |p, n| p.emit_type_element(n),
                    node,
                    Some(&**members),
                    LF_PRESERVE_LINES,
                );
            }
        }
        if single_line {
            self.write_space();
        } else {
            self.write_line();
            self.decrease_indent();
        }
        self.write_punctuation("}");
        self.exit_node(node, state);
    }

    pub fn emit_literal_type(&mut self, node: &Node) {
        let state = self.enter_node(node);
        let literal = match &node.data {
            NodeData::LiteralTypeNode(d) => &d.literal,
            _ => panic!("unexpected LiteralTypeNode: {:?}", node.kind),
        };
        self.emit_expression(literal, OperatorPrecedence::Comma);
        self.exit_node(node, state);
    }

    pub fn emit_named_tuple_member(&mut self, node: &Node) {
        let state = self.enter_node(node);
        let (dot_dot_dot_token, name, question_token, type_node) = match &node.data {
            NodeData::NamedTupleMember(d) => (
                d.dot_dot_dot_token.as_deref(),
                &d.name,
                d.question_token.as_deref(),
                &d.type_node,
            ),
            _ => panic!("unexpected NamedTupleMember: {:?}", node.kind),
        };
        self.emit_punctuation_node(dot_dot_dot_token);
        self.emit_identifier_name(name);
        self.emit_punctuation_node(question_token);
        let mut colon_pos = name.end();
        if let Some(question_token) = question_token {
            colon_pos = colon_pos.max(question_token.end());
        }
        self.emit_token(SyntaxKind::ColonToken, colon_pos, WriteKind::Punctuation, node);
        self.write_space();
        self.emit_type_node_outside_extends(type_node);
        self.exit_node(node, state);
    }

    pub fn emit_template_type_span_node(&mut self, node: &Node) {
        self.emit_template_type_span(node);
    }

    pub fn emit_template_type_span(&mut self, node: &Node) {
        let state = self.enter_node(node);
        let (type_node, literal) = match &node.data {
            NodeData::TemplateLiteralTypeSpan(d) => (&d.type_node, &d.literal),
            _ => panic!("unexpected TemplateLiteralTypeSpan: {:?}", node.kind),
        };
        self.emit_type_node_outside_extends(type_node);
        self.emit_template_middle_tail(literal);
        self.exit_node(node, state);
    }

    pub fn emit_template_head(&mut self, node: &Arc<Node>) {
        let state = self.enter_node(node);
        self.emit_literal(node, GET_LITERAL_TEXT_FLAGS_NONE);
        self.exit_node(node, state);
    }

    pub fn emit_template_middle(&mut self, node: &Arc<Node>) {
        let state = self.enter_node(node);
        self.emit_literal(node, GET_LITERAL_TEXT_FLAGS_NONE);
        self.exit_node(node, state);
    }

    pub fn emit_template_tail(&mut self, node: &Arc<Node>) {
        let state = self.enter_node(node);
        self.emit_literal(node, GET_LITERAL_TEXT_FLAGS_NONE);
        self.exit_node(node, state);
    }

    pub fn emit_template_middle_tail(&mut self, node: &Arc<Node>) {
        match node.kind {
            SyntaxKind::TemplateMiddle => self.emit_template_middle(node),
            SyntaxKind::TemplateTail => self.emit_template_tail(node),
            _ => {}
        }
    }

    pub fn emit_template_type(&mut self, node: &Node) {
        let state = self.enter_node(node);
        let (head, template_spans) = match &node.data {
            NodeData::TemplateLiteralTypeNode(d) => (&d.head, &d.template_spans),
            _ => panic!("unexpected TemplateLiteralTypeNode: {:?}", node.kind),
        };
        self.emit_template_head(head);
        self.emit_list(
            |p, n| p.emit_template_type_span_node(n),
            node,
            Some(template_spans),
            LF_TEMPLATE_EXPRESSION_SPANS,
        );
        self.exit_node(node, state);
    }

    pub fn emit_import_type_node(&mut self, node: &Node) {
        let state = self.enter_node(node);
        let (is_type_of, argument, attributes, qualifier, type_arguments) = match &node.data {
            NodeData::ImportTypeNode(d) => (
                d.is_type_of,
                &d.argument,
                &d.attributes,
                d.qualifier.as_ref(),
                d.type_arguments.as_ref(),
            ),
            _ => panic!("unexpected ImportTypeNode: {:?}", node.kind),
        };
        if is_type_of {
            self.write_keyword("typeof");
            self.write_space();
        }
        self.write_keyword("import");
        self.write_punctuation("(");
        self.emit_type_node_outside_extends(argument);
        if let Some(attributes) = attributes {
            self.write_punctuation(",");
            self.write_space();
            self.emit_import_type_node_attributes(attributes);
        }
        self.write_punctuation(")");
        if let Some(qualifier) = qualifier {
            self.write_punctuation(".");
            self.emit_entity_name(qualifier);
        }
        self.emit_type_arguments(node, type_arguments.map(|l| &**l));
        self.exit_node(node, state);
    }

    pub fn emit_import_type_node_attributes(&mut self, node: &Node) {
        let state = self.enter_node(node);
        let (token, attributes) = match &node.data {
            NodeData::ImportAttributes(d) => (d.token, &d.attributes),
            _ => panic!("unexpected ImportAttributes: {:?}", node.kind),
        };
        self.write_punctuation("{");
        self.write_space();
        self.write_keyword(if token == SyntaxKind::AssertKeyword { "assert" } else { "with" });
        self.write_punctuation(":");
        self.write_space();
        self.emit_list(
            |p, n| p.emit_import_attribute_node(n),
            node,
            Some(attributes),
            LF_IMPORT_ATTRIBUTES,
        );
        self.write_space();
        self.write_punctuation("}");
        self.exit_node(node, state);
    }

    pub fn emit_import_attribute_node(&mut self, node: &Node) {
        let state = self.enter_node(node);
        let (name, value) = match &node.data {
            NodeData::ImportAttribute(d) => (&d.name, &d.value),
            _ => panic!("unexpected ImportAttribute: {:?}", node.kind),
        };
        self.emit_property_name(Some(name));
        self.write_punctuation(":");
        self.write_space();
        self.emit_string_literal(value);
        self.exit_node(node, state);
    }

    pub fn emit_expression_with_type_arguments(&mut self, node: &Node) {
        let state = self.enter_node(node);
        let (expression, type_arguments) = match &node.data {
            NodeData::ExpressionWithTypeArguments(d) => (&d.expression, d.type_arguments.as_ref()),
            _ => panic!("unexpected ExpressionWithTypeArguments: {:?}", node.kind),
        };
        self.emit_expression(expression, OperatorPrecedence::Member);
        self.emit_type_arguments(node, type_arguments.map(|l| &**l));
        self.exit_node(node, state);
    }

    pub fn emit_jsdoc_all_type(&mut self, node: &Node) {
        self.emit_keyword_node(Some(node));
    }

    pub fn emit_jsdoc_non_nullable_type(&mut self, node: &Node) {
        let state = self.enter_node(node);
        let type_node = match &node.data {
            NodeData::JSDocNonNullableType(d) => &d.type_node,
            _ => panic!("unexpected JSDocNonNullableType: {:?}", node.kind),
        };
        self.write_punctuation("!");
        self.emit_type_node(type_node, TypePrecedence::NonArray);
        self.exit_node(node, state);
    }

    pub fn emit_jsdoc_nullable_type(&mut self, node: &Node) {
        let state = self.enter_node(node);
        let type_node = match &node.data {
            NodeData::JSDocNullableType(d) => &d.type_node,
            _ => panic!("unexpected JSDocNullableType: {:?}", node.kind),
        };
        self.write_punctuation("?");
        self.emit_type_node(type_node, TypePrecedence::NonArray);
        self.exit_node(node, state);
    }

    pub fn emit_jsdoc_optional_type(&mut self, node: &Node) {
        let state = self.enter_node(node);
        let type_node = match &node.data {
            NodeData::JSDocOptionalType(d) => &d.type_node,
            _ => panic!("unexpected JSDocOptionalType: {:?}", node.kind),
        };
        self.emit_type_node(type_node, TypePrecedence::Jsdoc);
        self.write_punctuation("=");
        self.exit_node(node, state);
    }

    pub fn emit_jsdoc_variadic_type(&mut self, node: &Node) {
        let state = self.enter_node(node);
        let type_node = match &node.data {
            NodeData::JSDocVariadicType(d) => &d.type_node,
            _ => panic!("unexpected JSDocVariadicType: {:?}", node.kind),
        };
        self.write_punctuation("...");
        self.emit_type_node(type_node, TypePrecedence::Jsdoc);
        self.exit_node(node, state);
    }

    pub fn emit_property_signature(&mut self, node: &Node) {
        let state = self.enter_node(node);
        let (modifiers, name, postfix_token, type_node) = match &node.data {
            NodeData::PropertySignatureDeclaration(d) => (
                d.modifiers.as_deref(),
                &d.name,
                d.postfix_token.as_deref(),
                &d.type_node,
            ),
            _ => panic!("unexpected PropertySignatureDeclaration: {:?}", node.kind),
        };
        self.emit_modifier_list(node, modifiers, false);
        self.emit_property_name(Some(name));
        self.emit_token_node(postfix_token);
        self.emit_type_annotation(Some(&**type_node));
        self.write_trailing_semicolon();
        self.exit_node(node, state);
    }

    pub fn emit_method_signature(&mut self, node: &Node) {
        let state = self.enter_node(node);
        let (modifiers, name, postfix_token) = match &node.data {
            NodeData::MethodSignatureDeclaration(d) => {
                (d.modifiers.as_deref(), &d.name, d.postfix_token.as_deref())
            }
            _ => panic!("unexpected MethodSignatureDeclaration: {:?}", node.kind),
        };
        self.emit_modifier_list(node, modifiers, false);
        self.emit_property_name(Some(name));
        self.emit_token_node(postfix_token);
        let indented = self.should_emit_indented(node);
        self.increase_indent_if(indented);
        self.push_name_generation_scope(Some(node));
        self.emit_signature(node);
        self.write_trailing_semicolon();
        self.pop_name_generation_scope(Some(node));
        self.decrease_indent_if(indented);
        self.exit_node(node, state);
    }

    pub fn emit_call_signature(&mut self, node: &Node) {
        let state = self.enter_node(node);
        let indented = self.should_emit_indented(node);
        self.increase_indent_if(indented);
        self.push_name_generation_scope(Some(node));
        self.emit_signature(node);
        self.write_trailing_semicolon();
        self.pop_name_generation_scope(Some(node));
        self.decrease_indent_if(indented);
        self.exit_node(node, state);
    }

    pub fn emit_construct_signature(&mut self, node: &Node) {
        let state = self.enter_node(node);
        self.write_keyword("new");
        self.write_space();
        let indented = self.should_emit_indented(node);
        self.increase_indent_if(indented);
        self.push_name_generation_scope(Some(node));
        self.emit_signature(node);
        self.write_trailing_semicolon();
        self.pop_name_generation_scope(Some(node));
        self.decrease_indent_if(indented);
        self.exit_node(node, state);
    }

    pub fn emit_accessor_declaration(&mut self, token: SyntaxKind, node: &Node) {
        let state = self.enter_node(node);
        let (modifiers, name, body) = match &node.data {
            NodeData::GetAccessorDeclaration(d) => (d.modifiers.as_deref(), &d.name, d.body.as_deref()),
            NodeData::SetAccessorDeclaration(d) => (d.modifiers.as_deref(), &d.name, d.body.as_deref()),
            _ => panic!("unexpected AccessorDeclaration: {:?}", node.kind),
        };
        let pos = self.emit_modifier_list(node, modifiers, true);
        self.emit_token(token, pos, WriteKind::Keyword, node);
        self.write_space();
        self.emit_property_name(Some(name));
        let indented = self.should_emit_indented(node);
        self.increase_indent_if(indented);
        self.push_name_generation_scope(Some(node));
        self.emit_signature(node);
        match body {
            None => self.write_trailing_semicolon(),
            Some(_) => panic!("accessor body emission awaits statement cluster (r40-k04)"),
        }
        self.pop_name_generation_scope(Some(node));
        self.decrease_indent_if(indented);
        self.exit_node(node, state);
    }

    pub fn emit_get_accessor_declaration(&mut self, node: &Node) {
        self.emit_accessor_declaration(SyntaxKind::GetKeyword, node);
    }

    pub fn emit_set_accessor_declaration(&mut self, node: &Node) {
        self.emit_accessor_declaration(SyntaxKind::SetKeyword, node);
    }

    pub fn emit_index_signature(&mut self, node: &Node) {
        let state = self.enter_node(node);
        let (modifiers, parameters, type_node) = match &node.data {
            NodeData::IndexSignatureDeclaration(d) => {
                (d.modifiers.as_deref(), &d.parameters, &d.type_node)
            }
            _ => panic!("unexpected IndexSignatureDeclaration: {:?}", node.kind),
        };
        self.emit_modifier_list(node, modifiers, false);
        let indented = self.should_emit_indented(node);
        self.increase_indent_if(indented);
        self.push_name_generation_scope(Some(node));
        self.emit_parameters_for_index_signature(node, parameters);
        self.emit_type_annotation(Some(&**type_node));
        self.write_trailing_semicolon();
        self.pop_name_generation_scope(Some(node));
        self.decrease_indent_if(indented);
        self.exit_node(node, state);
    }

    pub fn emit_not_emitted_type_element(&mut self, node: &Node) {
        let state = self.enter_node(node);
        self.exit_node(node, state);
    }
}
