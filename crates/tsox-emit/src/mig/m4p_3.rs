#![allow(dead_code, unused_imports, unused_variables)]

use std::sync::Arc;

use tsox_frontend::ast::node::{Node, NodeList};
use tsox_frontend::ast::node_data_generated::{is_keyword_kind, is_punctuation_kind, NodeData};
use tsox_frontend::ast::ModifierList;
use tsox_frontend::ast::syntax_kind_generated::SyntaxKind;
use tsox_frontend::ast::utilities::is_optional_chain;

use tsox_frontend::format::mig::m4o::TokenEmitFlags;
use tsox_frontend::format::mig::m4o::{EmitFlags, ListFormat};
use tsox_frontend::format::mig::m4o_2::WriteKind;
use tsox_frontend::ast::mig::m3e_3::{
    get_type_node_precedence, OperatorPrecedence, TypePrecedence, OPERATOR_PRECEDENCE_DISALLOW_COMMA,
    OPERATOR_PRECEDENCE_LOWEST, TYPE_PRECEDENCE_LOWEST,
};

use super::m4p::{CommentState, Printer};

const INTERSECTION_TYPE_CONSTITUENTS: ListFormat =
    ListFormat(ListFormat::BAR_DELIMITED.0 | ListFormat::SPACE_BETWEEN_SIBLINGS.0);
const IMPORT_ATTRIBUTES: ListFormat = ListFormat(
    ListFormat::PRESERVE_LINES.0
        | ListFormat::COMMA_DELIMITED.0
        | ListFormat::SPACE_BETWEEN_SIBLINGS.0
        | ListFormat::SPACE_BETWEEN_BRACES.0
        | ListFormat::INDENTED.0
        | ListFormat::BRACES.0
        | ListFormat::NO_SPACE_IF_EMPTY.0,
);
const LF_MODIFIERS: ListFormat = ListFormat(
    ListFormat::SPACE_BETWEEN_SIBLINGS.0
        | ListFormat::NO_INTERVENING_COMMENTS.0
        | ListFormat::SPACE_AFTER_LIST.0,
);
const LF_TYPE_PARAMETERS: ListFormat = ListFormat(
    ListFormat::COMMA_DELIMITED.0
        | ListFormat::SPACE_BETWEEN_SIBLINGS.0
        | ListFormat::ANGLE_BRACKETS.0
        | ListFormat::OPTIONAL.0,
);
const LF_TYPE_ARGUMENTS: ListFormat = ListFormat(
    ListFormat::COMMA_DELIMITED.0
        | ListFormat::SPACE_BETWEEN_SIBLINGS.0
        | ListFormat::ANGLE_BRACKETS.0
        | ListFormat::OPTIONAL.0,
);

pub struct PrinterState {
    pub comment_state: Option<CommentState>,
}

impl Default for PrinterState {
    fn default() -> Self { ::tsox_core::fntrace::enter("default"); 
        Self { comment_state: None }
    }
}

fn intersection_type_types(node: &Arc<Node>) -> &Arc<NodeList> { ::tsox_core::fntrace::enter("intersection_type_types"); 
    match &node.data {
        NodeData::IntersectionTypeNode(d) => &d.types,
        _ => panic!("unexpected IntersectionType: {:?}", node.kind),
    }
}

fn type_parameter_constraint(node: &Arc<Node>) -> Option<&Arc<Node>> { ::tsox_core::fntrace::enter("type_parameter_constraint"); 
    match &node.data {
        NodeData::TypeParameterDeclaration(d) => d.constraint.as_ref(),
        _ => None,
    }
}

fn infer_type_type_parameter(node: &Arc<Node>) -> &Arc<Node> { ::tsox_core::fntrace::enter("infer_type_type_parameter"); 
    match &node.data {
        NodeData::InferTypeNode(d) => &d.type_parameter,
        _ => panic!("unexpected InferType: {:?}", node.kind),
    }
}

fn indexed_access_object_type(node: &Arc<Node>) -> &Arc<Node> { ::tsox_core::fntrace::enter("indexed_access_object_type"); 
    match &node.data {
        NodeData::IndexedAccessTypeNode(d) => &d.object_type,
        _ => panic!("unexpected IndexedAccessType: {:?}", node.kind),
    }
}

fn indexed_access_index_type(node: &Arc<Node>) -> &Arc<Node> { ::tsox_core::fntrace::enter("indexed_access_index_type"); 
    match &node.data {
        NodeData::IndexedAccessTypeNode(d) => &d.index_type,
        _ => panic!("unexpected IndexedAccessType: {:?}", node.kind),
    }
}

fn mapped_type_readonly_token(node: &Arc<Node>) -> Option<&Arc<Node>> { ::tsox_core::fntrace::enter("mapped_type_readonly_token"); 
    match &node.data {
        NodeData::MappedTypeNode(d) => d.readonly_token.as_ref(),
        _ => None,
    }
}

fn mapped_type_type_parameter(node: &Arc<Node>) -> &Arc<Node> { ::tsox_core::fntrace::enter("mapped_type_type_parameter"); 
    match &node.data {
        NodeData::MappedTypeNode(d) => &d.type_parameter,
        _ => panic!("unexpected MappedType: {:?}", node.kind),
    }
}

fn mapped_type_name_type(node: &Arc<Node>) -> Option<&Arc<Node>> { ::tsox_core::fntrace::enter("mapped_type_name_type"); 
    match &node.data {
        NodeData::MappedTypeNode(d) => d.name_type.as_ref(),
        _ => None,
    }
}

fn mapped_type_question_token(node: &Arc<Node>) -> Option<&Arc<Node>> { ::tsox_core::fntrace::enter("mapped_type_question_token"); 
    match &node.data {
        NodeData::MappedTypeNode(d) => d.question_token.as_ref(),
        _ => None,
    }
}

fn mapped_type_members(node: &Arc<Node>) -> Option<&Arc<NodeList>> { ::tsox_core::fntrace::enter("mapped_type_members"); 
    match &node.data {
        NodeData::MappedTypeNode(d) => d.members.as_ref(),
        _ => None,
    }
}

fn literal_type_literal(node: &Arc<Node>) -> &Arc<Node> { ::tsox_core::fntrace::enter("literal_type_literal"); 
    match &node.data {
        NodeData::LiteralTypeNode(d) => &d.literal,
        _ => panic!("unexpected LiteralType: {:?}", node.kind),
    }
}

fn import_attributes_token(node: &Arc<Node>) -> SyntaxKind { ::tsox_core::fntrace::enter("import_attributes_token"); 
    match &node.data {
        NodeData::ImportAttributes(d) => d.token,
        _ => panic!("unexpected ImportAttributes: {:?}", node.kind),
    }
}

fn import_attributes_attributes(node: &Arc<Node>) -> &Arc<NodeList> { ::tsox_core::fntrace::enter("import_attributes_attributes"); 
    match &node.data {
        NodeData::ImportAttributes(d) => &d.attributes,
        _ => panic!("unexpected ImportAttributes: {:?}", node.kind),
    }
}

fn import_type_is_type_of(node: &Arc<Node>) -> bool { ::tsox_core::fntrace::enter("import_type_is_type_of"); 
    match &node.data {
        NodeData::ImportTypeNode(d) => d.is_type_of,
        _ => false,
    }
}

fn import_type_argument(node: &Arc<Node>) -> &Arc<Node> { ::tsox_core::fntrace::enter("import_type_argument"); 
    match &node.data {
        NodeData::ImportTypeNode(d) => &d.argument,
        _ => panic!("unexpected ImportType: {:?}", node.kind),
    }
}

fn import_type_attributes(node: &Arc<Node>) -> Option<&Arc<Node>> { ::tsox_core::fntrace::enter("import_type_attributes"); 
    match &node.data {
        NodeData::ImportTypeNode(d) => d.attributes.as_ref(),
        _ => None,
    }
}

fn import_type_qualifier(node: &Arc<Node>) -> Option<&Arc<Node>> { ::tsox_core::fntrace::enter("import_type_qualifier"); 
    match &node.data {
        NodeData::ImportTypeNode(d) => d.qualifier.as_ref(),
        _ => None,
    }
}

fn element_access_argument_expression(node: &Arc<Node>) -> &Arc<Node> { ::tsox_core::fntrace::enter("element_access_argument_expression"); 
    match &node.data {
        NodeData::ElementAccessExpression(d) => &d.argument_expression,
        _ => panic!("unexpected ElementAccessExpression: {:?}", node.kind),
    }
}

fn function_expression_asterisk_token(node: &Arc<Node>) -> Option<&Arc<Node>> { ::tsox_core::fntrace::enter("function_expression_asterisk_token"); 
    match &node.data {
        NodeData::FunctionExpression(d) => d.asterisk_token.as_ref(),
        _ => None,
    }
}

fn greatest_node_end(end: usize, nodes: &[Option<&Arc<Node>>]) -> usize { ::tsox_core::fntrace::enter("greatest_node_end"); 
    let mut end = end;
    for node in nodes.iter().rev() {
        if let Some(node_end) = node.map(|n| n.end()) {
            if end < node_end {
                end = node_end;
            }
        }
    }
    end
}

impl Printer {
    pub fn emit_function_type(&mut self, node: &Arc<Node>) { ::tsox_core::fntrace::enter("emit_function_type"); 
        let state = self.enter_node(node);
        let indented = self.should_emit_indented(node);
        self.increase_indent_if(indented);
        self.emit_type_parameters(node, node.type_parameters());
        self.emit_parameters(node, node.parameters().unwrap());
        self.write_space();
        self.emit_return_type(node.type_node().unwrap());
        self.decrease_indent_if(indented);
        self.exit_node(node, state);
    }

    pub fn emit_intersection_type_constituent(&mut self, node: &Arc<Node>) { ::tsox_core::fntrace::enter("emit_intersection_type_constituent"); 
        self.emit_type_node(node, TypePrecedence::TypeOperator);
    }

    pub fn emit_intersection_type(&mut self, node: &Arc<Node>) { ::tsox_core::fntrace::enter("emit_intersection_type"); 
        let state = self.enter_node(node);
        self.emit_list(
            Self::emit_intersection_type_constituent,
            node,
            intersection_type_types(node),
            INTERSECTION_TYPE_CONSTITUENTS,
        );
        self.exit_node(node, state);
    }

    pub fn emit_infer_type_parameter(&mut self, node: &Arc<Node>) { ::tsox_core::fntrace::enter("emit_infer_type_parameter"); 
        let state = self.enter_node(node);
        self.emit_binding_identifier(node.name().unwrap());
        if let Some(constraint) = type_parameter_constraint(node) {
            self.write_space();
            self.write_keyword("extends");
            self.write_space();
            self.emit_type_node_in_extends(constraint);
        }
        self.exit_node(node, state);
    }

    pub fn emit_infer_type(&mut self, node: &Arc<Node>) { ::tsox_core::fntrace::enter("emit_infer_type"); 
        let state = self.enter_node(node);
        self.write_keyword("infer");
        self.write_space();
        self.emit_infer_type_parameter(infer_type_type_parameter(node));
        self.exit_node(node, state);
    }

    pub fn emit_indexed_access_type(&mut self, node: &Arc<Node>) { ::tsox_core::fntrace::enter("emit_indexed_access_type"); 
        let state = self.enter_node(node);
        self.emit_postfix_type_operand(indexed_access_object_type(node), node);
        self.write_punctuation("[");
        self.emit_type_node_outside_extends(indexed_access_index_type(node));
        self.write_punctuation("]");
        self.exit_node(node, state);
    }

    pub fn emit_mapped_type_parameter(&mut self, node: &Arc<Node>) { ::tsox_core::fntrace::enter("emit_mapped_type_parameter"); 
        let state = self.enter_node(node);
        self.emit_binding_identifier(node.name().unwrap());
        self.write_space();
        self.write_keyword("in");
        self.write_space();
        self.emit_type_node_outside_extends(
            type_parameter_constraint(node).expect("mapped type parameter requires constraint"),
        );
        self.exit_node(node, state);
    }

    pub fn emit_mapped_type(&mut self, node: &Arc<Node>) { ::tsox_core::fntrace::enter("emit_mapped_type"); 
        let state = self.enter_node(node);
        let single_line = self.should_emit_on_single_line(node);
        self.write_punctuation("{");
        if single_line {
            self.write_space();
        } else {
            self.write_line();
            self.increase_indent();
        }
        if let Some(readonly_token) = mapped_type_readonly_token(node) {
            self.emit_token_node(Some(readonly_token));
            if readonly_token.kind != SyntaxKind::ReadonlyKeyword {
                self.write_keyword("readonly");
            }
            self.write_space();
        }
        self.write_punctuation("[");
        self.emit_mapped_type_parameter(mapped_type_type_parameter(node));
        if let Some(name_type) = mapped_type_name_type(node) {
            self.write_space();
            self.write_keyword("as");
            self.write_space();
            self.emit_type_node_outside_extends(name_type);
        }
        self.write_punctuation("]");
        if let Some(question_token) = mapped_type_question_token(node) {
            self.emit_punctuation_node(question_token);
            if question_token.kind != SyntaxKind::QuestionToken {
                self.write_punctuation("?");
            }
        }
        if let Some(type_node) = node.type_node() {
            self.write_punctuation(":");
            self.write_space();
            self.emit_type_node_outside_extends(type_node);
        }
        self.write_trailing_semicolon();
        if let Some(members) = mapped_type_members(node) {
            if !members.nodes.is_empty() {
                if single_line {
                    self.write_space();
                } else {
                    self.write_line();
                }
                self.emit_list(Self::emit_type_element, node, members, ListFormat::PRESERVE_LINES);
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

    pub fn emit_literal_type(&mut self, node: &Arc<Node>) { ::tsox_core::fntrace::enter("emit_literal_type"); 
        let state = self.enter_node(node);
        self.emit_expression(literal_type_literal(node), OperatorPrecedence::Comma);
        self.exit_node(node, state);
    }

    pub fn emit_import_type_node_attributes(&mut self, node: &Arc<Node>) { ::tsox_core::fntrace::enter("emit_import_type_node_attributes"); 
        let state = self.enter_node(node);
        self.write_punctuation("{");
        self.write_space();
        self.write_keyword(if import_attributes_token(node) == SyntaxKind::AssertKeyword {
            "assert"
        } else {
            "with"
        });
        self.write_punctuation(":");
        self.write_space();
        self.emit_list(
            Self::emit_import_attribute_node,
            node,
            import_attributes_attributes(node),
            IMPORT_ATTRIBUTES,
        );
        self.write_space();
        self.write_punctuation("}");
        self.exit_node(node, state);
    }

    pub fn emit_import_type_node(&mut self, node: &Arc<Node>) { ::tsox_core::fntrace::enter("emit_import_type_node"); 
        let state = self.enter_node(node);
        if import_type_is_type_of(node) {
            self.write_keyword("typeof");
            self.write_space();
        }
        self.write_keyword("import");
        self.write_punctuation("(");
        self.emit_type_node_outside_extends(import_type_argument(node));
        if let Some(attributes) = import_type_attributes(node) {
            self.write_punctuation(",");
            self.write_space();
            self.emit_import_type_node_attributes(attributes);
        }
        self.write_punctuation(")");
        if let Some(qualifier) = import_type_qualifier(node) {
            self.write_punctuation(".");
            self.emit_entity_name(qualifier);
        }
        self.emit_type_arguments(node, node.type_arguments());
        self.exit_node(node, state);
    }

    pub fn emit_element_access_expression(&mut self, node: &Arc<Node>) { ::tsox_core::fntrace::enter("emit_element_access_expression"); 
        let state = self.enter_node(node);
        let expression = node.expression().unwrap();
        let argument_expression = element_access_argument_expression(node);
        self.emit_expression(
            expression,
            if is_optional_chain(node) {
                OperatorPrecedence::OptionalChain
            } else {
                OperatorPrecedence::Member
            },
        );
        self.emit_token_node(node.question_dot_token());
        self.emit_token(
            SyntaxKind::OpenBracketToken,
            greatest_node_end(expression.pos(), &[node.question_dot_token()]),
            WriteKind::Punctuation,
            node,
        );
        self.emit_expression(argument_expression, OperatorPrecedence::Comma);
        self.emit_token(
            SyntaxKind::CloseBracketToken,
            argument_expression.end(),
            WriteKind::Punctuation,
            node,
        );
        self.exit_node(node, state);
    }

    pub fn emit_function_expression(&mut self, node: &Arc<Node>) { ::tsox_core::fntrace::enter("emit_function_expression"); 
        let state = self.enter_node(node);
        self.generate_name_if_needed(node.name());
        self.emit_modifier_list(node, node.modifiers(), false);
        self.write_keyword("function");
        self.emit_token_node(function_expression_asterisk_token(node));
        self.write_space();
        self.emit_identifier_name_node(node.name());
        let indented = self.should_emit_indented(node);
        self.increase_indent_if(indented);
        self.emit_signature(node);
        self.emit_function_body_node(node.body());
        self.decrease_indent_if(indented);
        self.exit_node(node, state);
    }

    pub fn emit_expression_with_type_arguments(&mut self, node: &Arc<Node>) { ::tsox_core::fntrace::enter("emit_expression_with_type_arguments"); 
        let state = self.enter_node(node);
        self.emit_expression(node.expression().unwrap(), OperatorPrecedence::Member);
        self.emit_type_arguments(node, node.type_arguments());
        self.exit_node(node, state);
    }

    pub fn emit_while_clause(
        &mut self,
        node: &Arc<Node>,
        expression: &Arc<Node>,
        start_pos: usize,
    ) { ::tsox_core::fntrace::enter("emit_while_clause"); 
        let pos = self.emit_token(SyntaxKind::WhileKeyword, start_pos, WriteKind::Keyword, node);
        self.write_space();
        self.emit_token(SyntaxKind::OpenParenToken, pos, WriteKind::Punctuation, node);
        self.emit_expression(expression, OPERATOR_PRECEDENCE_LOWEST);
        self.emit_token(SyntaxKind::CloseParenToken, expression.end(), WriteKind::Punctuation, node);
    }

    pub fn emit_type_element(printer: &mut Printer, node: &Arc<Node>) { ::tsox_core::fntrace::enter("emit_type_element"); 
        match node.kind {
            SyntaxKind::IndexSignature => printer.emit_index_signature(node),
            SyntaxKind::GetAccessor => printer.emit_get_accessor_declaration(node),
            other => panic!("unexpected TypeElement: {:?}", other),
        }
    }
}

impl Printer {
    pub fn write_as(&mut self, text: &str, write_kind: WriteKind) { ::tsox_core::fntrace::enter("write_as"); 
        match write_kind {
            WriteKind::None => self.writer.write(text),
            WriteKind::Keyword => self.writer.write_keyword(text),
            WriteKind::Operator => self.writer.write_operator(text),
            WriteKind::Punctuation => self.writer.write_punctuation(text),
            WriteKind::StringLiteral => self.writer.write_string_literal(text),
            WriteKind::Parameter => self.writer.write_parameter(text),
            WriteKind::Property => self.writer.write_property(text),
            WriteKind::Comment => self.writer.write_comment(text),
            WriteKind::Literal => self.writer.write_literal(text),
        }
    }

    pub fn write_space(&mut self) { ::tsox_core::fntrace::enter("write_space"); 
        self.writer.write_space(" ");
    }

    pub fn write_line(&mut self) { ::tsox_core::fntrace::enter("write_line"); 
        self.writer.write_line();
    }

    pub fn write_punctuation(&mut self, text: &str) { ::tsox_core::fntrace::enter("write_punctuation"); 
        self.writer.write_punctuation(text);
    }

    pub fn write_keyword(&mut self, text: &str) { ::tsox_core::fntrace::enter("write_keyword"); 
        self.writer.write_keyword(text);
    }

    pub fn write_trailing_semicolon(&mut self) { ::tsox_core::fntrace::enter("write_trailing_semicolon"); 
        self.writer.write_trailing_semicolon(";");
    }

    pub fn increase_indent(&mut self) { ::tsox_core::fntrace::enter("increase_indent"); 
        self.writer.increase_indent();
    }

    pub fn decrease_indent(&mut self) { ::tsox_core::fntrace::enter("decrease_indent"); 
        self.writer.decrease_indent();
    }

    pub fn increase_indent_if(&mut self, indent_requested: bool) { ::tsox_core::fntrace::enter("increase_indent_if"); 
        if indent_requested {
            self.increase_indent();
        }
    }

    pub fn decrease_indent_if(&mut self, indent_requested: bool) { ::tsox_core::fntrace::enter("decrease_indent_if"); 
        if indent_requested {
            self.decrease_indent();
        }
    }

    pub fn should_emit_indented(&self, node: &Arc<Node>) -> bool { ::tsox_core::fntrace::enter("should_emit_indented"); 
        let flags = self.emit_context.emit_flags(node);
        flags.intersects(EmitFlags::INDENTED) && !flags.intersects(EmitFlags::NO_INDENTATION)
    }

    pub fn should_emit_on_single_line(&self, node: &Arc<Node>) -> bool { ::tsox_core::fntrace::enter("should_emit_on_single_line"); 
        self.emit_context.emit_flags(node).intersects(EmitFlags::SINGLE_LINE)
    }

    pub fn enter_node(&mut self, node: &Arc<Node>) -> PrinterState { ::tsox_core::fntrace::enter("enter_node"); 
        let mut state = PrinterState::default();
        if let Some(on_before_emit_node) = &self.on_before_emit_node {
            on_before_emit_node(node);
        }
        state.comment_state = self.emit_comments_before_node(node);
        state
    }

    pub fn exit_node(&mut self, node: &Arc<Node>, previous_state: PrinterState) { ::tsox_core::fntrace::enter("exit_node"); 
        self.emit_comments_after_node(node, previous_state.comment_state);
        if let Some(on_after_emit_node) = &self.on_after_emit_node {
            on_after_emit_node(node);
        }
    }

    pub fn emit_comments_before_node(&mut self, node: &Arc<Node>) -> Option<CommentState> { ::tsox_core::fntrace::enter("emit_comments_before_node"); 
        if self.comments_disabled {
            return None;
        }
        let emit_flags = self.emit_context.emit_flags(node);
        let comment_range = self.emit_context.comment_range(node);
        let container_pos = self.container_pos;
        let container_end = self.container_end;
        let declaration_list_container_end = self.declaration_list_container_end;
        self.emit_leading_comments_of_node(node, emit_flags, comment_range);
        self.emit_leading_synthetic_comments_of_node(node, emit_flags);
        if emit_flags.intersects(EmitFlags::NO_NESTED_COMMENTS) {
            self.comments_disabled = true;
        }
        Some(CommentState::new(
            emit_flags,
            comment_range,
            container_pos,
            container_end,
            declaration_list_container_end,
        ))
    }

    pub fn emit_comments_after_node(&mut self, node: &Arc<Node>, previous_state: Option<CommentState>) { ::tsox_core::fntrace::enter("emit_comments_after_node"); 
        let Some(state) = previous_state else {
            return;
        };
        if state.emit_flags.intersects(EmitFlags::NO_NESTED_COMMENTS) {
            self.comments_disabled = false;
        }
    }

    pub fn enter_token_node(&mut self, node: &Arc<Node>, flags: TokenEmitFlags) -> PrinterState { ::tsox_core::fntrace::enter("enter_token_node"); 
        let mut state = PrinterState::default();
        if flags.0 & TokenEmitFlags::NO_COMMENTS.0 == 0 {
            state.comment_state = self.emit_comments_before_node(node);
        }
        state
    }

    pub fn exit_token_node(&mut self, node: &Arc<Node>, previous_state: PrinterState) { ::tsox_core::fntrace::enter("exit_token_node"); 
        self.emit_comments_after_node(node, previous_state.comment_state);
    }

    pub fn write_token_text(&mut self, token: SyntaxKind, write_kind: WriteKind, pos: usize) -> usize { ::tsox_core::fntrace::enter("write_token_text"); 
        let text = tsox_frontend::scanner::token_to_string(token);
        self.write_as(text, write_kind);
        pos + text.len()
    }

    pub fn emit_token(
        &mut self,
        token: SyntaxKind,
        pos: usize,
        write_kind: WriteKind,
        context_node: &Arc<Node>,
    ) -> usize { ::tsox_core::fntrace::enter("emit_token"); 
        let state = self.enter_token_node(context_node, TokenEmitFlags::NONE);
        let pos = self.write_token_text(token, write_kind, pos);
        self.exit_token_node(context_node, state);
        pos
    }

    pub fn emit_token_node_ex(&mut self, node: &Arc<Node>, flags: TokenEmitFlags) { ::tsox_core::fntrace::enter("emit_token_node_ex"); 
        let state = self.enter_token_node(node, flags);
        let write_kind = if is_keyword_kind(node.kind) {
            WriteKind::Keyword
        } else if is_punctuation_kind(node.kind) {
            WriteKind::Punctuation
        } else {
            WriteKind::None
        };
        self.write_token_text(node.kind, write_kind, node.pos());
        self.exit_token_node(node, state);
    }

    pub fn emit_token_node(&mut self, node: Option<&Arc<Node>>) { ::tsox_core::fntrace::enter("emit_token_node"); 
        let Some(node) = node else {
            return;
        };
        self.emit_token_node_ex(node, TokenEmitFlags::NONE);
    }

    pub fn emit_punctuation_node(&mut self, node: &Arc<Node>) { ::tsox_core::fntrace::enter("emit_punctuation_node"); 
        self.emit_token_node_ex(node, TokenEmitFlags::NONE);
    }

    pub fn emit_type_node(&mut self, node: &Arc<Node>, precedence: TypePrecedence) { ::tsox_core::fntrace::enter("emit_type_node"); 
        let parens = (get_type_node_precedence(node) as i32) < (precedence as i32);
        if parens {
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
            SyntaxKind::JSDocNonNullableType => self.emit_jsdoc_non_nullable_type(node),
            SyntaxKind::JSDocNullableType => self.emit_jsdoc_nullable_type(node),
            SyntaxKind::JSDocOptionalType => self.emit_jsdoc_optional_type(node),
            SyntaxKind::JSDocVariadicType => self.emit_jsdoc_variadic_type(node),
            SyntaxKind::FunctionType => self.emit_function_type(node),
            SyntaxKind::IntersectionType => self.emit_intersection_type(node),
            SyntaxKind::InferType => self.emit_infer_type(node),
            SyntaxKind::IndexedAccessType => self.emit_indexed_access_type(node),
            SyntaxKind::MappedType => self.emit_mapped_type(node),
            SyntaxKind::LiteralType => self.emit_literal_type(node),
            SyntaxKind::ImportType => self.emit_import_type_node(node),
            other => {
                let text = node.text().to_string();
                self.write_as(&text, WriteKind::None);
                let _ = other;
            }
        }
        if parens {
            self.write_punctuation(")");
        }
    }

    pub fn emit_type_node_in_extends(&mut self, node: &Arc<Node>) { ::tsox_core::fntrace::enter("emit_type_node_in_extends"); 
        self.emit_type_node(node, TYPE_PRECEDENCE_LOWEST);
    }

    pub fn emit_type_node_outside_extends(&mut self, node: &Arc<Node>) { ::tsox_core::fntrace::enter("emit_type_node_outside_extends"); 
        self.emit_type_node(node, TYPE_PRECEDENCE_LOWEST);
    }

    pub fn emit_postfix_type_operand(&mut self, operand: &Arc<Node>, parent_node: &Arc<Node>) { ::tsox_core::fntrace::enter("emit_postfix_type_operand"); 
        if operand.kind == SyntaxKind::TypeQuery {
            self.emit_type_node(operand, TypePrecedence::TypeOperator);
            return;
        }
        self.emit_type_node(operand, TypePrecedence::Postfix);
    }

    pub fn emit_return_type(&mut self, node: &Arc<Node>) { ::tsox_core::fntrace::enter("emit_return_type"); 
        self.write_punctuation("=>");
        self.write_space();
        self.emit_type_node(node, TYPE_PRECEDENCE_LOWEST);
    }

    pub fn emit_binding_identifier(&mut self, node: &Arc<Node>) { ::tsox_core::fntrace::enter("emit_binding_identifier"); 
        let mut node = node.clone();
        if self.unique_helper_names.is_some()
            && self.emit_context.emit_flags(&node).intersects(EmitFlags::HELPER_NAME)
        {
            let helper_name = self.get_unique_helper_name(node.text());
            if let Some(emit_context) = Arc::get_mut(&mut self.emit_context) {
                emit_context.assign_comment_and_source_map_ranges(&helper_name, &node);
            }
            node = helper_name;
        }
        let state = self.enter_node(&node);
        self.emit_identifier_text(&node);
        self.exit_node(&node, state);
    }

    pub fn emit_type_annotation(&mut self, node: Option<&Arc<Node>>) { ::tsox_core::fntrace::enter("emit_type_annotation"); 
        let Some(node) = node else {
            return;
        };
        self.write_punctuation(":");
        self.write_space();
        self.emit_type_node_outside_extends(node);
    }

    pub fn emit_type_parameter_declaration_node(&mut self, node: &Arc<Node>) { ::tsox_core::fntrace::enter("emit_type_parameter_declaration_node"); 
        if let Some(name) = node.name() {
            self.emit_identifier_name(name);
        }
        if let Some(constraint) = type_parameter_constraint(node) {
            self.write_space();
            self.write_keyword("extends");
            self.write_space();
            self.emit_type_node_outside_extends(constraint);
        }
    }

    pub fn emit_type_parameters(&mut self, parent_node: &Arc<Node>, nodes: Option<&Arc<NodeList>>) { ::tsox_core::fntrace::enter("emit_type_parameters"); 
        let Some(nodes) = nodes else {
            return;
        };
        self.emit_list(
            Self::emit_type_parameter_declaration_node,
            parent_node,
            nodes,
            LF_TYPE_PARAMETERS,
        );
    }

    pub fn emit_parameter_declaration_node(&mut self, node: &Arc<Node>) { ::tsox_core::fntrace::enter("emit_parameter_declaration_node"); 
        self.emit_token_node(node.dot_dot_dot_token());
        if let Some(name) = node.name() {
            self.emit_binding_identifier(name);
        }
        self.emit_type_annotation(node.type_());
        let name_end = node.name().map(|n| n.end()).unwrap_or_else(|| node.pos());
        self.emit_initializer(node.initializer(), name_end, node);
    }

    pub fn emit_parameters(&mut self, parent_node: &Arc<Node>, parameters: &Arc<NodeList>) { ::tsox_core::fntrace::enter("emit_parameters"); 
        self.write_punctuation("(");
        self.emit_list(
            Self::emit_parameter_declaration_node,
            parent_node,
            parameters,
            ListFormat(ListFormat::COMMA_DELIMITED.0 | ListFormat::SPACE_BETWEEN_SIBLINGS.0),
        );
        self.write_punctuation(")");
    }

    pub fn emit_signature(&mut self, node: &Arc<Node>) { ::tsox_core::fntrace::enter("emit_signature"); 
        self.emit_type_parameters(node, node.type_parameters());
        if let Some(parameters) = node.parameters() {
            self.emit_parameters(node, parameters);
        }
        self.emit_type_annotation(node.type_());
    }

    pub fn emit_type_arguments(&mut self, parent_node: &Arc<Node>, nodes: Option<&Arc<NodeList>>) { ::tsox_core::fntrace::enter("emit_type_arguments"); 
        let Some(nodes) = nodes else {
            return;
        };
        self.emit_list(
            |printer, node| printer.emit_type_node_outside_extends(node),
            parent_node,
            nodes,
            LF_TYPE_ARGUMENTS,
        );
    }

    pub fn emit_modifier_list(
        &mut self,
        parent_node: &Arc<Node>,
        modifiers: Option<&Arc<ModifierList>>,
        allow_decorators: bool,
    ) -> usize { ::tsox_core::fntrace::enter("emit_modifier_list"); 
        let Some(modifiers) = modifiers else {
            return parent_node.pos();
        };
        let nodes = &modifiers.list.nodes;
        if nodes.is_empty() {
            return parent_node.pos();
        }
        let has_decorator = nodes.iter().any(|n| n.kind == SyntaxKind::Decorator);
        if !has_decorator {
            for (i, modifier) in nodes.iter().enumerate() {
                if i > 0 {
                    self.write_space();
                }
                self.emit_keyword_node(modifier);
            }
        } else {
            if !allow_decorators {
                return parent_node.pos();
            }
            for (i, decorator) in nodes.iter().enumerate() {
                if i > 0 {
                    self.write_space();
                }
                self.write_punctuation("@");
                if let Some(expression) = decorator.expression() {
                    self.emit_expression(expression, OPERATOR_PRECEDENCE_DISALLOW_COMMA);
                }
            }
        }
        nodes
            .last()
            .map(|m| m.end())
            .unwrap_or_else(|| parent_node.pos())
    }
}
