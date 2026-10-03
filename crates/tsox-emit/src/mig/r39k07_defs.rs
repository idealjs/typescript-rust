#![allow(dead_code, unused_imports, unused_variables)]

use std::sync::Arc;

use tsox_frontend::ast::node::{Node, NodeList};
use tsox_frontend::ast::node_data_generated::{is_partially_emitted_expression, is_string_literal, NodeData};
use tsox_frontend::ast::node_source_file::ScriptKind;
use tsox_frontend::ast::syntax_kind_generated::SyntaxKind;
use tsox_frontend::ast::utilities::{is_member_name, node_is_synthesized, position_is_synthesized};
use tsox_frontend::format::mig::m4t_2::{escape_jsx_attribute_string, escape_non_ascii_string, escape_string, QuoteChar};
use tsox_frontend::format::mig::m4t_6::get_literal_text;
use tsox_frontend::scanner::mig::m3i::get_source_text_of_node_from_source_file;
use tsox_frontend::ast::mig::m3e_3::OPERATOR_PRECEDENCE_DISALLOW_COMMA;
use tsox_frontend::scanner::token_to_string;

use crate::mig::m4q::r33k12_defs::{
    EmitContext, EmitFlags, ListFlags, OperatorPrecedence, PrinterState, TokenEmitFlags, TypePrecedence, WriteKind,
    LF_ALLOW_TRAILING_COMMA, LF_ARRAY_BINDING_PATTERN_ELEMENTS, LF_MULTI_LINE, LF_NO_TRAILING_NEW_LINE, LF_NONE,
    LF_PREFER_NEW_LINE, LF_PRESERVE_LINES, LF_TYPE_ARGUMENTS, LF_TYPE_PARAMETERS,
};
use crate::mig::m4q::Printer;
use crate::mig::m4q::r39k08_defs::EmitContextExtK08;
use crate::mig::m4q_2::r36k14_defs::TypedNode;
use crate::mig::m4q_3::r37k6_defs::TypedNode6;
use crate::mig::m4s::EmitTextWriter;
use crate::printer::GetLiteralTextFlags;

pub(crate) trait EmitContextExt07 {
    fn auto_generate_of(&self, node: &Node) -> Option<()>;
    fn text_source_of(&self, node: &Node) -> Option<Node>;
}

impl EmitContextExt07 for EmitContext {
    fn auto_generate_of(&self, _node: &Node) -> Option<()> { ::tsox_core::fntrace::enter("auto_generate_of"); 
        None
    }

    fn text_source_of(&self, _node: &Node) -> Option<Node> { ::tsox_core::fntrace::enter("text_source_of"); 
        None
    }
}

pub(crate) trait QNode07 {
    fn q_node(&self) -> &Node;
}

impl QNode07 for Node {
    fn q_node(&self) -> &Node { ::tsox_core::fntrace::enter("q_node"); 
        self
    }
}

impl QNode07 for Arc<Node> {
    fn q_node(&self) -> &Node { ::tsox_core::fntrace::enter("q_node"); 
        self
    }
}

impl<'a> QNode07 for TypedNode<'a> {
    fn q_node(&self) -> &Node { ::tsox_core::fntrace::enter("q_node"); 
        self.node
    }
}

impl<'a> QNode07 for TypedNode6<'a> {
    fn q_node(&self) -> &Node { ::tsox_core::fntrace::enter("q_node"); 
        self.node
    }
}

pub(crate) fn greatest_end07(end: usize, nodes: &[Option<&Node>]) -> usize { ::tsox_core::fntrace::enter("greatest_end07"); 
    let mut end = end;
    for node in nodes.iter().rev() {
        if let Some(node) = node {
            let node_end = node.end();
            if end < node_end {
                end = node_end;
            }
        }
    }
    end
}

pub(crate) fn skip_partially_emitted_expressions07(node: &Node) -> &Node { ::tsox_core::fntrace::enter("skip_partially_emitted_expressions07"); 
    let mut node = node;
    while is_partially_emitted_expression(node) {
        match &node.data {
            NodeData::PartiallyEmittedExpression(d) => node = d.expression.as_ref(),
            _ => break,
        }
    }
    node
}

pub(crate) fn is_binary_operation07(node: &Node, operator: SyntaxKind) -> bool { ::tsox_core::fntrace::enter("is_binary_operation07"); 
    match &node.data {
        NodeData::BinaryExpression(d) => d.operator_token.kind == operator,
        _ => false,
    }
}

pub(crate) fn get_type_node_precedence07(node: &Node) -> TypePrecedence { ::tsox_core::fntrace::enter("get_type_node_precedence07"); 
    match node.kind {
        SyntaxKind::ConditionalType => TypePrecedence::Conditional,
        SyntaxKind::JSDocOptionalType | SyntaxKind::JSDocVariadicType => TypePrecedence::Jsdoc,
        SyntaxKind::FunctionType | SyntaxKind::ConstructorType => TypePrecedence::Function,
        SyntaxKind::UnionType => TypePrecedence::Union,
        SyntaxKind::IntersectionType => TypePrecedence::Intersection,
        SyntaxKind::TypeOperator | SyntaxKind::TypeQuery => TypePrecedence::TypeOperator,
        SyntaxKind::InferType => {
            let has_constraint = match &node.data {
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
        SyntaxKind::IndexedAccessType | SyntaxKind::ArrayType | SyntaxKind::OptionalType => TypePrecedence::Postfix,
        _ => TypePrecedence::NonArray,
    }
}

pub(crate) fn can_emit_simple_arrow_head07(parent_node: &Node, parameters: &NodeList) -> bool { ::tsox_core::fntrace::enter("can_emit_simple_arrow_head07"); 
    if parent_node.kind != SyntaxKind::ArrowFunction || parameters.nodes.len() != 1 {
        return false;
    }
    let parameter = &parameters.nodes[0];
    if parameter.kind != SyntaxKind::Parameter {
        return false;
    }
    let (type_parameters, type_node) = match &parent_node.data {
        NodeData::ArrowFunction(d) => (d.type_parameters.as_deref(), d.type_node.as_deref()),
        _ => return false,
    };
    let (modifiers, dot_dot_dot_token, question_token, ty, initializer, name_is_identifier) = match &parameter.data {
        NodeData::ParameterDeclaration(d) => (
            d.modifiers.as_ref().map_or(true, |m| m.nodes.is_empty()),
            d.dot_dot_dot_token.is_some(),
            d.question_token.is_some(),
            d.type_node.is_some(),
            d.initializer.is_some(),
            d.name.kind == SyntaxKind::Identifier,
        ),
        _ => return false,
    };
    parameter.pos() == parent_node.pos()
        && type_parameters.is_none()
        && type_node.is_none()
        && modifiers
        && !parameters.has_trailing_comma()
        && !dot_dot_dot_token
        && !question_token
        && !ty
        && !initializer
        && name_is_identifier
}

impl Printer {
    pub(crate) fn write_as(&mut self, text: &str, write_kind: WriteKind) { ::tsox_core::fntrace::enter("write_as"); 
        match write_kind {
            WriteKind::None => self.writer.write(text),
            WriteKind::Parameter => self.writer.write_parameter(text),
            WriteKind::Keyword => self.writer.write_keyword(text),
            WriteKind::Operator => self.writer.write_operator(text),
            WriteKind::Property => self.writer.write_property(text),
            WriteKind::Punctuation => self.writer.write_punctuation(text),
            WriteKind::StringLiteral => self.writer.write_string_literal(text),
            WriteKind::Comment => self.writer.write_comment(text),
            WriteKind::Literal => self.writer.write_literal(text),
        }
    }

    pub(crate) fn write(&mut self, text: &str) { ::tsox_core::fntrace::enter("write"); 
        self.write_as(text, self.write_kind);
    }

    pub(crate) fn write_punctuation(&mut self, text: &str) { ::tsox_core::fntrace::enter("write_punctuation"); 
        self.writer.write_punctuation(text);
    }

    pub(crate) fn write_operator(&mut self, text: &str) { ::tsox_core::fntrace::enter("write_operator"); 
        self.writer.write_operator(text);
    }

    pub(crate) fn write_keyword(&mut self, text: &str) { ::tsox_core::fntrace::enter("write_keyword"); 
        self.writer.write_keyword(text);
    }

    pub(crate) fn write_comment(&mut self, text: &str) { ::tsox_core::fntrace::enter("write_comment"); 
        self.writer.write_comment(text);
    }

    pub(crate) fn write_trailing_semicolon(&mut self) { ::tsox_core::fntrace::enter("write_trailing_semicolon"); 
        self.writer.write_trailing_semicolon(";");
    }

    pub(crate) fn increase_indent(&mut self) { ::tsox_core::fntrace::enter("increase_indent"); 
        self.writer.increase_indent();
    }

    pub(crate) fn decrease_indent(&mut self) { ::tsox_core::fntrace::enter("decrease_indent"); 
        self.writer.decrease_indent();
    }

    pub(crate) fn increase_indent_if(&mut self, indent_requested: bool) { ::tsox_core::fntrace::enter("increase_indent_if"); 
        if indent_requested {
            self.increase_indent();
        }
    }

    pub(crate) fn decrease_indent_if(&mut self, indent_requested: bool) { ::tsox_core::fntrace::enter("decrease_indent_if"); 
        if indent_requested {
            self.decrease_indent();
        }
    }

    pub(crate) fn write_token_text(&mut self, token: SyntaxKind, write_kind: WriteKind, pos: usize) -> usize { ::tsox_core::fntrace::enter("write_token_text"); 
        let token_string = token_to_string(token);
        self.write_as(&token_string, write_kind);
        if position_is_synthesized(pos) {
            pos
        } else {
            pos + token_string.len()
        }
    }

    pub(crate) fn emit_literal(&mut self, node: &Node, mut flags: GetLiteralTextFlags) { ::tsox_core::fntrace::enter("emit_literal"); 
        if self.options.never_ascii_escape {
            flags |= GetLiteralTextFlags::NEVER_ASCII_ESCAPE;
        }
        if self.options.terminate_unterminated_literals {
            flags |= GetLiteralTextFlags::TERMINATE_UNTERMINATED_LITERALS;
        }
        let text = self.get_literal_text_of_node(node, None, flags);
        self.writer.write_string_literal(&text);
    }

    pub(crate) fn get_literal_text_of_node(
        &mut self,
        node: &Node,
        source_file: Option<&Node>,
        mut flags: GetLiteralTextFlags,
    ) -> String { ::tsox_core::fntrace::enter("get_literal_text_of_node"); 
        if is_string_literal(node) {
            if let Some(text_source_node) = self.emit_context.text_source_of(node) {
                let text = match text_source_node.kind {
                    SyntaxKind::NumericLiteral => text_source_node.text().to_string(),
                    SyntaxKind::Identifier | SyntaxKind::PrivateIdentifier | SyntaxKind::JsxNamespacedName => {
                        self.get_text_of_node(&text_source_node, false)
                    }
                    _ => {
                        return self.get_literal_text_of_node(&text_source_node, None, flags);
                    }
                };
                if flags.contains(GetLiteralTextFlags::JSX_ATTRIBUTE_ESCAPE) {
                    return format!("\"{}\"", escape_jsx_attribute_string(&text, QuoteChar::DoubleQuote));
                }
                if flags.contains(GetLiteralTextFlags::NEVER_ASCII_ESCAPE)
                    || self.emit_context.emit_flags_of(node) & EmitFlags::NO_ASCII_ESCAPING.0 != 0
                {
                    return escape_string(&text, QuoteChar::DoubleQuote);
                }
                return escape_non_ascii_string(&text, QuoteChar::DoubleQuote);
            }
        }
        if self.emit_context.emit_flags_of(node) & EmitFlags::NO_ASCII_ESCAPING.0 != 0 {
            flags |= GetLiteralTextFlags::NEVER_ASCII_ESCAPE;
        }
        node.text().to_string()
    }

    pub(crate) fn get_text_of_node(&mut self, node: &Node, include_trivia: bool) -> String { ::tsox_core::fntrace::enter("get_text_of_node"); 
        if is_member_name(node) && self.emit_context.auto_generate_of(node).is_some() {
            return node.text().to_string();
        }
        let can_use_source_file = self.current_source_file.is_some() && !node_is_synthesized(node);
        match node.kind {
            SyntaxKind::Identifier | SyntaxKind::PrivateIdentifier | SyntaxKind::JsxNamespacedName => {
                if !can_use_source_file {
                    return node.text().to_string();
                }
            }
            SyntaxKind::StringLiteral
            | SyntaxKind::NumericLiteral
            | SyntaxKind::BigIntLiteral
            | SyntaxKind::NoSubstitutionTemplateLiteral
            | SyntaxKind::TemplateHead
            | SyntaxKind::TemplateMiddle
            | SyntaxKind::TemplateTail => {
                return self.get_literal_text_of_node(node, None, GetLiteralTextFlags::NONE);
            }
            _ => panic!("unexpected node: {:?}", node.kind),
        }
        node.text().to_string()
    }

    pub(crate) fn emit_identifier_text(&mut self, node: &Node) { ::tsox_core::fntrace::enter("emit_identifier_text"); 
        let text = self.get_text_of_node(node, false);
        self.write(&text);
    }

    pub(crate) fn emit_identifier_name<'a, N: QNode07 + ?Sized>(&mut self, node: &'a N) { ::tsox_core::fntrace::enter("emit_identifier_name"); 
        let node = node.q_node();
        let state = self.enter_node(node);
        self.emit_identifier_text(node);
        self.exit_node(node, state);
    }

    pub(crate) fn emit_binding_identifier<'a, N: QNode07 + ?Sized>(&mut self, node: &'a N) { ::tsox_core::fntrace::enter("emit_binding_identifier"); 
        let node = node.q_node();
        let state = self.enter_node(node);
        self.emit_identifier_text(node);
        self.exit_node(node, state);
    }

    pub(crate) fn emit_identifier_reference(&mut self, node: &Node) { ::tsox_core::fntrace::enter("emit_identifier_reference"); 
        let state = self.enter_node(node);
        self.emit_identifier_text(node);
        self.exit_node(node, state);
    }

    pub(crate) fn emit_binding_name(&mut self, node: &Node) { ::tsox_core::fntrace::enter("emit_binding_name"); 
        match node.kind {
            SyntaxKind::Identifier => self.emit_binding_identifier(node),
            SyntaxKind::ObjectBindingPattern => self.emit_object_binding_pattern(node),
            SyntaxKind::ArrayBindingPattern => self.emit_array_binding_pattern(node),
            _ => panic!("unexpected BindingName: {:?}", node.kind),
        }
    }

    pub(crate) fn emit_array_binding_pattern(&mut self, node: &Node) { ::tsox_core::fntrace::enter("emit_array_binding_pattern"); 
        let state = self.enter_node(node);
        let elements = match &node.data {
            NodeData::BindingPattern(d) => &d.elements,
            _ => panic!("unexpected ArrayBindingPattern: {:?}", node.kind),
        };
        self.write_punctuation("[");
        self.emit_list(Self::emit_binding_element_node, node, elements, LF_ARRAY_BINDING_PATTERN_ELEMENTS);
        self.write_punctuation("]");
        self.exit_node(node, state);
    }

    pub(crate) fn emit_binding_element_node(&mut self, node: &Node) { ::tsox_core::fntrace::enter("emit_binding_element_node"); 
        let state = self.enter_node(node);
        let (dot_dot_dot_token, property_name, name, initializer) = match &node.data {
            NodeData::BindingElement(d) => (
                d.dot_dot_dot_token.as_deref(),
                d.property_name.as_deref(),
                d.name.as_deref(),
                d.initializer.as_deref(),
            ),
            _ => panic!("unexpected BindingElement: {:?}", node.kind),
        };
        self.emit_token_node(dot_dot_dot_token);
        if let Some(property_name) = property_name {
            self.emit_property_name(Some(property_name));
            self.write_punctuation(":");
            self.write_space();
        }
        if let Some(name) = name {
            let name_end = name.end();
            self.emit_binding_name(name);
            self.emit_initializer(initializer, name_end, node);
        }
        self.exit_node(node, state);
    }

    pub(crate) fn emit_import_specifier_node(&mut self, node: &Node) { ::tsox_core::fntrace::enter("emit_import_specifier_node"); 
        let state = self.enter_node(node);
        let (is_type_only, property_name, name) = match &node.data {
            NodeData::ImportSpecifier(d) => (d.is_type_only, d.property_name.as_deref(), &d.name),
            _ => panic!("unexpected ImportSpecifier: {:?}", node.kind),
        };
        if is_type_only {
            self.write_keyword("type");
            self.write_space();
        }
        if let Some(property_name) = property_name {
            self.emit_module_export_name(Some(property_name));
            self.write_space();
            self.emit_token(SyntaxKind::AsKeyword, property_name.end(), WriteKind::Keyword, node);
            self.write_space();
        }
        self.emit_binding_identifier(name);
        self.exit_node(node, state);
    }

    pub(crate) fn emit_big_int_literal(&mut self, node: &Node) { ::tsox_core::fntrace::enter("emit_big_int_literal"); 
        let state = self.enter_node(node);
        self.emit_literal(node, GetLiteralTextFlags::NONE);
        self.exit_node(node, state);
    }

    pub(crate) fn emit_computed_property_name(&mut self, node: &Node) { ::tsox_core::fntrace::enter("emit_computed_property_name"); 
        let state = self.enter_node(node);
        let expression = match &node.data {
            NodeData::ComputedPropertyName(d) => &d.expression,
            _ => panic!("unexpected ComputedPropertyName: {:?}", node.kind),
        };
        self.write_punctuation("[");
        self.emit_expression(expression, OPERATOR_PRECEDENCE_DISALLOW_COMMA);
        self.write_punctuation("]");
        self.exit_node(node, state);
    }

    pub(crate) fn emit_entity_name(&mut self, node: &Node) { ::tsox_core::fntrace::enter("emit_entity_name"); 
        match node.kind {
            SyntaxKind::Identifier => self.emit_identifier_reference(node),
            SyntaxKind::QualifiedName => self.emit_qualified_name(node),
            SyntaxKind::PropertyAccessExpression => {
                self.emit_expression(node, OPERATOR_PRECEDENCE_DISALLOW_COMMA);
            }
            _ => panic!("unexpected EntityName: {:?}", node.kind),
        }
    }

    pub(crate) fn emit_initializer(&mut self, node: Option<&Node>, equal_token_pos: usize, context_node: &Node) { ::tsox_core::fntrace::enter("emit_initializer"); 
        let Some(node) = node else {
            return;
        };
        self.write_space();
        self.emit_token(SyntaxKind::EqualsToken, equal_token_pos, WriteKind::Operator, context_node);
        self.write_space();
        self.emit_expression(node, OPERATOR_PRECEDENCE_DISALLOW_COMMA);
    }

    pub(crate) fn emit_argument(&mut self, node: &Node) { ::tsox_core::fntrace::enter("emit_argument"); 
        self.emit_expression(node, OPERATOR_PRECEDENCE_DISALLOW_COMMA);
    }

    pub(crate) fn emit_accessor_declaration(&mut self, token: SyntaxKind, node: &Node) { ::tsox_core::fntrace::enter("emit_accessor_declaration"); 
        let state = self.enter_node(node);
        let (modifiers, name, body) = match &node.data {
            NodeData::GetAccessorDeclaration(d) => (d.modifiers.as_deref(), &d.name, d.body.as_deref()),
            NodeData::SetAccessorDeclaration(d) => (d.modifiers.as_deref(), &d.name, d.body.as_deref()),
            _ => panic!("unexpected AccessorDeclaration: {:?}", node.kind),
        };
        let pos = self.emit_modifier_list(node, modifiers, true);
        self.emit_token(token, pos, WriteKind::Keyword, node);
        self.write_space();
        self.emit_property_name(Some(name.as_ref()));
        let indented = self.should_emit_indented(node);
        self.increase_indent_if(indented);
        self.push_name_generation_scope(node);
        self.emit_signature(node);
        self.emit_function_body_node(body);
        self.pop_name_generation_scope(node);
        self.decrease_indent_if(indented);
        self.exit_node(node, state);
    }

    pub(crate) fn emit_get_accessor_declaration(&mut self, node: &Node) { ::tsox_core::fntrace::enter("emit_get_accessor_declaration"); 
        self.emit_accessor_declaration(SyntaxKind::GetKeyword, node);
    }
}
