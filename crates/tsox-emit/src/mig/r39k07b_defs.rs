#![allow(dead_code, unused_imports, unused_variables)]

use std::sync::Arc;

use tsox_frontend::ast::node::{Node, NodeList};
use tsox_frontend::ast::node_data_generated::NodeData;
use tsox_frontend::ast::node_source_file::ScriptKind;
use tsox_frontend::ast::syntax_kind_generated::SyntaxKind;
use tsox_frontend::ast::utilities::is_member_name;

use tsox_frontend::ast::mig::m3e_3::OperatorPrecedence;
use crate::printer::generated_identifier_flags::GeneratedIdentifierFlags;
use crate::mig::m4q::r33k12_defs::{
    EmitFlags, ListFlags, PrinterState, TokenEmitFlags, TypePrecedence, WriteKind,
    LF_ALLOW_TRAILING_COMMA, LF_IMPORT_ATTRIBUTES, LF_INTERSECTION_TYPE_CONSTITUENTS,
    LF_MULTI_LINE, LF_MULTI_LINE_TUPLE_TYPE_ELEMENTS, LF_MULTI_LINE_TYPE_LITERAL_MEMBERS,
    LF_NO_SPACE_IF_EMPTY, LF_NO_TRAILING_NEW_LINE, LF_NONE, LF_PREFER_NEW_LINE, LF_PRESERVE_LINES,
    LF_SINGLE_LINE, LF_SINGLE_LINE_TUPLE_TYPE_ELEMENTS, LF_SINGLE_LINE_TYPE_LITERAL_MEMBERS,
    LF_TEMPLATE_EXPRESSION_SPANS, LF_TYPE_ARGUMENTS, LF_TYPE_PARAMETERS,
    LF_UNION_TYPE_CONSTITUENTS,
};
use crate::mig::m4q::r39k08_defs::{EmitContextExtK08, PrinterExtK08};
use super::r39k07_defs::get_type_node_precedence07;
use crate::mig::m4q::Printer;

impl Printer {
    pub(crate) fn emit_type_annotation(&mut self, node: Option<&Node>) {
        let Some(node) = node else {
            return;
        };
        self.write_punctuation(":");
        self.write_space();
        self.emit_type_node_outside_extends(node);
    }

    pub(crate) fn emit_type_node_in_extends(&mut self, node: &Node) {
        let saved_in_extends = self.in_extends;
        self.in_extends = true;
        self.emit_type_node_preserving_extends(node, TypePrecedence::Conditional);
        self.in_extends = saved_in_extends;
    }

    pub(crate) fn emit_type_node_outside_extends(&mut self, node: &Node) {
        let saved_in_extends = self.in_extends;
        self.in_extends = false;
        self.emit_type_node_preserving_extends(node, TypePrecedence::Conditional);
        self.in_extends = saved_in_extends;
    }

    pub(crate) fn emit_type_node_preserving_extends(&mut self, node: &Node, precedence: TypePrecedence) {
        self.emit_type_node(node, precedence);
    }

    pub(crate) fn emit_type_node(&mut self, node: &Node, precedence: TypePrecedence) {
        let mut precedence = precedence;
        if self.in_extends && precedence <= TypePrecedence::Conditional {
            precedence = TypePrecedence::Function;
        }

        let saved_in_extends = self.in_extends;
        let parens = get_type_node_precedence07(node) < precedence;
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

            SyntaxKind::PropertyAccessExpression => self.emit_property_access_expression(node),
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

    pub(crate) fn emit_keyword_type_node(&mut self, node: &Node) {
        self.emit_keyword_node(node);
    }

    pub(crate) fn emit_type_predicate_parameter_name(&mut self, node: &Node) {
        match node.kind {
            SyntaxKind::Identifier => self.emit_identifier_reference(node),
            SyntaxKind::ThisType => self.emit_this_type(node),
            _ => panic!("unexpected TypePredicateParameterName: {:?}", node.kind),
        }
    }

    pub(crate) fn emit_type_predicate(&mut self, node: &Node) {
        let state = self.enter_node(node);
        let (asserts_modifier, parameter_name, type_node) = match &node.data {
            NodeData::TypePredicateNode(d) => (d.asserts_modifier.as_deref(), &d.parameter_name, d.type_node.as_deref()),
            _ => panic!("unexpected TypePredicate: {:?}", node.kind),
        };
        if let Some(asserts_modifier) = asserts_modifier {
            self.emit_token_node(Some(asserts_modifier));
            self.write_space();
        }
        self.emit_type_predicate_parameter_name(parameter_name);
        if let Some(type_node) = type_node {
            self.write_space();
            self.write_keyword("is");
            self.write_space();
            self.emit_type_node_outside_extends(type_node);
        }
        self.exit_node(node, state);
    }

    pub(crate) fn emit_type_argument(&mut self, node: &Node) {
        self.emit_type_node_outside_extends(node);
    }

    pub(crate) fn emit_type_reference(&mut self, node: &Node) {
        let state = self.enter_node(node);
        let (type_name, type_arguments) = match &node.data {
            NodeData::TypeReferenceNode(d) => (&d.type_name, d.type_arguments.as_deref()),
            _ => panic!("unexpected TypeReference: {:?}", node.kind),
        };
        self.emit_entity_name(type_name);
        self.emit_type_arguments(node, type_arguments);
        self.exit_node(node, state);
    }

    pub(crate) fn emit_function_type(&mut self, node: &Node) {
        let state = self.enter_node(node);
        let (type_parameters, parameters, type_node) = match &node.data {
            NodeData::FunctionTypeNode(d) => (d.type_parameters.as_deref(), &d.parameters, d.type_node.as_deref()),
            _ => panic!("unexpected FunctionType: {:?}", node.kind),
        };
        let indented = self.should_emit_indented(node);
        self.increase_indent_if(indented);
        self.push_name_generation_scope(node);
        self.emit_type_parameters(node, type_parameters);
        self.emit_parameters(node, parameters);
        self.write_space();
        self.emit_return_type(type_node);
        self.pop_name_generation_scope(node);
        self.decrease_indent_if(indented);
        self.exit_node(node, state);
    }

    pub(crate) fn emit_constructor_type(&mut self, node: &Node) {
        let state = self.enter_node(node);
        let (modifiers, type_parameters, parameters, type_node) = match &node.data {
            NodeData::ConstructorTypeNode(d) => (
                d.modifiers.as_deref(),
                d.type_parameters.as_deref(),
                &d.parameters,
                d.type_node.as_deref(),
            ),
            _ => panic!("unexpected ConstructorType: {:?}", node.kind),
        };
        self.emit_modifier_list(node, modifiers, false);
        self.write_keyword("new");
        self.write_space();
        let indented = self.should_emit_indented(node);
        self.increase_indent_if(indented);
        self.push_name_generation_scope(node);
        self.emit_type_parameters(node, type_parameters);
        self.emit_parameters(node, parameters);
        self.write_space();
        self.emit_return_type(type_node);
        self.pop_name_generation_scope(node);
        self.decrease_indent_if(indented);
        self.exit_node(node, state);
    }

    pub(crate) fn emit_type_query(&mut self, node: &Node) {
        let state = self.enter_node(node);
        let (expr_name, type_arguments) = match &node.data {
            NodeData::TypeQueryNode(d) => (&d.expr_name, d.type_arguments.as_deref()),
            _ => panic!("unexpected TypeQuery: {:?}", node.kind),
        };
        self.write_keyword("typeof");
        self.write_space();
        self.emit_entity_name(expr_name);
        self.emit_type_arguments(node, type_arguments);
        self.exit_node(node, state);
    }

    pub(crate) fn emit_type_literal(&mut self, node: &Node) {
        let state = self.enter_node(node);
        let members = match &node.data {
            NodeData::TypeLiteralNode(d) => &d.members,
            _ => panic!("unexpected TypeLiteral: {:?}", node.kind),
        };
        self.push_name_generation_scope(node);
        self.generate_all_member_names(members);
        self.write_punctuation("{");
        let format = if self.should_emit_on_single_line(node) {
            LF_SINGLE_LINE_TYPE_LITERAL_MEMBERS
        } else {
            LF_MULTI_LINE_TYPE_LITERAL_MEMBERS
        };
        self.emit_list(Self::emit_type_element, node, members, format | LF_NO_SPACE_IF_EMPTY);
        self.write_punctuation("}");
        self.pop_name_generation_scope(node);
        self.exit_node(node, state);
    }

    pub(crate) fn emit_type_element(&mut self, node: &Node) {
        self.emit_type_node_outside_extends(node);
    }

    pub(crate) fn emit_array_type(&mut self, node: &Node) {
        let state = self.enter_node(node);
        let element_type = match &node.data {
            NodeData::ArrayTypeNode(d) => &d.element_type,
            _ => panic!("unexpected ArrayType: {:?}", node.kind),
        };
        self.emit_postfix_type_operand(element_type, node);
        self.write_punctuation("[");
        self.write_punctuation("]");
        self.exit_node(node, state);
    }

    pub(crate) fn emit_tuple_element_type(&mut self, node: &Node) {
        self.emit_type_node_outside_extends(node);
    }

    pub(crate) fn emit_tuple_type(&mut self, node: &Node) {
        let state = self.enter_node(node);
        let elements = match &node.data {
            NodeData::TupleTypeNode(d) => &d.elements,
            _ => panic!("unexpected TupleType: {:?}", node.kind),
        };
        self.emit_token(SyntaxKind::OpenBracketToken, node.pos(), WriteKind::Punctuation, node);
        let format = if self.should_emit_on_single_line(node) {
            LF_SINGLE_LINE_TUPLE_TYPE_ELEMENTS
        } else {
            LF_MULTI_LINE_TUPLE_TYPE_ELEMENTS
        };
        self.emit_list(Self::emit_tuple_element_type, node, elements, format | LF_NO_SPACE_IF_EMPTY);
        self.emit_token(SyntaxKind::CloseBracketToken, elements.end(), WriteKind::Punctuation, node);
        self.exit_node(node, state);
    }

    pub(crate) fn emit_union_type_constituent(&mut self, node: &Node) {
        self.emit_type_node(node, TypePrecedence::TypeOperator);
    }

    pub(crate) fn emit_union_type(&mut self, node: &Node) {
        let state = self.enter_node(node);
        let types = match &node.data {
            NodeData::UnionTypeNode(d) => &d.types,
            _ => panic!("unexpected UnionType: {:?}", node.kind),
        };
        self.emit_list(Self::emit_union_type_constituent, node, types, LF_UNION_TYPE_CONSTITUENTS);
        self.exit_node(node, state);
    }

    pub(crate) fn emit_intersection_type_constituent(&mut self, node: &Node) {
        self.emit_type_node(node, TypePrecedence::TypeOperator);
    }

    pub(crate) fn emit_intersection_type(&mut self, node: &Node) {
        let state = self.enter_node(node);
        let types = match &node.data {
            NodeData::IntersectionTypeNode(d) => &d.types,
            _ => panic!("unexpected IntersectionType: {:?}", node.kind),
        };
        self.emit_list(Self::emit_intersection_type_constituent, node, types, LF_INTERSECTION_TYPE_CONSTITUENTS);
        self.exit_node(node, state);
    }

    pub(crate) fn emit_conditional_type(&mut self, node: &Node) {
        let state = self.enter_node(node);
        let (check_type, extends_type, true_type, false_type) = match &node.data {
            NodeData::ConditionalTypeNode(d) => (&d.check_type, &d.extends_type, &d.true_type, &d.false_type),
            _ => panic!("unexpected ConditionalType: {:?}", node.kind),
        };
        self.emit_type_node(check_type, TypePrecedence::Union);
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

    pub(crate) fn emit_infer_type_parameter(&mut self, node: &Node) {
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

    pub(crate) fn emit_infer_type(&mut self, node: &Node) {
        let state = self.enter_node(node);
        let type_parameter = match &node.data {
            NodeData::InferTypeNode(d) => &d.type_parameter,
            _ => panic!("unexpected InferType: {:?}", node.kind),
        };
        self.write_keyword("infer");
        self.write_space();
        self.emit_infer_type_parameter(type_parameter);
        self.exit_node(node, state);
    }

    pub(crate) fn emit_type_operator(&mut self, node: &Node) {
        let state = self.enter_node(node);
        let (operator, type_node) = match &node.data {
            NodeData::TypeOperatorNode(d) => (d.operator, &d.type_node),
            _ => panic!("unexpected TypeOperator: {:?}", node.kind),
        };
        self.emit_token(operator, node.pos(), WriteKind::Keyword, node);
        self.write_space();
        let precedence = if operator == SyntaxKind::ReadonlyKeyword {
            TypePrecedence::Postfix
        } else {
            TypePrecedence::TypeOperator
        };
        self.emit_type_node(type_node, precedence);
        self.exit_node(node, state);
    }

    pub(crate) fn emit_indexed_access_type(&mut self, node: &Node) {
        let state = self.enter_node(node);
        let (object_type, index_type) = match &node.data {
            NodeData::IndexedAccessTypeNode(d) => (&d.object_type, &d.index_type),
            _ => panic!("unexpected IndexedAccessType: {:?}", node.kind),
        };
        self.emit_postfix_type_operand(object_type, node);
        self.write_punctuation("[");
        self.emit_type_node_outside_extends(index_type);
        self.write_punctuation("]");
        self.exit_node(node, state);
    }

    pub(crate) fn emit_mapped_type_parameter(&mut self, node: &Node) {
        let state = self.enter_node(node);
        let (name, constraint) = match &node.data {
            NodeData::TypeParameterDeclaration(d) => (&d.name, d.constraint.as_deref()),
            _ => panic!("unexpected TypeParameterDeclaration: {:?}", node.kind),
        };
        self.emit_binding_identifier(name);
        self.write_space();
        self.write_keyword("in");
        self.write_space();
        self.emit_type_node_outside_extends(constraint.expect("mapped type parameter constraint"));
        self.exit_node(node, state);
    }

    pub(crate) fn emit_mapped_type(&mut self, node: &Node) {
        let state = self.enter_node(node);
        let (readonly_token, type_parameter, name_type, question_token, type_node, members) = match &node.data {
            NodeData::MappedTypeNode(d) => (
                d.readonly_token.as_deref(),
                &d.type_parameter,
                d.name_type.as_deref(),
                d.question_token.as_deref(),
                d.type_node.as_deref(),
                d.members.as_deref(),
            ),
            _ => panic!("unexpected MappedType: {:?}", node.kind),
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
                self.emit_list(Self::emit_type_element, node, members, LF_PRESERVE_LINES);
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

    pub(crate) fn emit_literal_type(&mut self, node: &Node) {
        let state = self.enter_node(node);
        let literal = match &node.data {
            NodeData::LiteralTypeNode(d) => &d.literal,
            _ => panic!("unexpected LiteralType: {:?}", node.kind),
        };
        self.emit_expression(literal, OperatorPrecedence::Comma);
        self.exit_node(node, state);
    }

    pub(crate) fn emit_import_type_node_attributes(&mut self, node: &Node) {
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
        self.emit_list(Self::emit_import_attribute_node, node, attributes, LF_IMPORT_ATTRIBUTES);
        self.write_space();
        self.write_punctuation("}");
        self.exit_node(node, state);
    }

    pub(crate) fn emit_import_attribute_node(&mut self, node: &Node) {
        let state = self.enter_node(node);
        let (name, value) = match &node.data {
            NodeData::ImportAttribute(d) => (&d.name, &d.value),
            _ => panic!("unexpected ImportAttribute: {:?}", node.kind),
        };
        self.emit_property_name(Some(name));
        self.write_punctuation(":");
        self.write_space();
        self.emit_expression(value, OperatorPrecedence::Comma);
        self.exit_node(node, state);
    }

    pub(crate) fn emit_import_type_node(&mut self, node: &Node) {
        let state = self.enter_node(node);
        let (is_type_of, argument, attributes, qualifier, type_arguments) = match &node.data {
            NodeData::ImportTypeNode(d) => (
                d.is_type_of,
                &d.argument,
                d.attributes.as_deref(),
                d.qualifier.as_deref(),
                d.type_arguments.as_deref(),
            ),
            _ => panic!("unexpected ImportType: {:?}", node.kind),
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
        self.emit_type_arguments(node, type_arguments);
        self.exit_node(node, state);
    }

    pub(crate) fn emit_expression_with_type_arguments(&mut self, node: &Node) {
        let state = self.enter_node(node);
        let (expression, type_arguments) = match &node.data {
            NodeData::ExpressionWithTypeArguments(d) => (&d.expression, d.type_arguments.as_deref()),
            _ => panic!("unexpected ExpressionWithTypeArguments: {:?}", node.kind),
        };
        self.emit_expression(expression, OperatorPrecedence::Member);
        self.emit_type_arguments(node, type_arguments);
        self.exit_node(node, state);
    }

    pub(crate) fn emit_jsdoc_all_type(&mut self, node: &Node) {
        self.emit_keyword_node(node);
    }

    pub(crate) fn emit_jsdoc_non_nullable_type(&mut self, node: &Node) {
        let state = self.enter_node(node);
        let type_node = match &node.data {
            NodeData::JSDocNonNullableType(d) => &d.type_node,
            _ => panic!("unexpected JSDocNonNullableType: {:?}", node.kind),
        };
        self.write_punctuation("!");
        self.emit_type_node(type_node, TypePrecedence::NonArray);
        self.exit_node(node, state);
    }

    pub(crate) fn emit_jsdoc_nullable_type(&mut self, node: &Node) {
        let state = self.enter_node(node);
        let type_node = match &node.data {
            NodeData::JSDocNullableType(d) => &d.type_node,
            _ => panic!("unexpected JSDocNullableType: {:?}", node.kind),
        };
        self.write_punctuation("?");
        self.emit_type_node(type_node, TypePrecedence::NonArray);
        self.exit_node(node, state);
    }

    pub(crate) fn emit_jsdoc_optional_type(&mut self, node: &Node) {
        let state = self.enter_node(node);
        let type_node = match &node.data {
            NodeData::JSDocOptionalType(d) => &d.type_node,
            _ => panic!("unexpected JSDocOptionalType: {:?}", node.kind),
        };
        self.emit_type_node(type_node, TypePrecedence::Jsdoc);
        self.write_punctuation("=");
        self.exit_node(node, state);
    }

    pub(crate) fn emit_jsdoc_variadic_type(&mut self, node: &Node) {
        let state = self.enter_node(node);
        let type_node = match &node.data {
            NodeData::JSDocVariadicType(d) => &d.type_node,
            _ => panic!("unexpected JSDocVariadicType: {:?}", node.kind),
        };
        self.write_punctuation("...");
        self.emit_type_node(type_node, TypePrecedence::Jsdoc);
        self.exit_node(node, state);
    }

    pub(crate) fn emit_type_parameter(&mut self, node: &Node) {
        let state = self.enter_node(node);
        let (modifiers, name, constraint, default_type) = match &node.data {
            NodeData::TypeParameterDeclaration(d) => (
                d.modifiers.as_deref(),
                &d.name,
                d.constraint.as_deref(),
                d.default_type.as_deref(),
            ),
            _ => panic!("unexpected TypeParameterDeclaration: {:?}", node.kind),
        };
        self.emit_modifier_list(node, modifiers, false);
        self.emit_binding_identifier(name);
        if let Some(constraint) = constraint {
            self.write_space();
            self.write_keyword("extends");
            self.write_space();
            self.emit_type_node_outside_extends(constraint);
        }
        if let Some(default_type) = default_type {
            self.write_space();
            self.write_operator("=");
            self.write_space();
            self.emit_type_node_outside_extends(default_type);
        }
        self.exit_node(node, state);
    }

    pub(crate) fn emit_type_parameter_declaration_node(&mut self, node: &Node) {
        if matches!(node.kind, SyntaxKind::TypeParameter) {
            self.emit_type_parameter(node);
        } else {
            self.emit_type_argument(node);
        }
    }

    pub(crate) fn emit_type_parameters(&mut self, parent_node: &Node, nodes: Option<&NodeList>) {
        let Some(nodes) = nodes else {
            return;
        };
        let format = if parent_node.kind == SyntaxKind::ArrowFunction {
            LF_TYPE_PARAMETERS | LF_ALLOW_TRAILING_COMMA
        } else {
            LF_TYPE_PARAMETERS
        };
        self.emit_list(Self::emit_type_parameter_declaration_node, parent_node, nodes, format);
    }

    pub(crate) fn emit_type_arguments(&mut self, parent_node: &Node, nodes: Option<&NodeList>) {
        let Some(nodes) = nodes else {
            return;
        };
        self.emit_list(Self::emit_type_parameter_declaration_node, parent_node, nodes, LF_TYPE_ARGUMENTS);
    }

    pub(crate) fn generate_all_names(&mut self, nodes: &NodeList) {
        self.generate_all_names_opt(Some(nodes));
    }

    pub(crate) fn generate_all_names_opt(&mut self, nodes: Option<&NodeList>) {
        let Some(nodes) = nodes else {
            return;
        };
        for node in &nodes.nodes {
            self.generate_names(Some(node));
        }
    }

    fn generate_statement_of(node: &Node) -> Option<&Node> {
        match &node.data {
            NodeData::LabeledStatement(d) => Some(&d.statement),
            NodeData::WithStatement(d) => Some(&d.statement),
            NodeData::DoStatement(d) => Some(&d.statement),
            NodeData::WhileStatement(d) => Some(&d.statement),
            NodeData::ForStatement(d) => Some(&d.statement),
            NodeData::ForInOrOfStatement(d) => Some(&d.statement),
            _ => None,
        }
    }

    pub(crate) fn generate_names(&mut self, node: Option<&Node>) {
        let Some(node) = node else {
            return;
        };
        match node.kind {
            SyntaxKind::Block => {
                let statements = match &node.data {
                    NodeData::Block(d) => &d.statements,
                    _ => panic!("unexpected Block: {:?}", node.kind),
                };
                self.generate_all_names_opt(Some(statements));
            }
            SyntaxKind::CaseClause | SyntaxKind::DefaultClause => {
                let statements = match &node.data {
                    NodeData::CaseOrDefaultClause(d) => &d.statements,
                    _ => panic!("unexpected CaseOrDefaultClause: {:?}", node.kind),
                };
                self.generate_all_names_opt(Some(statements));
            }
            SyntaxKind::LabeledStatement
            | SyntaxKind::WithStatement
            | SyntaxKind::DoStatement
            | SyntaxKind::WhileStatement => self.generate_names(Self::generate_statement_of(node)),
            SyntaxKind::IfStatement => {
                let (then_statement, else_statement) = match &node.data {
                    NodeData::IfStatement(d) => (&d.then_statement, d.else_statement.as_deref()),
                    _ => panic!("unexpected IfStatement: {:?}", node.kind),
                };
                self.generate_names(Some(then_statement));
                self.generate_names(else_statement);
            }
            SyntaxKind::ForStatement | SyntaxKind::ForOfStatement | SyntaxKind::ForInStatement => {
                let (initializer, statement) = match &node.data {
                    NodeData::ForStatement(d) => (d.initializer.as_deref(), &d.statement),
                    NodeData::ForInOrOfStatement(d) => (Some(d.initializer.as_ref()), &d.statement),
                    _ => panic!("unexpected ForStatement: {:?}", node.kind),
                };
                self.generate_names(initializer);
                self.generate_names(Some(statement));
            }
            SyntaxKind::SwitchStatement => {
                let case_block = match &node.data {
                    NodeData::SwitchStatement(d) => &d.case_block,
                    _ => panic!("unexpected SwitchStatement: {:?}", node.kind),
                };
                self.generate_names(Some(case_block));
            }
            SyntaxKind::CaseBlock => {
                let clauses = match &node.data {
                    NodeData::CaseBlock(d) => &d.clauses,
                    _ => panic!("unexpected CaseBlock: {:?}", node.kind),
                };
                self.generate_all_names_opt(Some(clauses));
            }
            SyntaxKind::TryStatement => {
                let (try_block, catch_clause, finally_block) = match &node.data {
                    NodeData::TryStatement(d) => (
                        &d.try_block,
                        d.catch_clause.as_deref(),
                        d.finally_block.as_deref(),
                    ),
                    _ => panic!("unexpected TryStatement: {:?}", node.kind),
                };
                self.generate_names(Some(try_block));
                self.generate_names(catch_clause);
                self.generate_names(finally_block);
            }
            SyntaxKind::CatchClause => {
                let (variable_declaration, block) = match &node.data {
                    NodeData::CatchClause(d) => (d.variable_declaration.as_deref(), &d.block),
                    _ => panic!("unexpected CatchClause: {:?}", node.kind),
                };
                self.generate_names(variable_declaration);
                self.generate_names(Some(block));
            }
            SyntaxKind::VariableStatement => {
                let declaration_list = match &node.data {
                    NodeData::VariableStatement(d) => &d.declaration_list,
                    _ => panic!("unexpected VariableStatement: {:?}", node.kind),
                };
                self.generate_names(Some(declaration_list));
            }
            SyntaxKind::VariableDeclarationList => {
                let declarations = match &node.data {
                    NodeData::VariableDeclarationList(d) => &d.declarations,
                    _ => panic!("unexpected VariableDeclarationList: {:?}", node.kind),
                };
                self.generate_all_names_opt(Some(declarations));
            }
            SyntaxKind::VariableDeclaration
            | SyntaxKind::Parameter
            | SyntaxKind::BindingElement
            | SyntaxKind::ClassDeclaration => self.generate_name_if_needed(node.name()),
            SyntaxKind::FunctionDeclaration => {
                self.generate_name_if_needed(node.name());
                if self.should_reuse_temp_variable_scope(Some(node)) {
                    let parameters = match &node.data {
                        NodeData::FunctionDeclaration(d) => &d.parameters,
                        _ => panic!("unexpected FunctionDeclaration: {:?}", node.kind),
                    };
                    self.generate_all_names_opt(Some(parameters));
                    let body = match &node.data {
                        NodeData::FunctionDeclaration(d) => d.body.as_deref(),
                        _ => panic!("unexpected FunctionDeclaration: {:?}", node.kind),
                    };
                    self.generate_names(body);
                }
            }
            SyntaxKind::ObjectBindingPattern | SyntaxKind::ArrayBindingPattern => {
                let elements = match &node.data {
                    NodeData::BindingPattern(d) => &d.elements,
                    _ => panic!("unexpected BindingPattern: {:?}", node.kind),
                };
                self.generate_all_names_opt(Some(elements));
            }
            SyntaxKind::ImportDeclaration | SyntaxKind::JSImportDeclaration => {
                self.generate_names(tsox_frontend::ast::mig::m3b::import_clause(node).map(|n| n.as_ref()));
            }
            SyntaxKind::ImportClause => {
                let (name, named_bindings) = match &node.data {
                    NodeData::ImportClause(d) => (d.name.as_ref(), d.named_bindings.as_deref()),
                    _ => panic!("unexpected ImportClause: {:?}", node.kind),
                };
                self.generate_name_if_needed(name);
                self.generate_names_opt(named_bindings);
            }
            SyntaxKind::NamespaceImport | SyntaxKind::NamespaceExport => {
                self.generate_name_if_needed(node.name())
            }
            SyntaxKind::NamedImports => {
                let elements = match &node.data {
                    NodeData::NamedImports(d) => &d.elements,
                    _ => panic!("unexpected NamedImports: {:?}", node.kind),
                };
                self.generate_all_names_opt(Some(elements));
            }
            SyntaxKind::ImportSpecifier => {
                let property_name = match &node.data {
                    NodeData::ImportSpecifier(d) => d.property_name.as_ref(),
                    _ => panic!("unexpected ImportSpecifier: {:?}", node.kind),
                };
                if let Some(property_name) = property_name {
                    self.generate_name_if_needed(Some(property_name));
                } else {
                    self.generate_name_if_needed(node.name());
                }
            }
            _ => {}
        }
    }

    pub(crate) fn generate_names_opt(&mut self, node: Option<&Node>) {
        self.generate_names(node);
    }

    pub(crate) fn generate_all_member_names(&mut self, nodes: &NodeList) {
        for node in &nodes.nodes {
            self.generate_member_names(Some(node));
        }
    }

    pub(crate) fn generate_member_names(&mut self, node: Option<&Node>) {
        let Some(node) = node else {
            return;
        };
        match node.kind {
            SyntaxKind::PropertyAssignment
            | SyntaxKind::ShorthandPropertyAssignment
            | SyntaxKind::PropertyDeclaration
            | SyntaxKind::PropertySignature
            | SyntaxKind::MethodDeclaration
            | SyntaxKind::MethodSignature
            | SyntaxKind::GetAccessor
            | SyntaxKind::SetAccessor => self.generate_name_if_needed(node.name()),
            _ => {}
        }
    }

    pub(crate) fn generate_name_if_needed(&mut self, name: Option<&Arc<Node>>) {
        if let Some(name) = name {
            if is_member_name(name) {
                self.name_generator
                    .generate_name_for_node(name, false, GeneratedIdentifierFlags::NONE, "", "");
            } else if matches!(name.kind, SyntaxKind::ObjectBindingPattern | SyntaxKind::ArrayBindingPattern) {
                self.generate_names(Some(name));
            }
        }
    }

    pub(crate) fn should_reuse_temp_variable_scope(&self, node: Option<&Node>) -> bool {
        node.is_some()
            && self.emit_context.emit_flags_of(node.unwrap()) & EmitFlags::REUSE_TEMP_VARIABLE_SCOPE.0 != 0
    }

    pub(crate) fn should_emit_indented(&self, node: &Node) -> bool {
        self.emit_context.emit_flags_of(node) & EmitFlags::INDENTED.0 != 0
    }

    pub(crate) fn should_allow_trailing_comma(&self, node: &Node, list: &NodeList) -> bool {
        if self
            .current_source_file
            .as_deref()
            .map_or(true, |source_file| source_file.script_kind == ScriptKind::Json)
        {
            return false;
        }
        match node.kind {
            SyntaxKind::ObjectLiteralExpression
            | SyntaxKind::ArrayLiteralExpression
            | SyntaxKind::ArrowFunction
            | SyntaxKind::Constructor
            | SyntaxKind::GetAccessor
            | SyntaxKind::SetAccessor
            | SyntaxKind::TypeAliasDeclaration
            | SyntaxKind::JSTypeAliasDeclaration
            | SyntaxKind::FunctionType
            | SyntaxKind::ConstructorType
            | SyntaxKind::CallSignature
            | SyntaxKind::ConstructSignature
            | SyntaxKind::TaggedTemplateExpression
            | SyntaxKind::ObjectBindingPattern
            | SyntaxKind::ArrayBindingPattern
            | SyntaxKind::NamedImports
            | SyntaxKind::NamedExports
            | SyntaxKind::ImportAttributes
            | SyntaxKind::FunctionDeclaration
            | SyntaxKind::FunctionExpression
            | SyntaxKind::MethodDeclaration
            | SyntaxKind::CallExpression
            | SyntaxKind::NewExpression => true,
            SyntaxKind::ClassExpression | SyntaxKind::ClassDeclaration | SyntaxKind::InterfaceDeclaration => {
                let type_parameters: Option<&NodeList> = match &node.data {
                    NodeData::ClassDeclaration(d) => d.type_parameters.as_deref(),
                    NodeData::ClassExpression(d) => d.type_parameters.as_deref(),
                    NodeData::InterfaceDeclaration(d) => d.type_parameters.as_deref(),
                    _ => None,
                };
                match type_parameters {
                    Some(tp) => std::ptr::eq(tp, list),
                    None => false,
                }
            }
            _ => false,
        }
    }

    pub(crate) fn write_line_separators_and_indent_before(&mut self, node: Option<&Node>, parent: &Node) -> bool {
        if self.options.preserve_source_newlines {
            let leading_newlines = self.get_leading_line_terminator_count07(parent, node, LF_NONE);
            if leading_newlines > 0 {
                self.increase_indent();
                self.write_line_repeat(leading_newlines);
                return true;
            }
        }
        false
    }

    pub(crate) fn write_line_separators_after(&mut self, node: Option<&Node>, parent: &Node) {
        if self.options.preserve_source_newlines {
            let trailing_newlines = self.get_closing_line_terminator_count07(parent, node, LF_NONE);
            if trailing_newlines > 0 {
                self.write_line_repeat(trailing_newlines);
            }
        }
    }

    pub(crate) fn write_line_repeat(&mut self, count: i32) {
        for _ in 0..count {
            self.write_line();
        }
    }

    pub(crate) fn get_leading_line_terminator_count07(
        &mut self,
        parent_node: &Node,
        first_child: Option<&Node>,
        format: ListFlags,
    ) -> i32 {
        if format & LF_PRESERVE_LINES != 0 || self.options.preserve_source_newlines {
            if format & LF_PREFER_NEW_LINE != 0 {
                return 1;
            }
            let Some(first_child) = first_child else {
                return 0;
            };
            if self.should_emit_on_new_line07(first_child, format) {
                return 1;
            }
        }
        if format & LF_MULTI_LINE != 0 && format & LF_NO_TRAILING_NEW_LINE == 0 {
            1
        } else {
            0
        }
    }

    pub(crate) fn get_closing_line_terminator_count07(
        &mut self,
        parent_node: &Node,
        last_child: Option<&Node>,
        format: ListFlags,
    ) -> i32 {
        if format & LF_PRESERVE_LINES != 0 || self.options.preserve_source_newlines {
            if format & LF_PREFER_NEW_LINE != 0 {
                return 1;
            }
            let Some(last_child) = last_child else {
                return 0;
            };
            if self.should_emit_on_new_line07(last_child, format) {
                return 1;
            }
        }
        if format & LF_MULTI_LINE != 0 && format & LF_NO_TRAILING_NEW_LINE == 0 {
            1
        } else {
            0
        }
    }

    pub(crate) fn should_emit_on_new_line07(&self, node: &Node, format: ListFlags) -> bool {
        self.emit_context.emit_flags_of(node) & EmitFlags::START_ON_NEW_LINE.0 != 0
            || format & LF_PREFER_NEW_LINE != 0
    }

    pub(crate) fn enter_token_node(&mut self, node: &Node, flags: TokenEmitFlags) -> PrinterState {
        PrinterState {
            comment_state: None,
            source_map_state: None,
        }
    }

    pub(crate) fn exit_token_node(&mut self, node: &Node, previous_state: PrinterState) {}
}
