#![allow(dead_code, unused_imports, unused_variables)]

use std::cell::RefCell;
use std::sync::Arc;

use tsox_core::core::text::TextRange;
use tsox_core::stringutil::mig::m3m_2::guess_indentation;
use tsox_core::stringutil::split_lines;
use tsox_frontend::ast::is_prologue_directive;
use tsox_frontend::ast::mig::m3e_3::{get_leftmost_expression, OPERATOR_PRECEDENCE_LOWEST};
use tsox_frontend::ast::mig::m3g_3::{is_var_await_using, is_var_const, is_var_let, is_var_using};
use tsox_frontend::ast::node::{Node, NodeList, SourceFile};
use tsox_frontend::ast::node_data_generated::{
    is_block, is_class_expression, is_function_expression, NodeData,
};
use tsox_frontend::ast::node_source_file::ScriptKind;
use tsox_frontend::ast::syntax_kind_generated::SyntaxKind;
use tsox_frontend::ast::utilities::*;
use tsox_frontend::astnav::get_line_and_character_of_position;
use tsox_frontend::format::mig::m4o_2::EmitHelper;
use tsox_frontend::format::mig::m4t_4::is_immediately_invoked_function_expression_or_arrow_function;
use tsox_frontend::format::mig::m4t_5::is_pinned_comment;
use tsox_frontend::scanner::CommentRange;
use tsox_frontend::scanner;

#[path = "r37k4_defs.rs"]
mod r37k4_defs;

use self::r37k4_defs::{EmitContextExt37k4, NodeStatementExt37k4, SourceMapStateSnapshot};
use super::m4q::Printer;
use super::m4q::r33k12_defs::EmitFlags;
use super::m4q::r33k12_defs::OperatorPrecedence;
use super::m4q::r33k12_defs::{
    CommentSeparator, ListFlags, TokenEmitFlags, WriteKind, LF_CALL_EXPRESSION_ARGUMENTS,
    LF_CLASS_HERITAGE_CLAUSES, LF_CLASS_MEMBERS, LF_ENUM_MEMBERS, LF_HERITAGE_CLAUSES,
    LF_HERITAGE_CLAUSE_TYPES, LF_IMPORT_ATTRIBUTES, LF_INTERFACE_MEMBERS, LF_MULTI_LINE,
    LF_NONE, LF_VARIABLE_DECLARATION_LIST, is_not_emitted_statement,
};
use super::m4q::r33k12_defs::{
    LF_MULTI_LINE_BLOCK_STATEMENTS, LF_SINGLE_LINE_BLOCK_STATEMENTS,
};
use super::m4q::r39k08_defs::PrinterExtK08;
use crate::mig::m4q_2::r39k07_defs::{greatest_end07, skip_partially_emitted_expressions07};

#[path = "r39k04_defs.rs"]
pub mod r39k04_defs;

use super::m4q;

fn source_file_statements(node: &SourceFile) -> &NodeList { ::tsox_core::fntrace::enter("source_file_statements"); 
    match &node.node.data {
        NodeData::SourceFile(d) => &d.statements,
        _ => panic!("unhandled node data: {:?}", node.node.kind),
    }
}

struct DetachedCommentsInfo42k03 {
    node_pos: usize,
    detached_comment_end_pos: usize,
}

thread_local! {
    static DETACHED_COMMENTS_INFO_42K03: RefCell<Vec<DetachedCommentsInfo42k03>> =
        RefCell::new(Vec::new());
    static REGISTERED_EMIT_HELPERS_42K03: RefCell<Vec<Arc<EmitHelper>>> =
        RefCell::new(Vec::new());
}

fn registered_emit_helpers_42k03() -> Vec<Arc<EmitHelper>> { ::tsox_core::fntrace::enter("registered_emit_helpers_42k03"); 
    REGISTERED_EMIT_HELPERS_42K03.with(|helpers| helpers.borrow().clone())
}

impl Printer {
    pub(crate) fn emit_triple_slash_directives(&mut self, node: &Arc<SourceFile>) { ::tsox_core::fntrace::enter("emit_triple_slash_directives"); 
        self.emit_directive("path", &node.referenced_files);
        self.emit_directive("types", &node.type_reference_directives);
        self.emit_directive("lib", &node.lib_reference_directives);
    }

    pub(crate) fn emit_directive(
        &mut self,
        kind: &str,
        refs: &[tsox_frontend::ast::node_source_file::FileReference],
    ) { ::tsox_core::fntrace::enter("emit_directive"); 
        use tsox_core::core::compiler_options_kinds::ResolutionMode;
        for r in refs {
            let resolution_mode = if r.resolution_mode != ResolutionMode::None {
                format!(
                    "resolution-mode=\"{}\" ",
                    if r.resolution_mode == ResolutionMode::ESNext { "import" } else { "require" }
                )
            } else {
                String::new()
            };
            let preserve = if r.preserve { "preserve=\"true\" " } else { "" };
            self.write_comment(&format!(
                "/// <reference {}=\"{}\" {}{}/>",
                kind, r.file_name, resolution_mode, preserve
            ));
            self.write_line();
        }
    }

    pub(crate) fn emit_statement(&mut self, node: &Node) { ::tsox_core::fntrace::enter("emit_statement"); 
        if let Some(snippet_element) = self.emit_context.snippet_element(node) {
            self.emit_snippet_node(node, &snippet_element);
            return;
        }

        match node.kind {
            SyntaxKind::Block => self.emit_block(node),
            SyntaxKind::EmptyStatement => self.emit_empty_statement(node, false),
            SyntaxKind::VariableStatement => self.emit_variable_statement(node),
            SyntaxKind::ExpressionStatement => self.emit_expression_statement(node),
            SyntaxKind::IfStatement => self.emit_if_statement(node),
            SyntaxKind::DoStatement => self.emit_do_statement(node),
            SyntaxKind::WhileStatement => self.emit_while_statement(node),
            SyntaxKind::ForStatement => self.emit_for_statement(node),
            SyntaxKind::ForInStatement => self.emit_for_in_statement(node),
            SyntaxKind::ForOfStatement => self.emit_for_of_statement(node),
            SyntaxKind::ContinueStatement => self.emit_continue_statement(node),
            SyntaxKind::BreakStatement => self.emit_break_statement(node),
            SyntaxKind::ReturnStatement => self.emit_return_statement(node),
            SyntaxKind::WithStatement => self.emit_with_statement(node),
            SyntaxKind::SwitchStatement => self.emit_switch_statement(node),
            SyntaxKind::LabeledStatement => self.emit_labeled_statement(node),
            SyntaxKind::ThrowStatement => self.emit_throw_statement(node),
            SyntaxKind::TryStatement => self.emit_try_statement(node),
            SyntaxKind::DebuggerStatement => self.emit_debugger_statement(node),
            SyntaxKind::NotEmittedStatement => self.emit_not_emitted_statement(node),

            SyntaxKind::FunctionDeclaration => self.emit_function_declaration(node),
            SyntaxKind::ClassDeclaration => self.emit_class_declaration(node),
            SyntaxKind::InterfaceDeclaration => self.emit_interface_declaration(node),
            SyntaxKind::TypeAliasDeclaration | SyntaxKind::JSTypeAliasDeclaration => {
                self.emit_type_alias_declaration(node)
            }
            SyntaxKind::EnumDeclaration => self.emit_enum_declaration(node),
            SyntaxKind::ModuleDeclaration => self.emit_module_declaration(node),
            SyntaxKind::MissingDeclaration => {}

            SyntaxKind::NamespaceExportDeclaration => {
                self.emit_namespace_export_declaration(node)
            }
            SyntaxKind::ImportEqualsDeclaration => {
                self.emit_import_equals_declaration(node)
            }
            SyntaxKind::ImportDeclaration => self.emit_import_declaration(node),
            SyntaxKind::ExportAssignment => self.emit_export_assignment(node),
            SyntaxKind::ExportDeclaration => self.emit_export_declaration(node),

            _ => panic!("unhandled statement: {:?}", node.kind),
        }
    }

    pub(crate) fn emit_source_file(&mut self, node: &Arc<SourceFile>) { ::tsox_core::fntrace::enter("emit_source_file"); 
        let saved_current_source_file = self.current_source_file.take();
        let saved_comments_disabled = self.comments_disabled;
        self.current_source_file = Some(Arc::clone(node));

        self.write_line();

        self.push_name_generation_scope(&node.node);
        self.generate_all_names(source_file_statements(node));

        let mut index = 0;
        let mut state: Option<usize> = None;
        if node.script_kind != ScriptKind::Json {
            self.emit_shebang_if_needed(node);
            index = self.emit_prologue_directives(source_file_statements(node));
            if !self.emit_state39k04(|state| state.writer.is_at_start_of_line()) {
                self.write_line();
            }
            state = self.emit_detached_comments_before_statement_list(&node.node, &source_file_statements(node).loc);
            self.emit_helpers(&node.node);
            if node.is_declaration_file {
                self.emit_triple_slash_directives(node);
            }
        } else {
            state = self.emit_detached_comments_before_statement_list(&node.node, &source_file_statements(node).loc);
        }

        self.emit_list_range(
            Self::emit_statement,
            &node.node,
            Some(source_file_statements(node)),
            LF_MULTI_LINE,
            index,
            usize::MAX,
        );
        self.pop_name_generation_scope(&node.node);
        self.emit_detached_comments_after_statement_list(&node.node, &source_file_statements(node).loc, state);
        self.current_source_file = saved_current_source_file;
        self.comments_disabled = saved_comments_disabled;
    }

    pub(crate) fn emit_pos(&mut self, pos: usize) { ::tsox_core::fntrace::enter("emit_pos"); 
        if self.emit_state39k04(|state| {
            state.source_maps_disabled
                || state.source_map_source.is_none()
                || state.source_map_generator.is_none()
                || state.source_map_source_is_json
        }) || position_is_synthesized(pos)
        {
            return;
        }

        let mut source = self.emit_state39k04(|state| state.source_map_source.clone().unwrap());
        let mut source_index = self.emit_state39k04(|state| state.source_map_source_index);
        let mut line_char_cache =
            self.emit_state39k04(|state| state.source_map_line_char_cache.clone().unwrap());
        let mut pos = pos;
        let mapped = self.emit_state39k04_mut(|state| {
            state
                .map_source_position
                .as_mut()
                .and_then(|map| map(&source, pos))
        });
        match mapped {
            None => {
                self.emit_state39k04_mut(|state| {
                    let writer = state.writer.as_mut();
                    state
                        .source_map_generator
                        .as_mut()
                        .unwrap()
                        .add_generated_mapping(writer.get_line(), writer.get_column())
                        .unwrap_or_else(|err| panic!("{}", err));
                });
                return;
            }
            Some((mapped_source, mapped_pos)) => {
                pos = mapped_pos;
                if !Arc::ptr_eq(&mapped_source, &source) {
                    let saved_source = source.clone();
                    let saved_source_index = self.emit_state39k04(|state| state.source_map_source_index);
                    let saved_source_is_json = self.emit_state39k04(|state| state.source_map_source_is_json);
                    let saved_line_char_cache =
                        self.emit_state39k04(|state| state.source_map_line_char_cache.clone());
                    self.set_source_map_source(&mapped_source);
                    source_index = self.emit_state39k04(|state| state.source_map_source_index);
                    line_char_cache =
                        self.emit_state39k04(|state| state.source_map_line_char_cache.clone().unwrap());
                    self.emit_state39k04_mut(|state| {
                        state.source_map_source = Some(saved_source);
                        state.source_map_source_index = saved_source_index;
                        state.source_map_source_is_json = saved_source_is_json;
                        state.source_map_line_char_cache = saved_line_char_cache;
                    });
                    source = mapped_source;
                }
            }
        }

        let (source_line, source_character) = line_char_cache.get_line_and_character(pos);
        self.emit_state39k04_mut(|state| {
            let writer = state.writer.as_mut();
            state
                .source_map_generator
                .as_mut()
                .unwrap()
                .add_source_mapping(
                    writer.get_line(),
                    writer.get_column(),
                    source_index,
                    source_line,
                    source_character,
                )
                .unwrap_or_else(|err| panic!("{}", err));
        });
    }

    pub(crate) fn emit_source_pos(&mut self, source: &Arc<SourceFile>, pos: usize) { ::tsox_core::fntrace::enter("emit_source_pos"); 
        let same_source = self.emit_state39k04(|state| {
            state.source_map_source.as_ref().is_some_and(|s| Arc::ptr_eq(s, source))
        });
        if !same_source {
            let saved = self.emit_state39k04(|state| {
                (
                    state.source_map_source.clone(),
                    state.source_map_source_index,
                    state.source_map_line_char_cache.clone(),
                )
            });
            self.set_source_map_source(source);
            self.emit_pos(pos);
            self.emit_state39k04_mut(|state| {
                state.source_map_source = saved.0;
                state.source_map_source_index = saved.1;
                state.source_map_line_char_cache = saved.2;
            });
        } else {
            self.emit_pos(pos);
        }
    }

    pub(crate) fn emit_source_maps_before_node(&mut self, node: &Node) -> Option<SourceMapStateSnapshot> { ::tsox_core::fntrace::enter("emit_source_maps_before_node"); 
        if !self.should_emit_source_maps(node) {
            return None;
        }

        let emit_flags = self.emit_context.emit_flags_of(node);
        let loc = self.emit_context.source_map_range(node);

        if !is_not_emitted_statement(node)
            && emit_flags & EmitFlags::NO_LEADING_SOURCE_MAP.0 == 0
            && self.current_source_file.is_some()
            && !position_is_synthesized(loc.pos())
        {
            let source_file = self.current_source_file.as_ref().unwrap();
            let source_map_source = self.emit_state39k04(|state| state.source_map_source.clone().unwrap());
            self.emit_source_pos(
                &source_map_source,
                scanner::skip_trivia(source_file.text.as_str(), loc.pos()),
            );
        }

        if emit_flags & EmitFlags::NO_NESTED_SOURCE_MAPS.0 != 0 {
            self.emit_state39k04_mut(|state| state.source_maps_disabled = true);
        }

        Some(SourceMapStateSnapshot::new(emit_flags, loc, false))
    }

    pub(crate) fn emit_source_maps_after_node(&mut self, node: &Node, previous_state: Option<SourceMapStateSnapshot>) { ::tsox_core::fntrace::enter("emit_source_maps_after_node"); 
        let Some(previous_state) = previous_state else { return };

        let emit_flags = previous_state.emit_flags;
        let loc = previous_state.source_map_range;

        if emit_flags & EmitFlags::NO_NESTED_SOURCE_MAPS.0 != 0 {
            self.emit_state39k04_mut(|state| state.source_maps_disabled = false);
        }

        if !is_not_emitted_statement(node)
            && emit_flags & EmitFlags::NO_TRAILING_SOURCE_MAP.0 == 0
            && !position_is_synthesized(loc.end())
        {
            let source_map_source = self.emit_state39k04(|state| state.source_map_source.clone().unwrap());
            self.emit_source_pos(&source_map_source, loc.end());
        }
    }

    pub(crate) fn emit_source_maps_before_token(
        &mut self,
        token: SyntaxKind,
        mut pos: usize,
        context_node: &Node,
        flags: TokenEmitFlags,
    ) -> Option<SourceMapStateSnapshot> { ::tsox_core::fntrace::enter("emit_source_maps_before_token"); 
        if !self.should_emit_token_source_maps(token, pos, context_node, flags) {
            return None;
        }

        let emit_flags = self.emit_context.emit_flags_of(context_node);
        let (loc, has_loc) = self.emit_context.token_source_map_range(context_node, token);
        if has_loc {
            pos = loc.pos();
        }
        if pos != usize::MAX && self.current_source_file.is_some() {
            pos = scanner::skip_trivia(self.current_source_file.as_ref().unwrap().text.as_str(), pos);
        }
        if emit_flags & EmitFlags::NO_TOKEN_LEADING_SOURCE_MAPS.0 == 0 && pos != usize::MAX {
            let source_map_source = self.emit_state39k04(|state| state.source_map_source.clone().unwrap());
            self.emit_source_pos(&source_map_source, pos);
        }

        Some(SourceMapStateSnapshot::new(emit_flags, loc, has_loc))
    }

    pub(crate) fn emit_source_maps_after_token(
        &mut self,
        token: SyntaxKind,
        pos: usize,
        context_node: &Node,
        previous_state: Option<SourceMapStateSnapshot>,
    ) { ::tsox_core::fntrace::enter("emit_source_maps_after_token"); 
        let Some(previous_state) = previous_state else { return };

        let emit_flags = previous_state.emit_flags;
        let loc = previous_state.source_map_range;
        let has_loc = previous_state.has_token_source_map_range;
        if emit_flags & EmitFlags::NO_TOKEN_TRAILING_SOURCE_MAPS.0 == 0 {
            let end = if has_loc { loc.end() } else { pos };
            if end != usize::MAX && !position_is_synthesized(end) {
                let source_map_source = self.emit_state39k04(|state| state.source_map_source.clone().unwrap());
                self.emit_source_pos(&source_map_source, end);
            }
        }
    }

    pub(crate) fn emit_trailing_comments(&mut self, pos: usize, comment_separator: CommentSeparator) { ::tsox_core::fntrace::enter("emit_trailing_comments"); 
        if self.comments_disabled {
            return;
        }
        let Some(source_file) = self.current_source_file.clone() else { return };
        if self.container_end != usize::MAX
            && (pos == self.container_end || pos == self.declaration_list_container_end)
        {
            return;
        }

        let mut comments: Vec<CommentRange> = Vec::new();
        for comment in scanner::get_trailing_comment_ranges(&source_file.text, pos) {
            if self.should_write_comment(&comment) {
                comments.push(comment);
            }
        }

        self.emit_comments(&comments, comment_separator);
    }

    pub(crate) fn emit_trailing_comments_of_node(
        &mut self,
        node: &Node,
        emit_flags: EmitFlags,
        comment_range: TextRange,
        container_pos: usize,
        container_end: usize,
        declaration_list_container_end: usize,
    ) { ::tsox_core::fntrace::enter("emit_trailing_comments_of_node"); 
        let pos = comment_range.pos();
        let end = comment_range.end();
        let skip_trailing_comments =
            end == usize::MAX || emit_flags.contains(EmitFlags::NO_TRAILING_COMMENTS) || node.kind == SyntaxKind::JsxText;
        if (!position_is_synthesized(pos) || !position_is_synthesized(end)) && pos != end {
            self.container_pos = container_pos;
            self.container_end = container_end;
            self.declaration_list_container_end = declaration_list_container_end;

            if !skip_trailing_comments && node.kind != SyntaxKind::NotEmittedStatement {
                self.emit_trailing_comments(end, CommentSeparator::Before);
            }
        }
    }

    pub(crate) fn emit_trailing_comments_of_position(&mut self, pos: usize, prefix_space: bool, force_no_newline: bool) { ::tsox_core::fntrace::enter("emit_trailing_comments_of_position"); 
        if self.comments_disabled || self.current_source_file.is_none() {
            return;
        }
        if self.container_end != usize::MAX
            && (pos == self.container_end || pos == self.declaration_list_container_end)
        {
            return;
        }

        let source_file = self.current_source_file.clone().unwrap();
        let comments: Vec<CommentRange> = scanner::get_trailing_comment_ranges(&source_file.text, pos);
        if comments.is_empty() {
            return;
        }

        for comment in comments {
            if prefix_space {
                if !self.should_write_comment(&comment) {
                    continue;
                }
                if !self.emit_state39k04(|state| state.writer.is_at_start_of_line()) {
                    self.write_space();
                }
                self.emit_comment(&comment);
                if comment.has_trailing_new_line {
                    self.write_line();
                }
                continue;
            }

            self.emit_comment(&comment);
            if !force_no_newline && comment.has_trailing_new_line {
                self.write_line();
            }
        }
    }

    pub(crate) fn emit_block(&mut self, node: &Node) { ::tsox_core::fntrace::enter("emit_block"); 
        let state = self.enter_node(node);
        let block = node.as_block();
        self.generate_names(Some(node));
        self.emit_token(SyntaxKind::OpenBraceToken, node.pos(), WriteKind::Punctuation, node);

        let format = if (!block.multi_line && self.is_empty_block(node, &block.statements))
            || self.should_emit_on_single_line(node)
        {
            LF_SINGLE_LINE_BLOCK_STATEMENTS
        } else {
            LF_MULTI_LINE_BLOCK_STATEMENTS
        };
        self.emit_list(Self::emit_statement, node, &block.statements, format);

        self.emit_token_ex(
            SyntaxKind::CloseBraceToken,
            block.statements.end(),
            WriteKind::Punctuation,
            node,
            if (format & LF_MULTI_LINE) != 0 {
                TokenEmitFlags::INDENT_LEADING_COMMENTS
            } else {
                TokenEmitFlags::NONE
            },
        );
        self.exit_node(node, state);
    }

    pub(crate) fn emit_variable_statement(&mut self, node: &Node) { ::tsox_core::fntrace::enter("emit_variable_statement"); 
        let state = self.enter_node(node);
        let statement = node.as_variable_statement();
        self.emit_modifier_list(node, statement.modifiers.as_deref(), false);
        self.emit_variable_declaration_list(&statement.declaration_list);
        self.write_trailing_semicolon();
        self.exit_node(node, state);
    }

    pub(crate) fn emit_variable_declaration_list(&mut self, node: &Arc<Node>) { ::tsox_core::fntrace::enter("emit_variable_declaration_list"); 
        let state = self.enter_node(node);
        let list = match &node.data {
            NodeData::VariableDeclarationList(d) => d,
            _ => panic!("unexpected VariableDeclarationList: {:?}", node.kind),
        };
        if is_var_let(node) {
            self.write_keyword("let");
        } else if is_var_const(node) {
            self.write_keyword("const");
        } else if is_var_using(node) {
            self.write_keyword("using");
        } else if is_var_await_using(node) {
            self.write_keyword("await");
            self.write_space();
            self.write_keyword("using");
        } else {
            self.write_keyword("var");
        }
        self.write_space();
        self.emit_list(
            Self::emit_variable_declaration_node,
            node,
            &list.declarations,
            LF_VARIABLE_DECLARATION_LIST,
        );
        self.exit_node(node, state);
    }

    pub(crate) fn emit_variable_declaration_node(&mut self, node: &Node) { ::tsox_core::fntrace::enter("emit_variable_declaration_node"); 
        self.emit_variable_declaration(node);
    }

    pub(crate) fn emit_variable_declaration(&mut self, node: &Node) { ::tsox_core::fntrace::enter("emit_variable_declaration"); 
        let state = self.enter_node(node);
        let declaration = match &node.data {
            NodeData::VariableDeclaration(d) => d,
            _ => panic!("unexpected VariableDeclaration: {:?}", node.kind),
        };
        self.emit_binding_name(&declaration.name);
        self.emit_punctuation_node(declaration.exclamation_token.as_deref());
        self.emit_type_annotation(declaration.type_node.as_deref());
        self.emit_initializer(
            declaration.initializer.as_deref(),
            greatest_end07(declaration.name.end(), &[declaration.type_node.as_deref(), None]),
            node,
        );
        self.exit_node(node, state);
    }

    pub(crate) fn emit_empty_statement(&mut self, node: &Node, is_embedded_statement: bool) { ::tsox_core::fntrace::enter("emit_empty_statement"); 
        let state = self.enter_node(node);

        if is_embedded_statement {
            self.write_punctuation(";");
        } else {
            self.write_trailing_semicolon();
        }
        self.exit_node(node, state);
    }

    pub(crate) fn emit_expression_statement(&mut self, node: &Node) { ::tsox_core::fntrace::enter("emit_expression_statement"); 
        let state = self.enter_node(node);
        let statement = node.as_expression_statement();

        if self
            .current_source_file
            .as_ref()
            .is_some_and(|file| file.script_kind == ScriptKind::Json)
        {
            self.emit_expression(&statement.expression, OperatorPrecedence::Comma);
        } else if is_immediately_invoked_function_expression_or_arrow_function(&statement.expression) {
            self.emit_iife_with_parenthesized_callee(&statement.expression);
        } else {
            match get_leftmost_expression(&statement.expression, false).kind {
                SyntaxKind::FunctionExpression | SyntaxKind::ObjectLiteralExpression => {
                    self.emit_expression(&statement.expression, OperatorPrecedence::Parentheses)
                }
                _ => self.emit_expression(&statement.expression, OperatorPrecedence::Comma),
            }
        }

        if self.current_source_file.is_none()
            || !self
                .current_source_file
                .as_ref()
                .is_some_and(|file| file.script_kind == ScriptKind::Json)
            || node_is_synthesized(&statement.expression)
        {
            self.write_trailing_semicolon();
        }

        self.exit_node(node, state);
    }

    pub(crate) fn emit_iife_with_parenthesized_callee(&mut self, node: &Node) { ::tsox_core::fntrace::enter("emit_iife_with_parenthesized_callee"); 
        let call = skip_partially_emitted_expressions07(node);
        let state = self.enter_node(call);
        let call_expression = match &call.data {
            NodeData::CallExpression(d) => d,
            _ => panic!("unexpected CallExpression: {:?}", call.kind),
        };
        self.write_punctuation("(");
        self.emit_expression(&call_expression.expression, OPERATOR_PRECEDENCE_LOWEST);
        self.write_punctuation(")");
        self.emit_token_node(call_expression.question_dot_token.as_deref());
        self.emit_type_arguments(call, call_expression.type_arguments.as_deref());
        self.emit_list(
            Self::emit_argument,
            call,
            &call_expression.arguments,
            LF_CALL_EXPRESSION_ARGUMENTS,
        );
        self.exit_node(call, state);
    }

    pub(crate) fn emit_if_statement(&mut self, node: &Node) { ::tsox_core::fntrace::enter("emit_if_statement"); 
        let state = self.enter_node(node);
        let statement = node.as_if_statement();
        let pos = self.emit_token(SyntaxKind::IfKeyword, node.pos(), WriteKind::Keyword, node);
        self.write_space();
        self.emit_token(SyntaxKind::OpenParenToken, pos, WriteKind::Punctuation, node);
        self.emit_expression(&statement.expression, OPERATOR_PRECEDENCE_LOWEST);
        self.emit_token(
            SyntaxKind::CloseParenToken,
            statement.expression.end(),
            WriteKind::Punctuation,
            node,
        );
        self.emit_embedded_statement(node, &statement.then_statement);
        if let Some(else_statement) = statement.else_statement.as_deref() {
            self.write_line_or_space(node, &statement.then_statement, else_statement);
            self.emit_token(
                SyntaxKind::ElseKeyword,
                statement.then_statement.end(),
                WriteKind::Keyword,
                node,
            );
            if else_statement.kind == SyntaxKind::IfStatement {
                self.write_space();
                self.emit_if_statement(else_statement);
            } else {
                self.emit_embedded_statement(node, else_statement);
            }
        }
        self.exit_node(node, state);
    }

    pub(crate) fn emit_while_clause(&mut self, node: &Node, expression: &Node, start_pos: usize) { ::tsox_core::fntrace::enter("emit_while_clause"); 
        let pos = self.emit_token(SyntaxKind::WhileKeyword, start_pos, WriteKind::Keyword, node);
        self.write_space();
        self.emit_token(SyntaxKind::OpenParenToken, pos, WriteKind::Punctuation, node);
        self.emit_expression(expression, OPERATOR_PRECEDENCE_LOWEST);
        self.emit_token(
            SyntaxKind::CloseParenToken,
            expression.end(),
            WriteKind::Punctuation,
            node,
        );
    }

    pub(crate) fn emit_do_statement(&mut self, node: &Node) { ::tsox_core::fntrace::enter("emit_do_statement"); 
        let state = self.enter_node(node);
        let statement = node.as_do_statement();
        self.emit_token(SyntaxKind::DoKeyword, node.pos(), WriteKind::Keyword, node);
        self.emit_embedded_statement(node, &statement.statement);
        if is_block(&statement.statement) && !self.options.preserve_source_newlines {
            self.write_space();
        } else {
            self.write_line_or_space(node, &statement.statement, &statement.expression);
        }

        self.emit_while_clause(node, &statement.expression, statement.statement.end());
        self.write_trailing_semicolon();
        self.exit_node(node, state);
    }

    pub(crate) fn emit_while_statement(&mut self, node: &Node) { ::tsox_core::fntrace::enter("emit_while_statement"); 
        let state = self.enter_node(node);
        let statement = node.as_while_statement();
        self.emit_while_clause(node, &statement.expression, node.pos());
        self.emit_embedded_statement(node, &statement.statement);
        self.exit_node(node, state);
    }

    pub(crate) fn emit_for_initializer(&mut self, node: &Arc<Node>) { ::tsox_core::fntrace::enter("emit_for_initializer"); 
        if node.kind == SyntaxKind::VariableDeclarationList {
            self.emit_variable_declaration_list(node);
        } else {
            self.emit_expression(node, OPERATOR_PRECEDENCE_LOWEST);
        }
    }

    pub(crate) fn emit_for_statement(&mut self, node: &Node) { ::tsox_core::fntrace::enter("emit_for_statement"); 
        let state = self.enter_node(node);
        let statement = node.as_for_statement();
        let mut pos = self.emit_token(SyntaxKind::ForKeyword, node.pos(), WriteKind::Keyword, node);
        self.write_space();
        pos = self.emit_token(SyntaxKind::OpenParenToken, pos, WriteKind::Punctuation, node);
        if let Some(initializer) = &statement.initializer {
            self.emit_for_initializer(initializer);
            pos = initializer.end();
        }
        pos = self.emit_token(SyntaxKind::SemicolonToken, pos, WriteKind::Punctuation, node);
        if let Some(condition) = &statement.condition {
            self.write_space();
            self.emit_expression(condition, OPERATOR_PRECEDENCE_LOWEST);
            pos = condition.end();
        }
        pos = self.emit_token(SyntaxKind::SemicolonToken, pos, WriteKind::Punctuation, node);
        if let Some(incrementor) = &statement.incrementor {
            self.write_space();
            self.emit_expression(incrementor, OPERATOR_PRECEDENCE_LOWEST);
            pos = incrementor.end();
        }
        self.emit_token(SyntaxKind::CloseParenToken, pos, WriteKind::Punctuation, node);
        self.emit_embedded_statement(node, &statement.statement);
        self.exit_node(node, state);
    }

    pub(crate) fn emit_for_in_statement(&mut self, node: &Node) { ::tsox_core::fntrace::enter("emit_for_in_statement"); 
        let state = self.enter_node(node);
        let statement = node.as_for_in_or_of_statement();
        let pos = self.emit_token(SyntaxKind::ForKeyword, node.pos(), WriteKind::Keyword, node);
        self.write_space();
        self.emit_token(SyntaxKind::OpenParenToken, pos, WriteKind::Punctuation, node);
        self.emit_for_initializer(&statement.initializer);
        self.write_space();
        self.emit_token(
            SyntaxKind::InKeyword,
            statement.initializer.end(),
            WriteKind::Keyword,
            node,
        );
        self.write_space();
        self.emit_expression(&statement.expression, OPERATOR_PRECEDENCE_LOWEST);
        self.emit_token(
            SyntaxKind::CloseParenToken,
            statement.expression.end(),
            WriteKind::Punctuation,
            node,
        );
        self.emit_embedded_statement(node, &statement.statement);
        self.exit_node(node, state);
    }

    pub(crate) fn emit_for_of_statement(&mut self, node: &Node) { ::tsox_core::fntrace::enter("emit_for_of_statement"); 
        let state = self.enter_node(node);
        let statement = node.as_for_in_or_of_statement();
        let open_paren_pos = self.emit_token(SyntaxKind::ForKeyword, node.pos(), WriteKind::Keyword, node);
        self.write_space();
        if let Some(await_modifier) = statement.await_modifier.as_deref() {
            self.emit_keyword_node(await_modifier);
            self.write_space();
        }
        self.emit_token(SyntaxKind::OpenParenToken, open_paren_pos, WriteKind::Punctuation, node);
        self.emit_for_initializer(&statement.initializer);
        self.write_space();
        self.emit_token(
            SyntaxKind::OfKeyword,
            statement.initializer.end(),
            WriteKind::Keyword,
            node,
        );
        self.write_space();
        self.emit_expression(&statement.expression, OPERATOR_PRECEDENCE_LOWEST);
        self.emit_token(
            SyntaxKind::CloseParenToken,
            statement.expression.end(),
            WriteKind::Punctuation,
            node,
        );
        self.emit_embedded_statement(node, &statement.statement);
        self.exit_node(node, state);
    }

    pub(crate) fn emit_continue_statement(&mut self, node: &Node) { ::tsox_core::fntrace::enter("emit_continue_statement"); 
        let state = self.enter_node(node);
        let statement = node.as_continue_statement();
        self.emit_token(SyntaxKind::ContinueKeyword, node.pos(), WriteKind::Keyword, node);
        if let Some(label) = statement.label.as_deref() {
            self.write_space();
            self.emit_label_identifier(label);
        }
        self.write_trailing_semicolon();
        self.exit_node(node, state);
    }

    pub(crate) fn emit_break_statement(&mut self, node: &Node) { ::tsox_core::fntrace::enter("emit_break_statement"); 
        let state = self.enter_node(node);
        let statement = node.as_break_statement();
        self.emit_token(SyntaxKind::BreakKeyword, node.pos(), WriteKind::Keyword, node);
        if let Some(label) = statement.label.as_deref() {
            self.write_space();
            self.emit_label_identifier(label);
        }
        self.write_trailing_semicolon();
        self.exit_node(node, state);
    }

    pub(crate) fn emit_label_identifier(&mut self, node: &Node) { ::tsox_core::fntrace::enter("emit_label_identifier"); 
        let state = self.enter_node(node);
        self.emit_identifier_text(node);
        self.exit_node(node, state);
    }

    pub(crate) fn emit_with_statement(&mut self, node: &Node) { ::tsox_core::fntrace::enter("emit_with_statement"); 
        let state = self.enter_node(node);
        let statement = node.as_with_statement();
        let pos = self.emit_token(SyntaxKind::WithKeyword, node.pos(), WriteKind::Keyword, node);
        self.write_space();
        self.emit_token(SyntaxKind::OpenParenToken, pos, WriteKind::Punctuation, node);
        self.emit_expression(&statement.expression, OPERATOR_PRECEDENCE_LOWEST);
        self.emit_token(
            SyntaxKind::CloseParenToken,
            statement.expression.end(),
            WriteKind::Punctuation,
            node,
        );
        self.emit_embedded_statement(node, &statement.statement);
        self.exit_node(node, state);
    }

    pub(crate) fn emit_labeled_statement(&mut self, node: &Node) { ::tsox_core::fntrace::enter("emit_labeled_statement"); 
        let state = self.enter_node(node);
        let statement = node.as_labeled_statement();
        self.emit_label_identifier(&statement.label);
        self.emit_token(
            SyntaxKind::ColonToken,
            statement.label.end(),
            WriteKind::Punctuation,
            node,
        );
        self.write_space();
        self.emit_statement(&statement.statement);
        self.exit_node(node, state);
    }

    pub(crate) fn emit_try_statement(&mut self, node: &Node) { ::tsox_core::fntrace::enter("emit_try_statement"); 
        let state = self.enter_node(node);
        let statement = node.as_try_statement();
        self.emit_token(SyntaxKind::TryKeyword, node.pos(), WriteKind::Keyword, node);
        self.write_space();
        self.emit_block(&statement.try_block);
        if let Some(catch_clause) = statement.catch_clause.as_deref() {
            self.write_line_or_space(node, &statement.try_block, catch_clause);
            self.emit_catch_clause(catch_clause);
        }
        if let Some(finally_block) = statement.finally_block.as_deref() {
            let previous = statement.catch_clause.as_deref().unwrap_or(&statement.try_block);
            self.write_line_or_space(node, previous, finally_block);
            self.emit_token(SyntaxKind::FinallyKeyword, previous.end(), WriteKind::Keyword, node);
            self.write_space();
            self.emit_block(finally_block);
        }
        self.exit_node(node, state);
    }

    pub(crate) fn emit_catch_clause(&mut self, node: &Node) { ::tsox_core::fntrace::enter("emit_catch_clause"); 
        let state = self.enter_node(node);
        let clause = match &node.data {
            NodeData::CatchClause(d) => d,
            _ => panic!("unexpected CatchClause: {:?}", node.kind),
        };
        let open_paren_pos = self.emit_token(SyntaxKind::CatchKeyword, node.pos(), WriteKind::Keyword, node);
        self.write_space();

        if let Some(variable_declaration) = clause.variable_declaration.as_deref() {
            self.emit_token(SyntaxKind::OpenParenToken, open_paren_pos, WriteKind::Punctuation, node);
            self.emit_variable_declaration(variable_declaration);
            self.emit_token(
                SyntaxKind::CloseParenToken,
                variable_declaration.end(),
                WriteKind::Punctuation,
                node,
            );
            self.write_space();
        }

        self.emit_block(&clause.block);
        self.exit_node(node, state);
    }

    pub(crate) fn emit_debugger_statement(&mut self, node: &Node) { ::tsox_core::fntrace::enter("emit_debugger_statement"); 
        let state = self.enter_node(node);
        self.emit_token(SyntaxKind::DebuggerKeyword, node.pos(), WriteKind::Keyword, node);
        self.write_trailing_semicolon();
        self.exit_node(node, state);
    }

    pub(crate) fn emit_embedded_statement(&mut self, parent_node: &Node, node: &Node) { ::tsox_core::fntrace::enter("emit_embedded_statement"); 
        if is_block(node)
            || self.should_emit_on_single_line(parent_node)
            || self.options.preserve_source_newlines
                && self.get_leading_line_terminator_count07(parent_node, Some(node), LF_NONE) == 0
        {
            self.write_space();
            self.emit_statement(node);
        } else {
            self.write_line();
            self.increase_indent();
            if node.kind == SyntaxKind::EmptyStatement {
                self.emit_empty_statement(node, true);
            } else {
                self.emit_statement(node);
            }
            self.decrease_indent();
        }
    }

    pub(crate) fn emit_function_declaration(&mut self, node: &Node) { ::tsox_core::fntrace::enter("emit_function_declaration"); 
        let state = self.enter_node(node);
        let declaration = node.as_function_declaration();
        self.generate_name_if_needed(declaration.name.as_ref());
        self.emit_modifier_list(node, declaration.modifiers.as_deref(), false);
        self.write_keyword("function");
        self.emit_token_node(declaration.asterisk_token.as_deref());
        self.write_space();
        if let Some(name) = declaration.name.as_deref() {
            self.emit_identifier_name(name);
        }
        let indented = self.should_emit_indented(node);
        self.increase_indent_if(indented);
        self.push_name_generation_scope(node);
        self.emit_signature(node);
        self.emit_function_body_node(declaration.body.as_deref());
        self.pop_name_generation_scope(node);
        self.decrease_indent_if(indented);
        self.exit_node(node, state);
    }

    pub(crate) fn emit_class_declaration(&mut self, node: &Node) { ::tsox_core::fntrace::enter("emit_class_declaration"); 
        let state = self.enter_node(node);
        let declaration = node.as_class_declaration();
        self.generate_name_if_needed(declaration.name.as_ref());
        let pos = self.emit_modifier_list(node, declaration.modifiers.as_deref(), true);
        self.emit_token(SyntaxKind::ClassKeyword, pos, WriteKind::Keyword, node);
        if let Some(name) = declaration.name.as_deref() {
            self.write_space();
            self.emit_identifier_name(name);
        }
        let indented = self.should_emit_indented(node);
        self.increase_indent_if(indented);
        self.emit_type_parameters(node, declaration.type_parameters.as_deref());
        self.emit_list_range(
            Self::emit_heritage_clause_node,
            node,
            declaration.heritage_clauses.as_deref(),
            LF_CLASS_HERITAGE_CLAUSES,
            0,
            usize::MAX,
        );
        self.write_space();
        self.write_punctuation("{");
        self.push_name_generation_scope(node);
        self.generate_all_member_names(&declaration.members);
        self.emit_list(Self::emit_class_element, node, &declaration.members, LF_CLASS_MEMBERS);
        self.pop_name_generation_scope(node);
        self.write_punctuation("}");
        self.decrease_indent_if(indented);
        self.exit_node(node, state);
    }

    pub(crate) fn emit_interface_declaration(&mut self, node: &Node) { ::tsox_core::fntrace::enter("emit_interface_declaration"); 
        let state = self.enter_node(node);
        let declaration = node.as_interface_declaration();
        self.emit_modifier_list(node, declaration.modifiers.as_deref(), false);
        self.write_keyword("interface");
        self.write_space();
        self.emit_binding_identifier(&declaration.name);
        self.emit_type_parameters(node, declaration.type_parameters.as_deref());
        self.emit_list_range(
            Self::emit_heritage_clause_node,
            node,
            declaration.heritage_clauses.as_deref(),
            LF_HERITAGE_CLAUSES,
            0,
            usize::MAX,
        );
        self.write_space();
        self.write_punctuation("{");
        self.push_name_generation_scope(node);
        self.generate_all_member_names(&declaration.members);
        self.emit_list(Self::emit_type_element, node, &declaration.members, LF_INTERFACE_MEMBERS);
        self.pop_name_generation_scope(node);
        self.write_punctuation("}");
        self.exit_node(node, state);
    }

    pub(crate) fn emit_type_alias_declaration(&mut self, node: &Node) { ::tsox_core::fntrace::enter("emit_type_alias_declaration"); 
        let state = self.enter_node(node);
        let declaration = node.as_type_alias_declaration();
        self.emit_modifier_list(node, declaration.modifiers.as_deref(), false);
        self.write_keyword("type");
        self.write_space();
        self.emit_binding_identifier(&declaration.name);
        self.emit_type_parameters(node, declaration.type_parameters.as_deref());
        self.write_space();
        self.write_punctuation("=");
        self.write_space();
        self.emit_type_node_outside_extends(&declaration.type_node);
        self.write_trailing_semicolon();
        self.exit_node(node, state);
    }

    pub(crate) fn emit_enum_declaration(&mut self, node: &Node) { ::tsox_core::fntrace::enter("emit_enum_declaration"); 
        let state = self.enter_node(node);
        let declaration = node.as_enum_declaration();
        self.emit_modifier_list(node, declaration.modifiers.as_deref(), false);
        self.write_keyword("enum");
        self.write_space();
        self.emit_binding_identifier(&declaration.name);
        self.write_space();
        self.write_punctuation("{");
        self.emit_list(Self::emit_enum_member_node, node, &declaration.members, LF_ENUM_MEMBERS);
        self.write_punctuation("}");
        self.exit_node(node, state);
    }

    pub(crate) fn emit_enum_member_node(&mut self, node: &Node) { ::tsox_core::fntrace::enter("emit_enum_member_node"); 
        self.emit_enum_member(node);
    }

    pub(crate) fn emit_enum_member(&mut self, node: &Node) { ::tsox_core::fntrace::enter("emit_enum_member"); 
        let state = self.enter_node(node);
        let member = match &node.data {
            NodeData::EnumMember(d) => d,
            _ => panic!("unexpected EnumMember: {:?}", node.kind),
        };
        self.emit_property_name(Some(&member.name));
        self.emit_initializer(member.initializer.as_deref(), member.name.end(), node);
        self.exit_node(node, state);
    }

    pub(crate) fn emit_import_equals_declaration(&mut self, node: &Node) { ::tsox_core::fntrace::enter("emit_import_equals_declaration"); 
        let state = self.enter_node(node);
        let declaration = node.as_import_equals_declaration();
        let modifier_pos = self.emit_modifier_list(node, declaration.modifiers.as_deref(), false);
        let pos = self.emit_token(SyntaxKind::ImportKeyword, modifier_pos, WriteKind::Keyword, node);
        self.write_space();
        if declaration.is_type_only {
            self.emit_token(SyntaxKind::TypeKeyword, pos, WriteKind::Keyword, node);
            self.write_space();
        }
        self.emit_binding_identifier(&declaration.name);
        self.write_space();
        self.emit_token(
            SyntaxKind::EqualsToken,
            declaration.name.end(),
            WriteKind::Punctuation,
            node,
        );
        self.write_space();
        self.emit_module_reference(&declaration.module_reference);
        self.write_trailing_semicolon();
        self.exit_node(node, state);
    }

    pub(crate) fn emit_import_declaration(&mut self, node: &Node) { ::tsox_core::fntrace::enter("emit_import_declaration"); 
        let state = self.enter_node(node);
        let declaration = node.as_import_declaration();
        let modifier_pos = self.emit_modifier_list(node, declaration.modifiers.as_deref(), false);
        self.emit_token(SyntaxKind::ImportKeyword, modifier_pos, WriteKind::Keyword, node);
        self.write_space();
        if let Some(import_clause) = declaration.import_clause.as_deref() {
            self.emit_import_clause(import_clause);
            self.write_space();
            self.emit_token(SyntaxKind::FromKeyword, import_clause.end(), WriteKind::Keyword, node);
            self.write_space();
        }
        self.emit_expression(&declaration.module_specifier, OPERATOR_PRECEDENCE_LOWEST);
        if let Some(attributes) = declaration.attributes.as_deref() {
            self.write_space();
            self.emit_import_attributes(attributes);
        }
        self.write_trailing_semicolon();
        self.exit_node(node, state);
    }

    pub(crate) fn emit_import_clause(&mut self, node: &Node) { ::tsox_core::fntrace::enter("emit_import_clause"); 
        let state = self.enter_node(node);
        let clause = match &node.data {
            NodeData::ImportClause(d) => d,
            _ => panic!("unexpected ImportClause: {:?}", node.kind),
        };
        if let Some(phase_modifier) = clause.phase_modifier {
            self.emit_token(phase_modifier, node.pos(), WriteKind::Keyword, node);
            self.write_space();
        }
        if let Some(name) = clause.name.as_deref() {
            self.emit_binding_identifier(name);
            if clause.named_bindings.is_some() {
                self.emit_token(SyntaxKind::CommaToken, name.end(), WriteKind::Punctuation, node);
                self.write_space();
            }
        }
        self.emit_named_import_bindings(clause.named_bindings.as_deref());
        self.exit_node(node, state);
    }

    pub(crate) fn emit_import_attributes(&mut self, node: &Node) { ::tsox_core::fntrace::enter("emit_import_attributes"); 
        let state = self.enter_node(node);
        let attributes = match &node.data {
            NodeData::ImportAttributes(d) => d,
            _ => panic!("unexpected ImportAttributes: {:?}", node.kind),
        };
        self.emit_token(attributes.token, node.pos(), WriteKind::Keyword, node);
        self.write_space();
        self.emit_list(
            Self::emit_import_attribute_node,
            node,
            &attributes.attributes,
            LF_IMPORT_ATTRIBUTES,
        );
        self.exit_node(node, state);
    }

    pub(crate) fn emit_export_assignment(&mut self, node: &Node) { ::tsox_core::fntrace::enter("emit_export_assignment"); 
        let state = self.enter_node(node);
        let assignment = node.as_export_assignment();
        let next_pos = self.emit_token(SyntaxKind::ExportKeyword, node.pos(), WriteKind::Keyword, node);
        self.write_space();
        if assignment.is_export_equals {
            self.emit_token(SyntaxKind::EqualsToken, next_pos, WriteKind::Operator, node);
        } else {
            self.emit_token(SyntaxKind::DefaultKeyword, next_pos, WriteKind::Keyword, node);
        }
        self.write_space();
        if assignment.is_export_equals {
            self.emit_expression(&assignment.expression, OperatorPrecedence::Assignment);
        } else {
            let expr = get_leftmost_expression(&assignment.expression, false);
            if is_class_expression(&expr) || is_function_expression(&expr) {
                self.emit_expression(&assignment.expression, OperatorPrecedence::Parentheses);
            } else {
                self.emit_expression(&assignment.expression, OperatorPrecedence::Assignment);
            }
        }
        self.write_trailing_semicolon();
        self.exit_node(node, state);
    }

    pub(crate) fn emit_export_declaration(&mut self, node: &Node) { ::tsox_core::fntrace::enter("emit_export_declaration"); 
        let state = self.enter_node(node);
        let declaration = node.as_export_declaration();
        self.emit_modifier_list(node, declaration.modifiers.as_deref(), false);
        let mut pos = self.emit_token(SyntaxKind::ExportKeyword, node.pos(), WriteKind::Keyword, node);
        self.write_space();
        if declaration.is_type_only {
            pos = self.emit_token(SyntaxKind::TypeKeyword, pos, WriteKind::Keyword, node);
            self.write_space();
        }
        if let Some(export_clause) = declaration.export_clause.as_deref() {
            self.emit_named_export_bindings(export_clause);
        } else {
            pos = self.emit_token(SyntaxKind::AsteriskToken, pos, WriteKind::Punctuation, node);
        }
        if let Some(module_specifier) = declaration.module_specifier.as_deref() {
            self.write_space();
            self.emit_token(
                SyntaxKind::FromKeyword,
                greatest_end07(pos, &[declaration.export_clause.as_deref()]),
                WriteKind::Keyword,
                node,
            );
            self.write_space();
            self.emit_expression(module_specifier, OPERATOR_PRECEDENCE_LOWEST);
        }
        if let Some(attributes) = declaration.attributes.as_deref() {
            self.write_space();
            self.emit_import_attributes(attributes);
        }
        self.write_trailing_semicolon();
        self.exit_node(node, state);
    }

    pub(crate) fn emit_heritage_clause_node(&mut self, node: &Node) { ::tsox_core::fntrace::enter("emit_heritage_clause_node"); 
        self.emit_heritage_clause(node);
    }

    pub(crate) fn emit_heritage_clause_element(&mut self, node: &Node) { ::tsox_core::fntrace::enter("emit_heritage_clause_element"); 
        match node.kind {
            SyntaxKind::ExpressionWithTypeArguments => self.emit_expression_with_type_arguments(node),
            SyntaxKind::TypeReference => self.emit_type_reference(node),
            _ => panic!("unhandled HeritageClauseElement: {:?}", node.kind),
        }
    }

    pub(crate) fn write_line_or_space(&mut self, parent_node: &Node, prev_child_node: &Node, next_child_node: &Node) { ::tsox_core::fntrace::enter("write_line_or_space"); 
        if self.should_emit_on_single_line(parent_node) {
            self.write_space();
        } else if self.options.preserve_source_newlines {
            let lines = self.get_lines_between_nodes(parent_node, prev_child_node, next_child_node);
            if lines > 0 {
                self.write_line_repeat(lines);
            } else {
                self.write_space();
            }
        } else {
            self.write_line();
        }
    }

    pub(crate) fn write_lines(&mut self, text: &str) { ::tsox_core::fntrace::enter("write_lines"); 
        let lines = split_lines(text);
        let indentation = guess_indentation(&lines);
        for line in &lines {
            let line = if indentation > 0 { &line[indentation..] } else { line.as_str() };
            if !line.is_empty() {
                self.write_line();
                self.write(line);
            }
        }
    }

    pub(crate) fn emit_helpers(&mut self, node: &Node) -> bool { ::tsox_core::fntrace::enter("emit_helpers"); 
        let mut helpers_emitted = false;
        let should_skip = self.options.no_emit_helpers;
        let helpers = registered_emit_helpers_42k03();
        if !helpers.is_empty() {
            let mut sorted = helpers;
            sorted.sort_by_key(|helper| helper.priority.as_ref().map(|priority| priority.value));
            for helper in &sorted {
                if !helper.scoped && should_skip {
                    continue;
                }
                match &helper.text_callback {
                    Some(text_callback) => {
                        let name_maker = |name: &str| name.to_string();
                        let text = text_callback(&name_maker);
                        self.write_lines(&text);
                    }
                    None => self.write_lines(&helper.text),
                }
                helpers_emitted = true;
            }
        }
        helpers_emitted
    }

    pub(crate) fn should_emit_detached_comments(&self, node: &Node) -> bool { ::tsox_core::fntrace::enter("should_emit_detached_comments"); 
        if node.kind != SyntaxKind::SourceFile {
            return true;
        }
        let statements = match &node.data {
            NodeData::SourceFile(d) => &d.statements,
            _ => panic!("unhandled node data: {:?}", node.kind),
        };
        statements.nodes.is_empty()
            || !is_prologue_directive(&statements.nodes[0])
            || node_is_synthesized(&statements.nodes[0])
    }

    pub(crate) fn emit_detached_comments_and_update_comments_info(&mut self, text_range: TextRange) { ::tsox_core::fntrace::enter("emit_detached_comments_and_update_comments_info"); 
        if self.current_source_file.is_none() {
            return;
        }
        if let Some(info) = self.emit_detached_comments(text_range) {
            DETACHED_COMMENTS_INFO_42K03.with(|stack| stack.borrow_mut().push(info));
        }
    }

    fn emit_detached_comments(&mut self, text_range: TextRange) -> Option<DetachedCommentsInfo42k03> { ::tsox_core::fntrace::enter("emit_detached_comments"); 
        let source_file = self.current_source_file.clone()?;
        let text = source_file.text.clone();

        let mut leading_comments: Vec<CommentRange> = Vec::new();
        if self.comments_disabled {
            if text_range.pos() == 0 {
                for comment in scanner::get_leading_comment_ranges(&text, text_range.pos()) {
                    if is_pinned_comment(&text, &comment) {
                        leading_comments.push(comment);
                    }
                }
            }
        } else {
            leading_comments = scanner::get_leading_comment_ranges(&text, text_range.pos());
        }

        if leading_comments.is_empty() {
            return None;
        }

        let mut detached_comments: Vec<CommentRange> = Vec::new();
        let mut last_comment: Option<CommentRange> = None;
        for comment in leading_comments {
            if let Some(last) = &last_comment {
                let last_comment_line = get_line_and_character_of_position(&source_file, last.end).0;
                let comment_line = get_line_and_character_of_position(&source_file, comment.pos).0;

                if comment_line >= last_comment_line + 2 {
                    break;
                }
            }

            detached_comments.push(comment);
            last_comment = Some(comment);
        }

        let last_detached = detached_comments.last()?;
        let last_comment_line = get_line_and_character_of_position(&source_file, last_detached.end).0;
        let node_line = get_line_and_character_of_position(
            &source_file,
            scanner::skip_trivia(&text, text_range.pos()),
        )
        .0;
        if node_line < last_comment_line + 2 {
            return None;
        }

        let comments_to_emit: Vec<CommentRange> = detached_comments
            .iter()
            .filter(|comment| self.should_write_comment(comment))
            .copied()
            .collect();

        if !comments_to_emit.is_empty() {
            if self.should_emit_new_line_before_leading_comment_of_position(
                text_range.pos(),
                comments_to_emit[0].pos,
            ) {
                self.write_line();
            }

            self.emit_comments(&comments_to_emit, CommentSeparator::After);
        }

        Some(DetachedCommentsInfo42k03 {
            node_pos: text_range.pos(),
            detached_comment_end_pos: detached_comments
                .last()
                .map(|comment| comment.end)
                .unwrap_or(text_range.pos()),
        })
    }

    pub(crate) fn emit_detached_comments_before_statement_list(
        &mut self,
        node: &Node,
        detached_range: &TextRange,
    ) -> Option<usize> { ::tsox_core::fntrace::enter("emit_detached_comments_before_statement_list"); 
        if !self.should_emit_detached_comments(node) {
            return None;
        }

        let emit_flags = self.emit_context.emit_flags_of(node);
        let skip_leading_comments = position_is_synthesized(detached_range.pos())
            || (emit_flags & EmitFlags::NO_LEADING_COMMENTS.0) != 0;

        if !skip_leading_comments {
            self.emit_detached_comments_and_update_comments_info(*detached_range);
        }

        if (emit_flags & EmitFlags::NO_NESTED_COMMENTS.0) != 0 {
            self.comments_disabled = true;
        }

        Some(emit_flags as usize)
    }

    pub(crate) fn emit_detached_comments_after_statement_list(
        &mut self,
        node: &Node,
        detached_range: &TextRange,
        state: Option<usize>,
    ) { ::tsox_core::fntrace::enter("emit_detached_comments_after_statement_list"); 
        let Some(emit_flags) = state else {
            return;
        };
        let emit_flags = emit_flags as u32;

        let skip_trailing_comments = self.comments_disabled
            || position_is_synthesized(detached_range.end())
            || (emit_flags & EmitFlags::NO_TRAILING_COMMENTS.0) != 0;

        if !skip_trailing_comments {
            let has_written_comment = self.emit_leading_comments(detached_range.end(), false);
            if has_written_comment && !self.emit_state39k04(|state| state.writer.is_at_start_of_line()) {
                self.write_line();
            }
        }
    }

    pub(crate) fn should_emit_new_line_before_leading_comment_of_position(
        &self,
        pos: usize,
        comment_pos: usize,
    ) -> bool { ::tsox_core::fntrace::enter("should_emit_new_line_before_leading_comment_of_position"); 
        let Some(source_file) = self.current_source_file.as_ref() else {
            return false;
        };
        pos != comment_pos
            && get_line_and_character_of_position(source_file, pos).0
                != get_line_and_character_of_position(source_file, comment_pos).0
    }
}
