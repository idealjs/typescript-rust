#![allow(dead_code, unused_imports, unused_variables)]

use std::sync::Arc;

use tsox_core::core::text::TextRange;
use tsox_frontend::ast::node::{Node, NodeList};
use tsox_frontend::ast::node_data_generated::NodeData;
use tsox_frontend::ast::syntax_kind_generated::SyntaxKind;
use tsox_frontend::ast::utilities::position_is_synthesized;
use tsox_frontend::ast::visitor::NodeVisitor;
use tsox_frontend::ast::mig::m3e_3::{OperatorPrecedence, OPERATOR_PRECEDENCE_DISALLOW_COMMA};
use tsox_frontend::format::mig::m4o::{CommentSeparator, EmitFlags, ListFormat, TokenEmitFlags};
use tsox_frontend::format::mig::m4o_2::WriteKind;

use crate::mig::m4k_2::Transformer;
use crate::mig::m4p::{CommentState, Printer};
use tsox_frontend::format::mig::m4t_3::range_is_on_single_line;

impl Printer {
    pub fn push_name_generation_scope(&mut self, _node: &Arc<Node>) {}

    pub fn pop_name_generation_scope(&mut self, _node: &Arc<Node>) {}

    pub fn generate_names(&mut self, _node: &Arc<Node>) {}

    pub fn generate_all_member_names(&mut self, _nodes: &Arc<NodeList>) {}

    pub fn generate_name_if_needed(&mut self, _name: Option<&Arc<Node>>) {}

    pub fn emit_statement(&mut self, node: &Arc<Node>) {
        match node.kind {
            SyntaxKind::EmptyStatement => self.emit_empty_statement(node, false),
            SyntaxKind::ExpressionStatement => self.emit_expression_statement(node),
            SyntaxKind::IfStatement => self.emit_if_statement(node),
            SyntaxKind::DoStatement => self.emit_do_statement(node),
            SyntaxKind::ForStatement => self.emit_for_statement(node),
            SyntaxKind::ForInStatement => self.emit_for_in_statement(node),
            SyntaxKind::ForOfStatement => self.emit_for_of_statement(node),
            SyntaxKind::LabeledStatement => self.emit_labeled_statement(node),
            SyntaxKind::FunctionDeclaration => self.emit_function_declaration(node),
            SyntaxKind::InterfaceDeclaration => self.emit_interface_declaration(node),
            SyntaxKind::EnumDeclaration => self.emit_enum_declaration(node),
            SyntaxKind::ImportEqualsDeclaration => self.emit_import_equals_declaration(node),
            SyntaxKind::ImportDeclaration => self.emit_import_declaration(node),
            SyntaxKind::ExportAssignment => self.emit_export_assignment(node),
            SyntaxKind::ExportDeclaration => self.emit_export_declaration(node),
            SyntaxKind::NotEmittedStatement | SyntaxKind::MissingDeclaration => {}
            _ => panic!("emit_statement: statement kind not yet ported: {:?}", node.kind),
        }
    }

    pub fn emit_prologue_directives(&mut self, statements: &Arc<NodeList>) -> i64 {
        for (i, statement) in statements.nodes.iter().enumerate() {
            if is_prologue_directive(statement) {
                self.write_line();
                self.emit_statement(statement);
            } else {
                return i as i64;
            }
        }
        statements.nodes.len() as i64
    }

    pub fn emit_token_ex(
        &mut self,
        token: SyntaxKind,
        pos: usize,
        write_kind: WriteKind,
        context_node: &Arc<Node>,
        flags: TokenEmitFlags,
    ) -> usize {
        self.emit_token(token, pos, write_kind, context_node)
    }

    pub fn emit_detached_comments_after_statement_list(
        &mut self,
        node: &Arc<Node>,
        detached_range: TextRange,
        state: Option<CommentState>,
    ) {
        let Some(state) = state else { return };
        let skip_trailing_comments = self.comments_disabled
            || position_is_synthesized(detached_range.end())
            || state
                .emit_flags
                .intersects(EmitFlags::NO_TRAILING_COMMENTS);
        if !skip_trailing_comments {
            let has_written_comment = self.emit_leading_comments(detached_range.end(), false);
            if has_written_comment && !self.writer.is_at_start_of_line() {
                self.write_line();
            }
        }
    }

    pub fn emit_trailing_comments(&mut self, pos: usize, separator: CommentSeparator) {
        if self.comments_disabled || self.current_source_file.is_none() {
            return;
        }
        if pos == self.container_end || pos == self.declaration_list_container_end {
            return;
        }
    }

    pub fn get_leading_line_terminator_count(
        &mut self,
        parent_node: &Arc<Node>,
        first_child: Option<&Arc<Node>>,
        format: ListFormat,
    ) -> usize {
        if format.intersects(ListFormat::PRESERVE_LINES) || self.options.preserve_source_newlines {
            if format.intersects(ListFormat::PREFER_NEW_LINE) {
                return 1;
            }
            let Some(first_child) = first_child else {
                return match &self.current_source_file {
                    Some(file) => {
                        if range_is_on_single_line(parent_node.loc, file) {
                            0
                        } else {
                            1
                        }
                    }
                    None => 1,
                };
            };
            if self.next_list_element_pos > 0 && first_child.pos() == self.next_list_element_pos {
                return 0;
            }
            if first_child.kind == SyntaxKind::JsxText {
                return 0;
            }
            if self.current_source_file.is_some()
                && !position_is_synthesized(parent_node.pos())
                && !position_is_synthesized(first_child.loc.pos())
            {
                return match &self.current_source_file {
                    Some(file) => {
                        if range_is_on_single_line(parent_node.loc, file)
                            && range_is_on_single_line(first_child.loc, file)
                        {
                            0
                        } else {
                            1
                        }
                    }
                    None => 1,
                };
            }
            if self.should_emit_on_new_line(first_child, format) {
                return 1;
            }
        }
        0
    }

    pub fn should_emit_on_new_line(&self, node: &Arc<Node>, format: ListFormat) -> bool {
        if self
            .emit_context
            .emit_flags(node)
            .intersects(EmitFlags::START_ON_NEW_LINE)
        {
            return true;
        }
        format.intersects(ListFormat::PREFER_NEW_LINE)
    }

    pub fn should_emit_block_function_body_on_single_line(&self, body: &Arc<Node>) -> bool {
        if self.should_emit_on_single_line(body) {
            return true;
        }
        let multi_line = match &body.data {
            NodeData::Block(d) => d.multi_line,
            _ => false,
        };
        if multi_line {
            return false;
        }
        if let Some(file) = &self.current_source_file {
            if !range_is_on_single_line(body.loc, file) {
                return false;
            }
        }
        true
    }

    pub fn emit_parameters_for_index_signature(
        &mut self,
        parent_node: &Arc<Node>,
        parameters: &Arc<NodeList>,
    ) {
        self.emit_parameters(parent_node, parameters);
    }

    pub fn emit_accessor_declaration(&mut self, token: SyntaxKind, node: &Arc<Node>) {
        let state = self.enter_node(node);
        let pos = greatest_modifier_end(node.pos(), node.modifiers());
        self.emit_token(token, pos, WriteKind::Keyword, node);
        self.write_space();
        if let Some(name) = node.name() {
            self.emit_accessor_name(name);
        }
        let indented = self.should_emit_indented(node);
        self.increase_indent_if(indented);
        self.push_name_generation_scope(node);
        self.emit_signature(node);
        self.emit_function_body_node(node.body());
        self.pop_name_generation_scope(node);
        self.decrease_indent_if(indented);
        self.exit_node(node, state);
    }

    fn emit_accessor_name(&mut self, name: &Arc<Node>) {
        match name.kind {
            SyntaxKind::PrivateIdentifier => self.emit_private_identifier(name),
            SyntaxKind::ComputedPropertyName => {
                self.write_punctuation("[");
                let expression = match &name.data {
                    NodeData::ComputedPropertyName(d) => d.expression.clone(),
                    _ => panic!("ComputedPropertyName expected"),
                };
                self.emit_expression(&expression, OPERATOR_PRECEDENCE_DISALLOW_COMMA);
                self.write_punctuation("]");
            }
            _ => self.emit_identifier_name(name),
        }
    }
}

fn is_prologue_directive(statement: &Arc<Node>) -> bool {
    if statement.kind != SyntaxKind::ExpressionStatement {
        return false;
    }
    match &statement.data {
        NodeData::ExpressionStatement(d) => d.expression.kind == SyntaxKind::StringLiteral,
        _ => false,
    }
}

pub trait R37k15PlaceholderExt {
    fn placeholder() -> Self;
}

impl R37k15PlaceholderExt for Transformer {
    fn placeholder() -> Self {
        Transformer::new(r37k15_noop_visit, None)
    }
}

fn r37k15_noop_visit(
    _tx: &mut Transformer,
    node: Arc<Node>,
) -> Option<Arc<Node>> {
    Some(node)
}

impl R37k15PlaceholderExt for NodeVisitor {
    fn placeholder() -> Self {
        NodeVisitor::default()
    }
}

pub trait R37k15NodeVisitorExt {
    fn visit_each_child(&mut self, node: &Arc<Node>) -> Arc<Node>;
}

impl R37k15NodeVisitorExt for NodeVisitor {
    fn visit_each_child(&mut self, node: &Arc<Node>) -> Arc<Node> {
        Arc::clone(node)
    }
}

pub trait R37k15PropertyAccessExt {
    fn as_property_access_expression(&self) -> &tsox_frontend::ast::node_data_generated::PropertyAccessExpressionData;
}

impl R37k15PropertyAccessExt for Node {
    fn as_property_access_expression(
        &self,
    ) -> &tsox_frontend::ast::node_data_generated::PropertyAccessExpressionData {
        match &self.data {
            NodeData::PropertyAccessExpression(d) => d,
            _ => panic!("as_property_access_expression() on {:?}", self.kind),
        }
    }
}

impl crate::mig::m4h_5::EsDecoratorTransformer {
    pub fn discarded_value_visit(&mut self, node: &Arc<Node>) -> Arc<Node> {
        if node.kind == SyntaxKind::Decorator {
            return Arc::new(Node::new(
                SyntaxKind::Unknown,
                NodeData::MissingDeclaration(tsox_frontend::ast::node_data_generated::MissingDeclarationData {
                    modifiers: None,
                }),
            ));
        }
        self.outer_this_visitor.visit_each_child(node)
    }

    pub fn visit_binary_expression(&mut self, node: &Arc<Node>, discarded: bool) -> Arc<Node> {
        self.outer_this_visitor.visit_each_child(node)
    }

    pub fn visit_pre_or_postfix_unary_expression(
        &mut self,
        node: &Arc<Node>,
        discarded: bool,
    ) -> Arc<Node> {
        self.outer_this_visitor.visit_each_child(node)
    }
}

impl crate::mig::m4i_6::OptionalChainTransformer {
    pub fn visit_node(&mut self, node: &Arc<Node>) -> Arc<Node> {
        Arc::clone(node)
    }

    pub fn visit_nodes(&mut self, list: Option<&NodeList>) -> NodeList {
        match list {
            Some(list) => {
                let mut new_list = NodeList::new(list.nodes.clone());
                new_list.loc = list.loc;
                new_list
            }
            None => NodeList::new(Vec::new()),
        }
    }
}

fn greatest_modifier_end(pos: usize, modifiers: Option<&Arc<tsox_frontend::ast::ModifierList>>) -> usize {
    match modifiers {
        Some(list) => list
            .nodes
            .iter()
            .map(|m| m.end())
            .fold(pos, usize::max),
        None => pos,
    }
}
