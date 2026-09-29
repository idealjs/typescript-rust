#![allow(dead_code, unused_imports, unused_variables)]

use std::sync::Arc;

use tsox_frontend::ast::node::Node;
use tsox_frontend::ast::syntax_kind_generated::SyntaxKind;
use tsox_frontend::ast::mig::m3e_3::{
    get_leftmost_expression, OperatorPrecedence, OPERATOR_PRECEDENCE_DISALLOW_COMMA,
    OPERATOR_PRECEDENCE_LOWEST,
};
use tsox_frontend::ast::node_data_generated::{
    is_block, is_class_expression, is_function_expression,
};

use tsox_frontend::format::mig::m4o::{
    CommentSeparator, EmitFlags as PrinterEmitFlags, ListFormat,
};
use tsox_frontend::format::mig::m4o_2::WriteKind;

use super::m4p::Printer;
use crate::mig::m4r::EmitTextWriter;
use self::r36k12_defs::{
    attributes, export_clause, greatest_modifier_end, greatest_opt_node_end, import_attribute_value,
    import_attributes_elements, import_attributes_token, import_clause, is_export_equals,
    is_type_only, module_reference, module_specifier, named_bindings, phase_modifier,
    LF_IMPORT_ATTRIBUTES,
};

#[path = "r36k12_defs.rs"]
pub mod r36k12_defs;
#[path = "r36k12_printer_impl.rs"]
pub mod r36k12_printer_impl;

impl Printer {
    pub fn emit_import_equals_declaration(&mut self, node: &Arc<Node>) {
        let state = self.enter_node(node);
        self.emit_modifier_list(node, node.modifiers(), false);
        let name = node.name().unwrap();
        let pos = self.emit_token(
            SyntaxKind::ImportKeyword,
            greatest_modifier_end(node.pos(), node.modifiers()),
            WriteKind::Keyword,
            node,
        );
        self.writer.write_space(" ");
        if is_type_only(node) {
            self.emit_token(SyntaxKind::TypeKeyword, pos, WriteKind::Keyword, node);
            self.writer.write_space(" ");
        }
        self.emit_binding_identifier(name);
        self.writer.write_space(" ");
        self.emit_token(SyntaxKind::EqualsToken, name.end(), WriteKind::Punctuation, node);
        self.writer.write_space(" ");
        self.emit_module_reference(module_reference(node));
        self.writer.write_trailing_semicolon(";");
        self.exit_node(node, state);
    }

    pub fn emit_import_declaration(&mut self, node: &Arc<Node>) {
        let state = self.enter_node(node);
        self.emit_modifier_list(node, node.modifiers(), false);
        self.emit_token(
            SyntaxKind::ImportKeyword,
            greatest_modifier_end(node.pos(), node.modifiers()),
            WriteKind::Keyword,
            node,
        );
        self.writer.write_space(" ");
        if let Some(import_clause) = import_clause(node) {
            self.emit_import_clause(import_clause);
            self.writer.write_space(" ");
            self.emit_token(SyntaxKind::FromKeyword, import_clause.end(), WriteKind::Keyword, node);
            self.writer.write_space(" ");
        }
        self.emit_expression(module_specifier(node).unwrap(), OPERATOR_PRECEDENCE_LOWEST);
        if let Some(attributes) = attributes(node) {
            self.writer.write_space(" ");
            self.emit_import_attributes(attributes);
        }
        self.writer.write_trailing_semicolon(";");
        self.exit_node(node, state);
    }

    pub fn emit_import_clause(&mut self, node: &Arc<Node>) {
        let state = self.enter_node(node);
        if let Some(phase) = phase_modifier(node) {
            self.emit_token(phase, node.pos(), WriteKind::Keyword, node);
            self.writer.write_space(" ");
        }
        if let Some(name) = node.name() {
            self.emit_binding_identifier(name);
            if named_bindings(node).is_some() {
                self.emit_token(SyntaxKind::CommaToken, name.end(), WriteKind::Punctuation, node);
                self.writer.write_space(" ");
            }
        }
        self.emit_named_import_bindings(named_bindings(node));
        self.exit_node(node, state);
    }

    pub fn emit_import_specifier(&mut self, node: &Arc<Node>) {
        let state = self.enter_node(node);
        if is_type_only(node) {
            self.writer.write_keyword("type");
            self.writer.write_space(" ");
        }
        if let Some(property_name) = node.property_name() {
            self.emit_module_export_name(property_name);
            self.writer.write_space(" ");
            self.emit_token(SyntaxKind::AsKeyword, property_name.end(), WriteKind::Keyword, node);
            self.writer.write_space(" ");
        }
        self.emit_binding_identifier(node.name().unwrap());
        self.exit_node(node, state);
    }

    pub fn emit_import_specifier_node(&mut self, node: &Arc<Node>) {
        self.emit_import_specifier(node);
    }

    pub fn emit_export_assignment(&mut self, node: &Arc<Node>) {
        let state = self.enter_node(node);
        let expression = node.expression().unwrap();
        let next_pos = self.emit_token(SyntaxKind::ExportKeyword, node.pos(), WriteKind::Keyword, node);
        self.writer.write_space(" ");
        if is_export_equals(node) {
            self.emit_token(SyntaxKind::EqualsToken, next_pos, WriteKind::Operator, node);
        } else {
            self.emit_token(SyntaxKind::DefaultKeyword, next_pos, WriteKind::Keyword, node);
        }
        self.writer.write_space(" ");
        if is_export_equals(node) {
            self.emit_expression(expression, OperatorPrecedence::Assignment);
        } else {
            let expr = get_leftmost_expression(expression, false);
            if is_class_expression(&expr) || is_function_expression(&expr) {
                self.emit_expression(expression, OperatorPrecedence::Parentheses);
            } else {
                self.emit_expression(expression, OperatorPrecedence::Assignment);
            }
        }
        self.writer.write_trailing_semicolon(";");
        self.exit_node(node, state);
    }

    pub fn emit_export_declaration(&mut self, node: &Arc<Node>) {
        let state = self.enter_node(node);
        self.emit_modifier_list(node, node.modifiers(), false);
        let mut pos = self.emit_token(SyntaxKind::ExportKeyword, node.pos(), WriteKind::Keyword, node);
        self.writer.write_space(" ");
        if is_type_only(node) {
            pos = self.emit_token(SyntaxKind::TypeKeyword, pos, WriteKind::Keyword, node);
            self.writer.write_space(" ");
        }
        let export_clause_node = export_clause(node);
        if let Some(export_clause) = export_clause_node {
            self.emit_named_export_bindings(export_clause);
        } else {
            pos = self.emit_token(SyntaxKind::AsteriskToken, pos, WriteKind::Punctuation, node);
        }
        if let Some(module_specifier) = module_specifier(node) {
            self.writer.write_space(" ");
            self.emit_token(
                SyntaxKind::FromKeyword,
                greatest_opt_node_end(pos, export_clause_node),
                WriteKind::Keyword,
                node,
            );
            self.writer.write_space(" ");
            self.emit_expression(module_specifier, OPERATOR_PRECEDENCE_LOWEST);
        }
        if let Some(attributes) = attributes(node) {
            self.writer.write_space(" ");
            self.emit_import_attributes(attributes);
        }
        self.writer.write_trailing_semicolon(";");
        self.exit_node(node, state);
    }

    pub fn emit_import_attributes(&mut self, node: &Arc<Node>) {
        let state = self.enter_node(node);
        self.emit_token(import_attributes_token(node), node.pos(), WriteKind::Keyword, node);
        self.writer.write_space(" ");
        self.emit_list(
            Self::emit_import_attribute_node,
            node,
            import_attributes_elements(node),
            LF_IMPORT_ATTRIBUTES,
        );
        self.exit_node(node, state);
    }

    pub fn emit_import_attribute(&mut self, node: &Arc<Node>) {
        let state = self.enter_node(node);
        self.emit_import_attribute_name(node.name().unwrap());
        self.writer.write_punctuation(":");
        self.writer.write_space(" ");
        let value = import_attribute_value(node);
        if !self.emit_context.emit_flags(value).intersects(PrinterEmitFlags::NO_LEADING_COMMENTS) {
            let comment_range = self.emit_context.comment_range(value);
            self.emit_trailing_comments(comment_range.pos(), CommentSeparator::After);
        }
        self.emit_expression(value, OPERATOR_PRECEDENCE_DISALLOW_COMMA);
        self.exit_node(node, state);
    }

    pub fn emit_import_attribute_node(&mut self, node: &Arc<Node>) {
        self.emit_import_attribute(node);
    }

    pub fn emit_export_specifier(&mut self, node: &Arc<Node>) {
        let state = self.enter_node(node);
        if is_type_only(node) {
            self.writer.write_keyword("type");
            self.writer.write_space(" ");
        }
        if let Some(property_name) = node.property_name() {
            self.emit_module_export_name(property_name);
            self.writer.write_space(" ");
            self.emit_token(SyntaxKind::AsKeyword, property_name.end(), WriteKind::Keyword, node);
            self.writer.write_space(" ");
        }
        self.emit_module_export_name(node.name().unwrap());
        self.exit_node(node, state);
    }

    pub fn emit_export_specifier_node(&mut self, node: &Arc<Node>) {
        self.emit_export_specifier(node);
    }

    pub fn emit_embedded_statement(&mut self, parent_node: &Arc<Node>, node: &Arc<Node>) {
        if is_block(node)
            || self.should_emit_on_single_line(parent_node)
            || self.options.preserve_source_newlines
                && self.get_leading_line_terminator_count(parent_node, Some(node), ListFormat::NONE)
                    == 0
        {
            self.writer.write_space(" ");
            self.emit_statement(node);
        } else {
            self.write_line();
            self.writer.increase_indent();
            if node.kind == SyntaxKind::EmptyStatement {
                self.emit_empty_statement(node, true);
            } else {
                self.emit_statement(node);
            }
            self.writer.decrease_indent();
        }
    }
}
