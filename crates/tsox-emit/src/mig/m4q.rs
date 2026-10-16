#![allow(dead_code, unused_imports, unused_variables)]

#[path = "r33k12_defs.rs"]
pub mod r33k12_defs;

#[path = "r39k08_defs.rs"]
pub mod r39k08_defs;

#[path = "r40k13_defs.rs"]
pub mod r40k13_defs;

use crate::mig::m4q_4;

use std::collections::HashMap;
use std::sync::Arc;

use tsox_core::core::text::TextRange;
use tsox_frontend::ast::node::{ModifierList, Node, NodeList, SourceFile};
use tsox_frontend::scanner::TokenFlags;
use tsox_frontend::ast::syntax_kind_generated::SyntaxKind;
use tsox_frontend::ast::utilities::*;
use self::r33k12_defs::{OperatorPrecedence, TypePrecedence, greatest_end, is_decorator, is_modifier, is_module_declaration};

use self::r33k12_defs::EmitContext;
use self::r33k12_defs::EmitFlags;
use self::r33k12_defs::NameGenerator;
use self::r33k12_defs::{ListFlags, PrinterOptions, PrinterState, TokenEmitFlags, WriteKind};
use self::r33k12_defs::{
    LF_DECORATORS, LF_MODIFIERS, LF_MULTI_LINE, LF_MULTI_LINE_BLOCK_STATEMENTS,
    LF_NAMED_IMPORTS_OR_EXPORTS_ELEMENTS, LF_SINGLE_LINE_BLOCK_STATEMENTS,
};
use super::m4q_3::r37k6_defs::NodeAsExt;
use self::r39k08_defs::{EmitContextExtK08, NodeSpanExtK08, PrinterExtK08, TypedNode21ExtK08};
use crate::mig::m4r::EmitTextWriter;

pub struct Printer {
    pub options: PrinterOptions,
    pub emit_context: EmitContext,
    pub current_source_file: Option<Arc<SourceFile>>,
    pub writer: Box<dyn EmitTextWriter>,
    pub next_list_element_pos: usize,
    pub write_kind: WriteKind,
    pub container_pos: usize,
    pub container_end: usize,
    pub declaration_list_container_end: usize,
    pub comments_disabled: bool,
    pub in_extends: bool,
    pub name_generator: NameGenerator,
    pub on_before_emit_node_list: Option<Box<dyn FnMut(&NodeList)>>,
    pub on_after_emit_node_list: Option<Box<dyn FnMut(&NodeList)>>,
}

impl Printer {
    pub(crate) fn emit_member_name(&mut self, node: Option<&Node>) { ::tsox_core::fntrace::enter("emit_member_name"); 
        let Some(node) = node else { return };
        match node.kind {
            SyntaxKind::Identifier => self.emit_identifier_name(node.as_identifier().as_node()),
            SyntaxKind::PrivateIdentifier => {
                self.emit_private_identifier(node.as_private_identifier().as_node())
            }
            _ => panic!("unexpected MemberName: {:?}", node.kind),
        }
    }

    pub(crate) fn emit_meta_property(&mut self, node: &Node) { ::tsox_core::fntrace::enter("emit_meta_property"); 
        let state = self.enter_node(node);
        let meta = node.as_meta_property();
        self.emit_token(meta.keyword_token(), meta.as_node().pos(), WriteKind::Punctuation, meta.as_node());
        self.write_punctuation(".");
        self.emit_identifier_name(meta.name().as_identifier().as_node());
        self.exit_node(meta.as_node(), state);
    }

    pub(crate) fn emit_method_declaration(&mut self, node: &Node) { ::tsox_core::fntrace::enter("emit_method_declaration"); 
        let state = self.enter_node(node);
        let method = node.as_method_declaration();
        self.emit_modifier_list(method.as_node(), method.modifiers(), true);
        self.emit_token_node(method.asterisk_token());
        self.emit_property_name(Some(method.name()));
        self.emit_token_node(method.postfix_token());
        let indented = self.should_emit_indented(method.as_node());
        self.increase_indent_if(indented);
        self.push_name_generation_scope(method.as_node());
        self.emit_signature(method.as_node());
        self.emit_function_body_node(method.body());
        self.pop_name_generation_scope(method.as_node());
        self.decrease_indent_if(indented);
        self.exit_node(method.as_node(), state);
    }

    pub(crate) fn emit_method_signature(&mut self, node: &Node) { ::tsox_core::fntrace::enter("emit_method_signature"); 
        let state = self.enter_node(node);
        let method = node.as_method_signature_declaration();
        self.emit_modifier_list(method.as_node(), method.modifiers(), false);
        self.emit_property_name(Some(method.name()));
        self.emit_token_node(method.postfix_token());
        let indented = self.should_emit_indented(method.as_node());
        self.increase_indent_if(indented);
        self.push_name_generation_scope(method.as_node());
        self.emit_signature(method.as_node());
        self.write_trailing_semicolon();
        self.pop_name_generation_scope(method.as_node());
        self.decrease_indent_if(indented);
        self.exit_node(method.as_node(), state);
    }

    pub(crate) fn emit_modifier_like(&mut self, node: &Node) { ::tsox_core::fntrace::enter("emit_modifier_like"); 
        if is_decorator(node) {
            self.emit_decorator(node.as_decorator().as_node());
        } else if is_modifier(node) {
            self.emit_keyword_node(node);
        } else {
            panic!("unhandled ModifierLike: {:?}", node.kind);
        }
    }

    pub(crate) fn emit_modifier_list(
        &mut self,
        parent_node: &Node,
        modifiers: Option<&ModifierList>,
        allow_decorators: bool,
    ) -> usize { ::tsox_core::fntrace::enter("emit_modifier_list"); 
        let Some(modifiers) = modifiers else {
            return parent_node.pos();
        };
        if modifiers.nodes.is_empty() {
            return parent_node.pos();
        }

        if modifiers.nodes.iter().all(|n| is_modifier(n)) {
            self.emit_list(Self::emit_keyword_node, parent_node, &modifiers.list, LF_MODIFIERS);
        } else if modifiers.nodes.iter().all(|n| is_decorator(n)) {
            if !allow_decorators {
                return parent_node.pos();
            }
            self.emit_list(Self::emit_modifier_like, parent_node, &modifiers.list, LF_DECORATORS);
        } else {
            if let Some(on_before_emit_node_list) = self.on_before_emit_node_list.as_mut() {
                on_before_emit_node_list(&modifiers.list);
            }

            #[derive(PartialEq, Clone, Copy)]
            enum Mode {
                None,
                Modifiers,
                Decorators,
            }

            let mut last_mode = Mode::None;
            let mut mode = Mode::None;
            let mut start = 0;
            let mut pos = 0;

            let mut last_modifier: &Node = &modifiers.nodes[0];
            while start < modifiers.nodes.len() {
                while pos < modifiers.nodes.len() {
                    last_modifier = &modifiers.nodes[pos];
                    if is_decorator(last_modifier) {
                        mode = Mode::Decorators;
                    } else {
                        mode = Mode::Modifiers;
                    }
                    if last_mode == Mode::None {
                        last_mode = mode;
                    } else if mode != last_mode {
                        break;
                    }
                    pos += 1;
                }

                let mut text_range = TextRange::undefined();
                if start == 0 {
                    text_range = TextRange::new(modifiers.pos(), text_range.end());
                }
                if pos == modifiers.nodes.len() - 1 {
                    text_range = TextRange::new(text_range.pos(), modifiers.end());
                }
                if allow_decorators || last_mode == Mode::Modifiers {
                    self.emit_list_items(
                        Self::emit_modifier_like,
                        parent_node,
                        &modifiers.nodes[start..pos],
                        if last_mode == Mode::Modifiers { LF_MODIFIERS } else { LF_DECORATORS },
                        false,
                        text_range,
                    );
                }
                start = pos;
                last_mode = mode;
                pos += 1;
            }

            if let Some(on_after_emit_node_list) = self.on_after_emit_node_list.as_mut() {
                on_after_emit_node_list(&modifiers.list);
            }
        }

        let mut end = parent_node.pos() as i64;
        if let Some(last) = modifiers.nodes.last() {
            let last_end = last.end() as i64;
            if end < last_end {
                end = last_end;
            }
        }
        end as usize
    }

    pub(crate) fn emit_module_block(&mut self, node: &Node) { ::tsox_core::fntrace::enter("emit_module_block"); 
        let state = self.enter_node(node);
        let block = node.as_module_block();
        self.generate_names(Some(block.as_node()));
        self.emit_token(SyntaxKind::OpenBraceToken, block.as_node().pos(), WriteKind::Punctuation, block.as_node());
        let format = if self.is_empty_block(block.as_node(), block.statements())
            || self.should_emit_on_single_line(block.as_node())
        {
            LF_SINGLE_LINE_BLOCK_STATEMENTS
        } else {
            LF_MULTI_LINE_BLOCK_STATEMENTS
        };
        self.emit_list(Self::emit_statement, block.as_node(), block.statements(), format);
        self.emit_token_ex(
            SyntaxKind::CloseBraceToken,
            block.statements().end(),
            WriteKind::Punctuation,
            block.as_node(),
            if (format & LF_MULTI_LINE) != 0 {
                TokenEmitFlags::INDENT_LEADING_COMMENTS
            } else {
                TokenEmitFlags::NONE
            },
        );
        self.exit_node(block.as_node(), state);
    }

    pub(crate) fn emit_module_declaration(&mut self, node: &Node) { ::tsox_core::fntrace::enter("emit_module_declaration"); 
        let state = self.enter_node(node);
        let module = node.as_module_declaration();
        self.emit_modifier_list(module.as_node(), module.modifiers(), false);
        if module.keyword() != SyntaxKind::GlobalKeyword {
            self.write_keyword(if module.keyword() == SyntaxKind::NamespaceKeyword { "namespace" } else { "module" });
            self.write_space();
        }
        self.emit_module_name(Some(module.name()));
        let mut body = module.body();
        while let Some(b) = body {
            if !is_module_declaration(b) {
                break;
            }
            let inner = b.as_module_declaration();
            self.write_punctuation(".");
            self.emit_nested_module_name(Some(inner.name()));
            body = inner.body();
        }
        if let Some(attributes) = module.attributes() {
            self.write_space();
            self.write_keyword("with");
            self.write_space();
            self.emit_type_node(attributes, TypePrecedence::NonArray);
        }
        match body {
            None => self.write_trailing_semicolon(),
            Some(b) => {
                self.write_space();
                self.emit_module_block(b.as_module_block().as_node());
            }
        }
        self.exit_node(module.as_node(), state);
    }

    pub(crate) fn emit_module_export_name(&mut self, node: Option<&Node>) { ::tsox_core::fntrace::enter("emit_module_export_name"); 
        let Some(node) = node else { return };
        match node.kind {
            SyntaxKind::Identifier => self.emit_identifier_name(node.as_identifier().as_node()),
            SyntaxKind::StringLiteral => {
                self.emit_string_literal(node.as_string_literal().as_node())
            }
            _ => panic!("unexpected ModuleExportName: {:?}", node.kind),
        }
    }

    pub(crate) fn emit_module_name(&mut self, node: Option<&Node>) { ::tsox_core::fntrace::enter("emit_module_name"); 
        let Some(node) = node else { return };
        match node.kind {
            SyntaxKind::Identifier => self.emit_binding_identifier(node.as_identifier().as_node()),
            SyntaxKind::StringLiteral => self.emit_string_literal(node.as_string_literal().as_node()),
            _ => panic!("unexpected ModuleName: {:?}", node.kind),
        }
    }

    pub(crate) fn emit_module_reference(&mut self, node: &Node) { ::tsox_core::fntrace::enter("emit_module_reference"); 
        match node.kind {
            SyntaxKind::Identifier => self.emit_identifier_reference(node.as_identifier().as_node()),
            SyntaxKind::QualifiedName => self.emit_qualified_name(node.as_qualified_name().as_node()),
            SyntaxKind::ExternalModuleReference => {
                self.emit_external_module_reference(node.as_external_module_reference().as_node())
            }
            _ => panic!("unhandled ModuleReference: {:?}", node.kind),
        }
    }

    pub(crate) fn emit_named_export_bindings(&mut self, node: &Node) { ::tsox_core::fntrace::enter("emit_named_export_bindings"); 
        match node.kind {
            SyntaxKind::NamespaceExport => {
                self.emit_namespace_export(node.as_namespace_export().as_node())
            }
            SyntaxKind::NamedExports => self.emit_named_exports(node.as_named_exports().as_node()),
            _ => panic!("unhandled NamedExportBindings: {:?}", node.kind),
        }
    }

    pub(crate) fn emit_named_exports(&mut self, node: &Node) { ::tsox_core::fntrace::enter("emit_named_exports"); 
        let state = self.enter_node(node);
        let exports = node.as_named_exports();
        self.write_punctuation("{");
        self.emit_list(
            Self::emit_export_specifier_node,
            exports.as_node(),
            exports.elements(),
            LF_NAMED_IMPORTS_OR_EXPORTS_ELEMENTS,
        );
        self.write_punctuation("}");
        self.exit_node(exports.as_node(), state);
    }
}
