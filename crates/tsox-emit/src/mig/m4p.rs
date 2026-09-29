#![allow(dead_code, unused_imports, unused_variables)]

use std::collections::{HashMap, VecDeque};
use std::sync::Arc;

use tsox_frontend::ast::node::{Node, NodeList, SourceFile};
use tsox_frontend::ast::syntax_kind_generated::SyntaxKind;

use tsox_core::core::text::TextRange;
use tsox_frontend::ast::utilities::is_optional_chain;
use tsox_frontend::ast::mig::m3e_3::{
    OperatorPrecedence, TypePrecedence, OPERATOR_PRECEDENCE_DISALLOW_COMMA,
};
use tsox_frontend::format::mig::m4o::{EmitFlags, ListFormat, TokenEmitFlags};
use tsox_frontend::format::mig::m4o_2::{PrinterOptions, WriteKind};
use tsox_frontend::format::mig::m4t_2::{
    GetLiteralTextFlags, GET_LITERAL_TEXT_FLAGS_NEVER_ASCII_ESCAPE,
    GET_LITERAL_TEXT_FLAGS_TERMINATE_UNTERMINATED_LITERALS,
};
use tsox_frontend::format::mig::m4t_4::greatest_end;

use crate::mig::m4r::EmitTextWriter;
use crate::printer::EmitContext;

#[path = "r39k22_defs.rs"]
pub mod r39k22_defs;

pub type EmitFn = fn(&mut Printer, &Arc<Node>);

pub struct FileReference {
    pub loc: TextRange,
    pub file_name: String,
    pub resolution_mode: tsox_core::core::compiler_options_kinds::ResolutionMode,
    pub preserve: bool,
}

pub struct DetachedCommentsInfo {
    pub node_pos: usize,
    pub detached_comment_end_pos: usize,
}

pub struct CommentState {
    pub emit_flags: EmitFlags,
    pub comment_range: TextRange,
    pub container_pos: usize,
    pub container_end: usize,
    pub declaration_list_container_end: usize,
}

impl CommentState {
    pub fn new(
        emit_flags: EmitFlags,
        comment_range: TextRange,
        container_pos: usize,
        container_end: usize,
        declaration_list_container_end: usize,
    ) -> Self {
        Self {
            emit_flags,
            comment_range,
            container_pos,
            container_end,
            declaration_list_container_end,
        }
    }
}

pub struct Printer {
    pub writer: Box<dyn EmitTextWriter>,
    pub emit_context: Arc<EmitContext>,
    pub current_source_file: Option<Arc<SourceFile>>,
    pub options: PrinterOptions,
    pub id_to_symbol: Option<HashMap<*const Node, Arc<tsox_frontend::ast::symbol::Symbol>>>,
    pub external_helpers_module_name: Option<Arc<Node>>,
    pub unique_helper_names: Option<HashMap<String, Arc<Node>>>,
    pub on_before_emit_node: Option<Box<dyn Fn(&Arc<Node>)>>,
    pub on_after_emit_node: Option<Box<dyn Fn(&Arc<Node>)>>,
    pub on_before_emit_node_list: Option<Box<dyn Fn(&Arc<NodeList>)>>,
    pub on_after_emit_node_list: Option<Box<dyn Fn(&Arc<NodeList>)>>,
    pub container_pos: usize,
    pub container_end: usize,
    pub declaration_list_container_end: usize,
    pub detached_comments_info: VecDeque<DetachedCommentsInfo>,
    pub comments_disabled: bool,
    pub next_list_element_pos: usize,
}

impl Printer {
    pub fn emit_keyword_node(&mut self, node: &Arc<Node>) {
        self.emit_keyword_node_ex(node, TokenEmitFlags::NONE);
    }

    pub fn emit_keyword_node_ex(&mut self, node: &Arc<Node>, flags: TokenEmitFlags) {
        let state = self.enter_token_node(node, flags);
        self.write_token_text(node.kind, WriteKind::Keyword, node.pos());
        self.exit_token_node(node, state);
    }

    pub fn emit_literal(&mut self, node: &Arc<Node>, mut flags: GetLiteralTextFlags) {
        if self.options.never_ascii_escape {
            flags |= GET_LITERAL_TEXT_FLAGS_NEVER_ASCII_ESCAPE;
        }
        if self.options.terminate_unterminated_literals {
            flags |= GET_LITERAL_TEXT_FLAGS_TERMINATE_UNTERMINATED_LITERALS;
        }
        let text = self.get_literal_text_of_node(node, None, flags);
        self.writer.write_string_literal(&text);
    }

    pub fn emit_identifier_text(&mut self, node: &Arc<Node>) {
        let text = self.get_text_of_node(node, false);
        let symbol = self
            .id_to_symbol
            .as_ref()
            .and_then(|id_to_symbol| id_to_symbol.get(&(node.as_ref() as *const Node)).cloned());
        if let Some(symbol) = &symbol {
            self.write_symbol(&text, Some(symbol));
            return;
        }
        self.write(&text);
    }

    pub fn emit_identifier_name(&mut self, node: &Arc<Node>) {
        let state = self.enter_node(node);
        self.emit_identifier_text(node);
        self.exit_node(node, state);
    }

    pub fn emit_identifier_name_node(&mut self, node: Option<&Arc<Node>>) {
        let Some(node) = node else {
            return;
        };
        self.emit_identifier_name(node);
    }

    pub fn get_unique_helper_name(&mut self, name: &str) -> Arc<Node> {
        let existing = self.unique_helper_names.as_ref().and_then(|m| m.get(name).cloned());
        if let Some(helper_name) = existing {
            return helper_name.clone();
        }
        let generated = self.emit_context.factory().new_unique_name_ex(
            name,
            crate::printer::generated_identifier_flags::AutoGenerateOptions {
                flags: crate::printer::generated_identifier_flags::GeneratedIdentifierFlags::FILE_LEVEL
                    | crate::printer::generated_identifier_flags::GeneratedIdentifierFlags::OPTIMISTIC,
                prefix: String::new(),
                suffix: String::new(),
            },
        );
        let helper_name = Arc::new(Node::new(
            SyntaxKind::Identifier,
            tsox_frontend::ast::node_data_generated::NodeData::Identifier(
                tsox_frontend::ast::node_data_generated::IdentifierData {
                    text: generated.text().to_string(),
                },
            ),
        ));
        self.generate_name(&helper_name);
        self.unique_helper_names
            .as_mut()
            .unwrap()
            .insert(name.to_string(), helper_name.clone());
        helper_name
    }

    pub fn emit_identifier_reference(&mut self, node: &Arc<Node>) {
        let mut node = node.clone();
        if (self.external_helpers_module_name.is_some() || self.unique_helper_names.is_some())
            && self
                .emit_context
                .emit_flags(&node)
                .intersects(EmitFlags::HELPER_NAME)
        {
            if self.external_helpers_module_name.is_some() {
                let module = self.external_helpers_module_name.as_ref().unwrap().clone();
                let helper = Arc::new(Node::new(
                    SyntaxKind::PropertyAccessExpression,
                    tsox_frontend::ast::node_data_generated::NodeData::PropertyAccessExpression(
                        tsox_frontend::ast::node_data_generated::PropertyAccessExpressionData {
                            expression: module.clone(),
                            question_dot_token: None,
                            name: node.clone(),
                        },
                    ),
                ));
                if let Some(emit_context) = Arc::get_mut(&mut self.emit_context) {
                    emit_context.assign_comment_and_source_map_ranges(&helper, &node);
                }
                self.emit_property_access_expression(&helper);
                return;
            }
            if self.unique_helper_names.is_some() {
                let helper_name = self.get_unique_helper_name(node.text());
                if let Some(emit_context) = Arc::get_mut(&mut self.emit_context) {
                    emit_context.assign_comment_and_source_map_ranges(&helper_name, &node);
                }
                node = helper_name;
            }
        }
        let state = self.enter_node(&node);
        self.emit_identifier_text(&node);
        self.exit_node(&node, state);
    }

    pub fn emit_label_identifier(&mut self, node: &Arc<Node>) {
        let state = self.enter_node(node);
        self.emit_identifier_text(node);
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

    pub fn emit_initializer(
        &mut self,
        node: Option<&Arc<Node>>,
        equal_token_pos: usize,
        context_node: &Arc<Node>,
    ) {
        let Some(node) = node else {
            return;
        };
        self.write_space();
        self.emit_token(
            SyntaxKind::EqualsToken,
            equal_token_pos,
            WriteKind::Operator,
            context_node,
        );
        self.write_space();
        self.emit_expression(node, OPERATOR_PRECEDENCE_DISALLOW_COMMA);
    }

    pub fn emit_keyword_type_node(&mut self, node: &Arc<Node>) {
        self.emit_keyword_node(node);
    }

    pub fn emit_keyword_expression(&mut self, node: &Arc<Node>) {
        self.emit_keyword_node(node);
    }

    pub fn emit_jsdoc_all_type(&mut self, node: &Arc<Node>) {
        self.emit_keyword_node(node);
    }

    pub fn emit_jsdoc_non_nullable_type(&mut self, node: &Arc<Node>) {
        let state = self.enter_node(node);
        self.write_punctuation("!");
        self.emit_type_node(node.type_node().unwrap(), TypePrecedence::NonArray);
        self.exit_node(node, state);
    }

    pub fn emit_jsdoc_nullable_type(&mut self, node: &Arc<Node>) {
        let state = self.enter_node(node);
        self.write_punctuation("?");
        self.emit_type_node(node.type_node().unwrap(), TypePrecedence::NonArray);
        self.exit_node(node, state);
    }

    pub fn emit_jsdoc_optional_type(&mut self, node: &Arc<Node>) {
        let state = self.enter_node(node);
        self.emit_type_node(node.type_node().unwrap(), TypePrecedence::Jsdoc);
        self.write_punctuation("=");
        self.exit_node(node, state);
    }

    pub fn emit_jsdoc_variadic_type(&mut self, node: &Arc<Node>) {
        let state = self.enter_node(node);
        self.write_punctuation("...");
        self.emit_type_node(node.type_node().unwrap(), TypePrecedence::Jsdoc);
        self.exit_node(node, state);
    }
}
