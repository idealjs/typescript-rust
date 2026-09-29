#![allow(dead_code, unused_imports, unused_variables)]

use std::collections::HashMap;
use std::sync::{Arc, Mutex, OnceLock};

use tsox_frontend::ast::mig::m3b;
use tsox_frontend::ast::node::{Node, NodeList};
use tsox_frontend::ast::node_data_generated::NodeData;
use tsox_frontend::ast::syntax_kind_generated::SyntaxKind;
use tsox_frontend::format::mig::m4o;
use tsox_frontend::format::mig::m4o_2::EmitContext;
use tsox_frontend::format::mig::m4t_3::range_end_is_on_same_line_as_range_start;
use tsox_frontend::scanner::token_to_string;

use super::Printer;
use super::r33k12_defs::{EmitFlags, PrinterState, TokenEmitFlags, WriteKind};
use crate::mig::m4r::EmitTextWriter;
use crate::mig::m4q_4::r36k21_defs::TypedNode21;

static NODE_EMIT_FLAGS: OnceLock<Mutex<HashMap<u64, u32>>> = OnceLock::new();

fn node_emit_flags_table() -> &'static Mutex<HashMap<u64, u32>> {
    NODE_EMIT_FLAGS.get_or_init(|| Mutex::new(HashMap::new()))
}

pub trait EmitContextExtK08 {
    fn emit_flags_of(&self, node: &Node) -> u32;
    fn factory(&self) -> m4o::NodeFactory;
    fn add_emit_flags(&self, node: &Arc<Node>, flags: EmitFlags);
}

impl EmitContextExtK08 for EmitContext {
    fn emit_flags_of(&self, node: &Node) -> u32 {
        node_emit_flags_table()
            .lock()
            .unwrap()
            .get(&node.id())
            .copied()
            .unwrap_or(0)
    }

    fn factory(&self) -> m4o::NodeFactory {
        m4o::new_node_factory(m4o::EmitContext::default())
    }

    fn add_emit_flags(&self, node: &Arc<Node>, flags: EmitFlags) {
        let mut table = node_emit_flags_table().lock().unwrap();
        let entry = table.entry(node.id()).or_insert(0);
        *entry |= flags.0;
    }
}

pub trait NodeSpanExtK08 {
    fn type_(&self) -> &Node;
    fn literal(&self) -> &Node;
}

impl NodeSpanExtK08 for Node {
    fn type_(&self) -> &Node {
        match &self.data {
            NodeData::TemplateLiteralTypeSpan(d) => &d.type_node,
            _ => panic!("unexpected span type: {:?}", self.kind),
        }
    }

    fn literal(&self) -> &Node {
        match &self.data {
            NodeData::TemplateLiteralTypeSpan(d) => &d.literal,
            _ => panic!("unexpected span literal: {:?}", self.kind),
        }
    }
}

pub trait TypedNode21ExtK08 {
    fn question_dot_token(&self) -> Option<Arc<Node>>;
}

impl TypedNode21ExtK08 for TypedNode21<'_> {
    fn question_dot_token(&self) -> Option<Arc<Node>> {
        m3b::question_dot_token(self.as_node()).cloned()
    }
}

pub trait PrinterExtK08 {
    fn write(&mut self, text: &str);
    fn write_as(&mut self, text: &str, write_kind: WriteKind);
    fn write_space(&mut self);
    fn write_keyword(&mut self, text: &str);
    fn write_punctuation(&mut self, text: &str);
    fn write_trailing_semicolon(&mut self);
    fn write_line(&mut self);
    fn write_line_repeat(&mut self, count: i32);
    fn write_token_text(&mut self, token: SyntaxKind, write_kind: WriteKind, pos: usize) -> usize;
    fn increase_indent(&mut self);
    fn decrease_indent(&mut self);
    fn increase_indent_if(&mut self, indent_requested: bool);
    fn decrease_indent_if(&mut self, indent_requested: bool);
    fn should_emit_indented(&self, node: &Node) -> bool;
    fn should_emit_on_single_line(&self, node: &Node) -> bool;
    fn should_emit_on_multiple_lines(&self, node: &Node) -> bool;
    fn should_elide_indentation(&self, node: &Node) -> bool;
    fn is_empty_block(&self, block: &Node, statements: &NodeList) -> bool;
    fn enter_token(
        &mut self,
        token: SyntaxKind,
        pos: usize,
        context_node: &Node,
        flags: TokenEmitFlags,
    ) -> (PrinterState, usize);
    fn exit_token(
        &mut self,
        token: SyntaxKind,
        pos: usize,
        context_node: &Node,
        previous_state: PrinterState,
    );
    fn emit_keyword_node(&mut self, node: &Node);
    fn emit_keyword_node_ex(&mut self, node: &Node, flags: TokenEmitFlags);
    fn emit_identifier_text(&mut self, node: &Node);
    fn emit_identifier_name(&mut self, node: &Node);
    fn emit_binding_identifier(&mut self, node: &Node);
    fn emit_identifier_reference(&mut self, node: &Node);
    fn emit_export_specifier_node(&mut self, node: &Node);
}

impl PrinterExtK08 for Printer {
    fn write(&mut self, text: &str) {
        let kind = self.write_kind;
        self.write_as(text, kind);
    }

    fn write_as(&mut self, text: &str, write_kind: WriteKind) {
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

    fn write_space(&mut self) {
        self.writer.write_space(" ");
    }

    fn write_keyword(&mut self, text: &str) {
        self.writer.write_keyword(text);
    }

    fn write_punctuation(&mut self, text: &str) {
        self.writer.write_punctuation(text);
    }

    fn write_trailing_semicolon(&mut self) {
        self.writer.write_trailing_semicolon(";");
    }

    fn write_line(&mut self) {
        self.writer.write_line();
    }

    fn write_line_repeat(&mut self, count: i32) {
        for _ in 0..count {
            self.write_line();
        }
    }

    fn write_token_text(&mut self, token: SyntaxKind, write_kind: WriteKind, pos: usize) -> usize {
        let text = token_to_string(token);
        self.write_as(text, write_kind);
        pos + text.len()
    }

    fn increase_indent(&mut self) {
        self.writer.increase_indent();
    }

    fn decrease_indent(&mut self) {
        self.writer.decrease_indent();
    }

    fn increase_indent_if(&mut self, indent_requested: bool) {
        if indent_requested {
            self.increase_indent();
        }
    }

    fn decrease_indent_if(&mut self, indent_requested: bool) {
        if indent_requested {
            self.decrease_indent();
        }
    }

    fn should_emit_indented(&self, node: &Node) -> bool {
        let flags = self.emit_context.emit_flags_of(node);
        flags & EmitFlags::INDENTED.0 != 0 && flags & EmitFlags::NO_INDENTATION.0 == 0
    }

    fn should_emit_on_single_line(&self, node: &Node) -> bool {
        self.emit_context.emit_flags_of(node) & EmitFlags::SINGLE_LINE.0 != 0
    }

    fn should_emit_on_multiple_lines(&self, node: &Node) -> bool {
        self.emit_context.emit_flags_of(node) & EmitFlags::MULTI_LINE.0 != 0
    }

    fn should_elide_indentation(&self, node: &Node) -> bool {
        self.emit_context.emit_flags_of(node) & EmitFlags::NO_INDENTATION.0 != 0
    }

    fn is_empty_block(&self, block: &Node, statements: &NodeList) -> bool {
        statements.nodes.is_empty()
            && (self.current_source_file.is_none()
                || range_end_is_on_same_line_as_range_start(
                    block.loc,
                    block.loc,
                    self.current_source_file.as_deref().unwrap(),
                ))
    }

    fn enter_token(
        &mut self,
        token: SyntaxKind,
        pos: usize,
        context_node: &Node,
        flags: TokenEmitFlags,
    ) -> (PrinterState, usize) {
        let state = PrinterState {
            comment_state: None,
            source_map_state: None,
        };
        (state, pos)
    }

    fn exit_token(
        &mut self,
        token: SyntaxKind,
        pos: usize,
        context_node: &Node,
        previous_state: PrinterState,
    ) {
    }

    fn emit_keyword_node(&mut self, node: &Node) {
        self.emit_keyword_node_ex(node, TokenEmitFlags::NONE);
    }

    fn emit_keyword_node_ex(&mut self, node: &Node, flags: TokenEmitFlags) {
        let state = self.enter_node(node);
        self.write_token_text(node.kind, WriteKind::Keyword, node.pos());
        self.exit_node(node, state);
    }

    fn emit_identifier_text(&mut self, node: &Node) {
        let text = node.text();
        self.write(text);
    }

    fn emit_identifier_name(&mut self, node: &Node) {
        let state = self.enter_node(node);
        self.emit_identifier_text(node);
        self.exit_node(node, state);
    }

    fn emit_binding_identifier(&mut self, node: &Node) {
        let state = self.enter_node(node);
        self.emit_identifier_text(node);
        self.exit_node(node, state);
    }

    fn emit_identifier_reference(&mut self, node: &Node) {
        let state = self.enter_node(node);
        self.emit_identifier_text(node);
        self.exit_node(node, state);
    }

    fn emit_export_specifier_node(&mut self, node: &Node) {
        let state = self.enter_node(node);
        let specifier = match &node.data {
            NodeData::ExportSpecifier(d) => d,
            _ => panic!("unexpected ExportSpecifier: {:?}", node.kind),
        };
        if specifier.is_type_only {
            self.write_keyword("type");
            self.write_space();
        }
        if let Some(property_name) = &specifier.property_name {
            self.emit_module_export_name(Some(property_name.as_ref()));
            self.write_space();
            self.emit_token(SyntaxKind::AsKeyword, property_name.end(), WriteKind::Keyword, node);
            self.write_space();
        }
        self.emit_module_export_name(Some(specifier.name.as_ref()));
        self.exit_node(node, state);
    }
}
