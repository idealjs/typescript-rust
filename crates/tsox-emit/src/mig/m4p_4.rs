#![allow(dead_code, unused_imports, unused_variables)]

use std::sync::Arc;

use tsox_frontend::ast::node::Node;
use tsox_frontend::ast::node_data_generated::NodeData;
use tsox_frontend::ast::syntax_kind_generated::SyntaxKind;
use tsox_frontend::ast::mig::m3e_3::get_leftmost_expression;
use tsox_frontend::ast::node_data_generated::{
    is_block, is_class_expression, is_function_expression,
};
use tsox_frontend::format::mig::m4t_4::greatest_end;

use tsox_frontend::ast::mig::m3e_3::OperatorPrecedence;
use tsox_frontend::format::mig::m4o::{
    CommentSeparator, EmitFlags as PrinterEmitFlags, ListFormat, TokenEmitFlags,
};
use tsox_frontend::format::mig::m4o_2::WriteKind;

use crate::mig::m4m_2::r36k22_defs::{
    ENUM_MEMBERS, HERITAGE_CLAUSES, INTERFACE_MEMBERS, MULTI_LINE_FUNCTION_BODY_STATEMENTS,
    SINGLE_LINE_FUNCTION_BODY_STATEMENTS,
};

use super::m4p::Printer;

#[path = "r37k15_defs.rs"]
pub mod r37k15_defs;

impl Printer {
    pub fn emit_function_body(&mut self, body: &Arc<Node>) {
        Arc::make_mut(&mut self.emit_context)
            .add_emit_flags(body, PrinterEmitFlags::NO_SOURCE_MAP);
        if let Some(on_before) = &self.on_before_emit_node {
            on_before(body);
        }
        self.generate_names(body);
        self.write_punctuation("{");
        self.increase_indent();
        let statements = match &body.data {
            NodeData::Block(d) => d.statements.clone(),
            _ => panic!("Block expected"),
        };
        let statements = &statements;
        let detached_state =
            self.emit_detached_comments_before_statement_list(body, statements.loc);
        let statement_offset = self.emit_prologue_directives(statements);
        let pos = self.writer.get_text_pos();
        self.emit_helpers(body);
        if self.should_emit_block_function_body_on_single_line(body)
            && statement_offset == 0
            && pos == self.writer.get_text_pos()
        {
            self.decrease_indent();
            self.emit_list_range(
                Self::emit_statement,
                body,
                Some(statements),
                SINGLE_LINE_FUNCTION_BODY_STATEMENTS,
                statement_offset,
                -1,
            );
            self.increase_indent();
        } else {
            self.emit_list_range(
                Self::emit_statement,
                body,
                Some(statements),
                MULTI_LINE_FUNCTION_BODY_STATEMENTS,
                statement_offset,
                -1,
            );
        }
        self.emit_detached_comments_after_statement_list(body, statements.loc, detached_state);
        self.decrease_indent();
        self.emit_token_ex(
            SyntaxKind::CloseBraceToken,
            statements.end(),
            WriteKind::Punctuation,
            body,
            TokenEmitFlags::NO_COMMENTS,
        );
        if let Some(on_after) = &self.on_after_emit_node {
            on_after(body);
        }
    }

    pub fn emit_function_body_node(&mut self, node: Option<&Arc<Node>>) {
        let node = match node {
            Some(node) => node,
            None => {
                self.write_trailing_semicolon();
                return;
            }
        };
        self.write_space();
        self.emit_function_body(node);
    }

    pub fn emit_get_accessor_declaration(&mut self, node: &Arc<Node>) {
        self.emit_accessor_declaration(SyntaxKind::GetKeyword, node);
    }

    pub fn emit_index_signature(&mut self, node: &Arc<Node>) {
        let state = self.enter_node(node);
        self.emit_modifier_list(node, node.modifiers(), false);
        let indented = self.should_emit_indented(node);
        self.increase_indent_if(indented);
        self.push_name_generation_scope(node);
        let parameters = match &node.data {
            NodeData::IndexSignatureDeclaration(d) => d.parameters.clone(),
            _ => panic!("IndexSignatureDeclaration expected"),
        };
        self.emit_parameters_for_index_signature(node, &parameters);
        self.emit_type_annotation(node.type_node());
        self.write_trailing_semicolon();
        self.pop_name_generation_scope(node);
        self.decrease_indent_if(indented);
        self.exit_node(node, state);
    }

    pub fn emit_function_declaration(&mut self, node: &Arc<Node>) {
        let state = self.enter_node(node);
        self.generate_name_if_needed(node.name());
        self.emit_modifier_list(node, node.modifiers(), false);
        self.write_keyword("function");
        let asterisk_token = match &node.data {
            NodeData::FunctionDeclaration(d) => d.asterisk_token.clone(),
            _ => panic!("FunctionDeclaration expected"),
        };
        self.emit_token_node(asterisk_token.as_ref());
        self.write_space();
        if let Some(name) = node.name() {
            self.emit_identifier_name(name);
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

    pub fn emit_interface_declaration(&mut self, node: &Arc<Node>) {
        let state = self.enter_node(node);
        self.emit_modifier_list(node, node.modifiers(), false);
        self.write_keyword("interface");
        self.write_space();
        self.emit_binding_identifier(node.name().unwrap());
        let (type_parameters, heritage_clauses, members) = match &node.data {
            NodeData::InterfaceDeclaration(d) => (
                d.type_parameters.clone(),
                d.heritage_clauses.clone(),
                d.members.clone(),
            ),
            _ => panic!("InterfaceDeclaration expected"),
        };
        self.emit_type_parameters(node, type_parameters.as_ref());
        self.emit_list(
            Self::emit_heritage_clause_node,
            node,
            heritage_clauses.as_ref().unwrap(),
            HERITAGE_CLAUSES,
        );
        self.write_space();
        self.write_punctuation("{");
        self.push_name_generation_scope(node);
        self.generate_all_member_names(&members);
        self.emit_list(Self::emit_type_element, node, &members, INTERFACE_MEMBERS);
        self.pop_name_generation_scope(node);
        self.write_punctuation("}");
        self.exit_node(node, state);
    }

    pub fn emit_enum_declaration(&mut self, node: &Arc<Node>) {
        let state = self.enter_node(node);
        self.emit_modifier_list(node, node.modifiers(), false);
        self.write_keyword("enum");
        self.write_space();
        self.emit_binding_identifier(node.name().unwrap());
        self.write_space();
        self.write_punctuation("{");
        let members = match &node.data {
            NodeData::EnumDeclaration(d) => d.members.clone(),
            _ => panic!("EnumDeclaration expected"),
        };
        self.emit_list(
            Self::emit_enum_member_node,
            node,
            &members,
            ENUM_MEMBERS,
        );
        self.write_punctuation("}");
        self.exit_node(node, state);
    }
}
