use std::sync::Arc;

use crate::ast::mig::m3b::{label as node_label, members as node_members};
use crate::ast::mig::m3c::{
    statement_list, statements as node_statements, type_parameters as node_type_parameters,
};
use crate::ast::mig::m3e_3::{OperatorPrecedence, OPERATOR_PRECEDENCE_LOWEST};
use crate::ast::node_data_generated::NodeData;
use crate::ast::node_node::Node;
use crate::ast::node_node_list::NodeList;
use crate::ast::syntax_kind_generated::SyntaxKind;
use crate::ast::utilities_synthesized::{node_is_synthesized, position_is_synthesized};
use crate::scanner::is_conflict_marker_trivia::{get_leading_comment_ranges, get_trailing_comment_ranges, skip_trivia};
use crate::scanner::is_jsx_line_break::{CommentRange, CommentRangeKind};
use crate::scanner::mig::w1::compute_line_of_position;
use tsox_core::core::text::TextRange;

use super::m4o::{CommentSeparator, ListFormat, TokenEmitFlags, EF_NO_NESTED_COMMENTS, EF_NO_TRAILING_COMMENTS};
use super::m4o_2::{CommentState, DetachedCommentsInfo, Printer, WriteKind};
use super::m4t_3::{
    get_ecma_line_starts, positions_are_on_same_line, range_start_positions_are_on_same_line,
};
use super::m4t_5::is_pinned_comment;

fn heritage_clauses(node: &Node) -> &[Arc<Node>] { ::tsox_core::fntrace::enter("heritage_clauses"); 
    match &node.data {
        NodeData::ClassDeclaration(d) => heritage_clause_list(&d.heritage_clauses),
        NodeData::ClassExpression(d) => heritage_clause_list(&d.heritage_clauses),
        _ => panic!("Unhandled case in Node.HeritageClauses"),
    }
}

fn heritage_clause_list(heritage_clauses: &Option<Arc<NodeList>>) -> &[Arc<Node>] { ::tsox_core::fntrace::enter("heritage_clause_list"); 
    heritage_clauses
        .as_ref()
        .map(|l| l.nodes.as_slice())
        .unwrap_or(&[])
}

fn case_block_clauses(node: &Node) -> &NodeList { ::tsox_core::fntrace::enter("case_block_clauses"); 
    match &node.data {
        NodeData::CaseBlock(d) => &d.clauses,
        _ => panic!("Unhandled case in Node.Clauses"),
    }
}

fn catch_variable_declaration(node: &Node) -> Option<&Arc<Node>> { ::tsox_core::fntrace::enter("catch_variable_declaration"); 
    match &node.data {
        NodeData::CatchClause(d) => d.variable_declaration.as_ref(),
        _ => panic!("Unhandled case in Node.VariableDeclaration"),
    }
}

fn catch_block(node: &Node) -> &Arc<Node> { ::tsox_core::fntrace::enter("catch_block"); 
    match &node.data {
        NodeData::CatchClause(d) => &d.block,
        _ => panic!("Unhandled case in Node.Block"),
    }
}

fn block_multi_line(node: &Node) -> bool { ::tsox_core::fntrace::enter("block_multi_line"); 
    match &node.data {
        NodeData::Block(d) => d.multi_line,
        _ => panic!("Unhandled case in Node.MultiLine"),
    }
}

impl Printer {
    pub fn emit_as_expression(&mut self, node: &Arc<Node>) { ::tsox_core::fntrace::enter("emit_as_expression"); 
        let state = self.enter_node(node);
        self.emit_expression(
            node.expression().expect("as expression requires expression"),
            OperatorPrecedence::Relational,
        );
        self.write_space();
        self.write_keyword("as");
        self.write_space();
        self.emit_type_node_outside_extends(
            node.type_node().expect("as expression requires type node"),
        );
        self.exit_node(node, state);
    }

    pub fn emit_class_expression(&mut self, node: &Arc<Node>) { ::tsox_core::fntrace::enter("emit_class_expression"); 
        let state = self.enter_node(node);
        self.generate_name_needed(node.name());
        let pos = self.emit_modifier_list(node, node.modifiers(), true);
        self.emit_token(SyntaxKind::ClassKeyword, pos, WriteKind::Keyword, node);

        if let Some(name) = node.name() {
            self.write_space();
            self.emit_identifier_name(name);
        }

        let indented = self.should_emit_indented(node);
        self.increase_indent_if(indented);

        self.emit_type_parameters(node, node_type_parameters(node));
        self.emit_list(
            Printer::emit_heritage_clause_node,
            node,
            heritage_clauses(node),
            ListFormat::CLASS_HERITAGE_CLAUSES,
        );
        self.write_space();
        self.write_punctuation("{");
        self.push_name_generation_scope(node);
        self.generate_all_member_names(node_members(node));
        self.emit_list(
            Printer::emit_class_element,
            node,
            node_members(node),
            ListFormat::CLASS_MEMBERS,
        );
        self.pop_name_generation_scope(node);
        self.write_punctuation("}");

        self.decrease_indent_if(indented);
        self.exit_node(node, state);
    }

    pub fn emit_class_declaration(&mut self, node: &Arc<Node>) { ::tsox_core::fntrace::enter("emit_class_declaration"); 
        let state = self.enter_node(node);
        self.generate_name_needed(node.name());
        let pos = self.emit_modifier_list(node, node.modifiers(), true);
        self.emit_token(SyntaxKind::ClassKeyword, pos, WriteKind::Keyword, node);
        if let Some(name) = node.name() {
            self.write_space();
            self.emit_identifier_name(name);
        }
        let indented = self.should_emit_indented(node);
        self.increase_indent_if(indented);
        self.emit_type_parameters(node, node_type_parameters(node));
        self.emit_list(
            Printer::emit_heritage_clause_node,
            node,
            heritage_clauses(node),
            ListFormat::CLASS_HERITAGE_CLAUSES,
        );
        self.write_space();
        self.write_punctuation("{");
        self.push_name_generation_scope(node);
        self.generate_all_member_names(node_members(node));
        self.emit_list(
            Printer::emit_class_element,
            node,
            node_members(node),
            ListFormat::CLASS_MEMBERS,
        );
        self.pop_name_generation_scope(node);
        self.write_punctuation("}");
        self.decrease_indent_if(indented);
        self.exit_node(node, state);
    }

    pub fn emit_block(&mut self, node: &Arc<Node>) { ::tsox_core::fntrace::enter("emit_block"); 
        let state = self.enter_node(node);
        self.generate_names(node);
        self.emit_token(SyntaxKind::OpenBraceToken, node.pos(), WriteKind::Punctuation, node);

        let format = if !block_multi_line(node) && self.is_empty_block(node, node_statements(node))
            || self.should_emit_on_single_line(node)
        {
            ListFormat::SINGLE_LINE_BLOCK_STATEMENTS
        } else {
            ListFormat::MULTI_LINE_BLOCK_STATEMENTS
        };
        self.emit_list(Printer::emit_statement, node, node_statements(node), format);

        let statements_end = statement_list(node)
            .expect("block requires statement list")
            .end();
        self.emit_token_ex(
            SyntaxKind::CloseBraceToken,
            statements_end,
            WriteKind::Punctuation,
            node,
            if format.contains(ListFormat::MULTI_LINE) {
                TokenEmitFlags::INDENT_LEADING_COMMENTS
            } else {
                TokenEmitFlags::NONE
            },
        );
        self.exit_node(node, state);
    }

    pub fn emit_continue_statement(&mut self, node: &Arc<Node>) { ::tsox_core::fntrace::enter("emit_continue_statement"); 
        let state = self.enter_node(node);
        self.emit_token(SyntaxKind::ContinueKeyword, node.pos(), WriteKind::Keyword, node);
        if let Some(label) = node_label(node) {
            self.write_space();
            self.emit_label_identifier(label);
        }
        self.write_trailing_semicolon();
        self.exit_node(node, state);
    }

    pub fn emit_break_statement(&mut self, node: &Arc<Node>) { ::tsox_core::fntrace::enter("emit_break_statement"); 
        let state = self.enter_node(node);
        self.emit_token(SyntaxKind::BreakKeyword, node.pos(), WriteKind::Keyword, node);
        if let Some(label) = node_label(node) {
            self.write_space();
            self.emit_label_identifier(label);
        }
        self.write_trailing_semicolon();
        self.exit_node(node, state);
    }

    pub fn emit_debugger_statement(&mut self, node: &Arc<Node>) { ::tsox_core::fntrace::enter("emit_debugger_statement"); 
        let state = self.enter_node(node);
        self.emit_token(SyntaxKind::DebuggerKeyword, node.pos(), WriteKind::Keyword, node);
        self.write_trailing_semicolon();
        self.exit_node(node, state);
    }

    pub fn emit_case_block(&mut self, node: &Arc<Node>) { ::tsox_core::fntrace::enter("emit_case_block"); 
        let state = self.enter_node(node);
        self.emit_token(SyntaxKind::OpenBraceToken, node.pos(), WriteKind::Punctuation, node);
        let clauses = case_block_clauses(node);
        self.emit_list(
            Printer::emit_case_or_default_clause_node,
            node,
            &clauses.nodes,
            ListFormat::CASE_BLOCK_CLAUSES,
        );
        self.emit_token_ex(
            SyntaxKind::CloseBraceToken,
            clauses.end(),
            WriteKind::Punctuation,
            node,
            TokenEmitFlags::INDENT_LEADING_COMMENTS,
        );
        self.exit_node(node, state);
    }

    pub fn emit_case_or_default_clause_statements(&mut self, node: &Arc<Node>, colon_pos: usize) { ::tsox_core::fntrace::enter("emit_case_or_default_clause_statements"); 
        let statements = node_statements(node);
        let emit_as_single_statement = statements.len() == 1
            && (self
                .current_source_file
                .is_none()
                || node_is_synthesized(node)
                || node_is_synthesized(&statements[0])
                || range_start_positions_are_on_same_line(
                    node.loc,
                    statements[0].loc,
                    self.current_source_file.as_ref().unwrap(),
                ));

        let mut format = ListFormat::CASE_OR_DEFAULT_CLAUSE_STATEMENTS;
        if emit_as_single_statement {
            self.write_token_text(SyntaxKind::ColonToken, WriteKind::Punctuation, colon_pos);
            self.write_space();
            format &= !(ListFormat::MULTI_LINE | ListFormat::INDENTED);
        } else {
            self.emit_token(SyntaxKind::ColonToken, colon_pos, WriteKind::Punctuation, node);
        }

        self.emit_list(Printer::emit_statement, node, statements, format);
    }

    pub fn emit_case_clause(&mut self, node: &Arc<Node>) { ::tsox_core::fntrace::enter("emit_case_clause"); 
        let state = self.enter_node(node);
        self.emit_token(SyntaxKind::CaseKeyword, node.pos(), WriteKind::Keyword, node);
        self.write_space();
        self.emit_expression(
            node.expression().expect("case clause requires expression"),
            OPERATOR_PRECEDENCE_LOWEST,
        );
        let expression_end = node
            .expression()
            .expect("case clause requires expression")
            .end();
        self.emit_case_or_default_clause_statements(node, expression_end);
        self.exit_node(node, state);
    }

    pub fn emit_default_clause(&mut self, node: &Arc<Node>) { ::tsox_core::fntrace::enter("emit_default_clause"); 
        let state = self.enter_node(node);
        let pos = self.emit_token(SyntaxKind::DefaultKeyword, node.pos(), WriteKind::Keyword, node);
        self.emit_case_or_default_clause_statements(node, pos);
        self.exit_node(node, state);
    }

    pub fn emit_case_or_default_clause_node(&mut self, node: &Arc<Node>) { ::tsox_core::fntrace::enter("emit_case_or_default_clause_node"); 
        match node.kind {
            SyntaxKind::CaseClause => self.emit_case_clause(node),
            SyntaxKind::DefaultClause => self.emit_default_clause(node),
            _ => panic!("unhandled CaseOrDefaultClause: {:?}", node.kind),
        }
    }

    pub fn emit_catch_clause(&mut self, node: &Arc<Node>) { ::tsox_core::fntrace::enter("emit_catch_clause"); 
        let state = self.enter_node(node);
        let open_paren_pos =
            self.emit_token(SyntaxKind::CatchKeyword, node.pos(), WriteKind::Keyword, node);
        self.write_space();

        if let Some(variable_declaration) = catch_variable_declaration(node) {
            self.emit_token(
                SyntaxKind::OpenParenToken,
                open_paren_pos,
                WriteKind::Punctuation,
                node,
            );
            self.emit_variable_declaration(variable_declaration);
            self.emit_token(
                SyntaxKind::CloseParenToken,
                variable_declaration.end(),
                WriteKind::Punctuation,
                node,
            );
            self.write_space();
        }

        self.emit_block(catch_block(node));
        self.exit_node(node, state);
    }

    pub fn emit_comments_before_node(&mut self, node: &Arc<Node>) -> Option<CommentState> { ::tsox_core::fntrace::enter("emit_comments_before_node"); 
        if !self.should_emit_comments(node) {
            return None;
        }

        let emit_flags = self.emit_context.emit_flags(node);
        let comment_range = self.emit_context.comment_range(node);
        let container_pos = self.container_pos;
        let container_end = self.container_end;
        let declaration_list_container_end = self.declaration_list_container_end;

        self.emit_leading_comments_of_node(node, emit_flags, comment_range);
        self.emit_leading_synthetic_comments_of_node(node, emit_flags);
        if (emit_flags & EF_NO_NESTED_COMMENTS.0) != 0 {
            self.comments_disabled = true;
        }

        Some(CommentState {
            emit_flags: emit_flags as i32,
            comment_range,
            container_pos,
            container_end,
            declaration_list_container_end,
        })
    }

    pub fn emit_comments_after_node(&mut self, node: &Arc<Node>, state: Option<CommentState>) { ::tsox_core::fntrace::enter("emit_comments_after_node"); 
        let Some(state) = state else {
            return;
        };

        let emit_flags = state.emit_flags as u32;
        let comment_range = state.comment_range;
        let container_pos = state.container_pos;
        let container_end = state.container_end;
        let declaration_list_container_end = state.declaration_list_container_end;

        if (emit_flags & EF_NO_NESTED_COMMENTS.0) != 0 {
            self.comments_disabled = false;
        }

        self.emit_trailing_synthetic_comments_of_node(node, emit_flags);
        self.emit_trailing_comments_of_node(
            node,
            emit_flags,
            comment_range,
            container_pos,
            container_end,
            declaration_list_container_end,
        );

        if let Some(type_node) = self.emit_context.get_type_node(node) {
            self.emit_trailing_comments_of_node(
                node,
                emit_flags,
                type_node.loc,
                container_pos,
                container_end,
                declaration_list_container_end,
            );
        }
    }

    pub fn emit_comments_before_token(
        &mut self,
        token: SyntaxKind,
        mut pos: usize,
        context_node: &Arc<Node>,
        flags: TokenEmitFlags,
    ) -> (Option<CommentState>, usize) { ::tsox_core::fntrace::enter("emit_comments_before_token"); 
        if flags.contains(TokenEmitFlags::NO_COMMENTS) || self.comments_disabled {
            if let Some(source_file) = &self.current_source_file {
                if !position_is_synthesized(pos) {
                    pos = skip_trivia(&source_file.text, pos);
                }
            }
            return (None, pos);
        }

        let start_pos = pos;
        if let Some(source_file) = &self.current_source_file {
            pos = skip_trivia(&source_file.text, start_pos);
        }

        let node = self.emit_context.parse_node(context_node);
        let is_similar_node = matches!(node, Some(ref node) if node.kind == context_node.kind);
        if !is_similar_node {
            return (None, pos);
        }

        if context_node.pos() != start_pos {
            let indent_leading = flags.contains(TokenEmitFlags::INDENT_LEADING_COMMENTS);
            let needs_indent = indent_leading
                && self.current_source_file.is_some()
                && !positions_are_on_same_line(
                    start_pos as i64,
                    pos as i64,
                    self.current_source_file.as_ref().unwrap(),
                );
            self.increase_indent_if(needs_indent);
            self.emit_leading_comments(start_pos, false);
            self.decrease_indent_if(needs_indent);
        }

        (Some(CommentState::default()), pos)
    }

    pub fn emit_comments_after_token(
        &mut self,
        token: SyntaxKind,
        pos: usize,
        context_node: &Arc<Node>,
        state: Option<CommentState>,
    ) { ::tsox_core::fntrace::enter("emit_comments_after_token"); 
        let Some(_state) = state else {
            return;
        };

        if context_node.end() != pos {
            let is_jsx_expr_context = context_node.kind == SyntaxKind::JsxExpression;
            self.emit_trailing_comments(
                pos,
                if is_jsx_expr_context {
                    CommentSeparator::None
                } else {
                    CommentSeparator::Before
                },
            );
        }
    }

    pub fn emit_detached_comments(
        &mut self,
        text_range: TextRange,
    ) -> Option<DetachedCommentsInfo> { ::tsox_core::fntrace::enter("emit_detached_comments"); 
        let Some(source_file) = self.current_source_file.clone() else {
            return None;
        };

        let text = source_file.text.as_str();
        let line_map = get_ecma_line_starts(&source_file);

        let mut leading_comments: Vec<CommentRange> = Vec::new();
        if self.comments_disabled {
            if text_range.pos() == 0 {
                for comment in get_leading_comment_ranges(text, text_range.pos()) {
                    if is_pinned_comment(text, &comment) {
                        leading_comments.push(comment);
                    }
                }
            }
        } else {
            leading_comments = get_leading_comment_ranges(text, text_range.pos());
        }

        if !leading_comments.is_empty() {
            let mut detached_comments: Vec<CommentRange> = Vec::new();
            let mut last_comment: Option<CommentRange> = None;
            for (i, comment) in leading_comments.iter().enumerate() {
                if i > 0 {
                    let last_comment_line = compute_line_of_position(
                        &line_map,
                        last_comment.as_ref().unwrap().end as i32,
                    );
                    let comment_line = compute_line_of_position(&line_map, comment.pos as i32);

                    if comment_line >= last_comment_line + 2 {
                        break;
                    }
                }

                detached_comments.push(comment.clone());
                last_comment = Some(comment.clone());
            }

            if !detached_comments.is_empty() {
                let last_comment_line = compute_line_of_position(
                    &line_map,
                    detached_comments.last().unwrap().end as i32,
                );
                let node_line =
                    compute_line_of_position(&line_map, skip_trivia(text, text_range.pos()) as i32);
                if node_line >= last_comment_line + 2 {
                    let mut comments_to_emit: Vec<CommentRange> = Vec::new();
                    for comment in &detached_comments {
                        if self.should_write_comment(comment) {
                            comments_to_emit.push(comment.clone());
                        }
                    }

                    if !comments_to_emit.is_empty() {
                        if self.should_emit_new_line_before_leading_comment_of_position(
                            text_range.pos(),
                            comments_to_emit[0].pos,
                        ) {
                            self.write_line();
                        }

                        self.emit_comments(&comments_to_emit, CommentSeparator::After);
                    }
                    return Some(DetachedCommentsInfo {
                        node_pos: text_range.pos(),
                        detached_comment_end_pos: detached_comments.last().unwrap().end,
                    });
                }
            }
        }
        None
    }

    pub fn emit_comments(
        &mut self,
        comments: &[CommentRange],
        comment_separator: CommentSeparator,
    ) -> bool { ::tsox_core::fntrace::enter("emit_comments"); 
        let mut intervening_separator = false;
        if comments.is_empty() {
            return false;
        }

        if comment_separator == CommentSeparator::Before {
            self.write_space();
        }

        for comment in comments {
            if intervening_separator {
                self.write_space();
                intervening_separator = false;
            }

            self.emit_comment(comment);

            if matches!(comment.kind, CommentRangeKind::SingleLine)
                || comment.has_trailing_new_line && comment_separator != CommentSeparator::None
            {
                self.write_line();
            } else {
                intervening_separator = comment_separator != CommentSeparator::None;
            }
        }

        if intervening_separator && comment_separator == CommentSeparator::After {
            self.write_space();
        }

        true
    }

    pub fn emit_comment(&mut self, comment: &CommentRange) { ::tsox_core::fntrace::enter("emit_comment"); 
        self.emit_pos(comment.pos);
        self.write_comment_range(comment);
        self.emit_pos(comment.end);
    }

    pub fn emit_detached_comments_after_statement_list(
        &mut self,
        node: &Arc<Node>,
        detached_range: TextRange,
        state: Option<CommentState>,
    ) { ::tsox_core::fntrace::enter("emit_detached_comments_after_statement_list"); 
        let Some(state) = state else {
            return;
        };

        let emit_flags = state.emit_flags as u32;
        let skip_trailing_comments = self.comments_disabled
            || position_is_synthesized(detached_range.end())
            || (emit_flags & EF_NO_TRAILING_COMMENTS.0) != 0;

        if !skip_trailing_comments {
            let has_written_comment = self.emit_leading_comments(detached_range.end(), false);
            let writer_text = self.writer.as_ref().map(|w| w.string()).unwrap_or_default();
            if has_written_comment && !(writer_text.is_empty() || writer_text.ends_with('\n')) {
                self.write_line();
            }
        }
    }
}

impl Default for CommentState {
    fn default() -> Self { ::tsox_core::fntrace::enter("default"); 
        CommentState {
            emit_flags: 0,
            comment_range: TextRange::default(),
            container_pos: -1,
            container_end: -1,
            declaration_list_container_end: -1,
        }
    }
}

impl Printer {
    pub fn should_emit_comments(&self, node: &Arc<Node>) -> bool { ::tsox_core::fntrace::enter("should_emit_comments"); 
        !self.comments_disabled
            && (self.emit_context.emit_flags(node) & super::m4o::EmitFlags::NO_COMMENTS.0)
                != super::m4o::EmitFlags::NO_COMMENTS.0
    }

    pub fn should_write_comment(&self, comment: &CommentRange) -> bool { ::tsox_core::fntrace::enter("should_write_comment"); 
        let _ = comment;
        !self.comments_disabled
    }

    pub fn should_emit_new_line_before_leading_comment_of_position(
        &mut self,
        pos: usize,
        comment_pos: usize,
    ) -> bool { ::tsox_core::fntrace::enter("should_emit_new_line_before_leading_comment_of_position"); 
        let Some(source_file) = self.current_source_file.clone() else {
            return false;
        };
        let line_map = get_ecma_line_starts(&source_file);
        let node_line = compute_line_of_position(&line_map, pos as i32);
        let comment_line = compute_line_of_position(&line_map, comment_pos as i32);
        comment_line > node_line
    }

    pub fn emit_leading_comments(&mut self, pos: usize, elided: bool) -> bool { ::tsox_core::fntrace::enter("emit_leading_comments"); 
        let _ = elided;
        if self.comments_disabled {
            return false;
        }
        let Some(source_file) = self.current_source_file.clone() else {
            return false;
        };
        let text = source_file.text.as_str();
        let comments = get_leading_comment_ranges(text, pos);
        if comments.is_empty() {
            return false;
        }
        self.emit_comments(&comments, CommentSeparator::After)
    }

    pub fn emit_leading_comments_of_node(
        &mut self,
        node: &Arc<Node>,
        emit_flags: u32,
        comment_range: TextRange,
    ) { ::tsox_core::fntrace::enter("emit_leading_comments_of_node"); 
        let _ = node;
        if (emit_flags & super::m4o::EmitFlags::NO_LEADING_COMMENTS.0) != 0 {
            return;
        }
        self.emit_leading_comments(comment_range.pos(), false);
    }

    pub fn emit_leading_synthetic_comments_of_node(&mut self, node: &Arc<Node>, emit_flags: u32) { ::tsox_core::fntrace::enter("emit_leading_synthetic_comments_of_node"); 
        let _ = (node, emit_flags);
    }

    pub fn emit_trailing_comments(&mut self, pos: usize, separator: CommentSeparator) { ::tsox_core::fntrace::enter("emit_trailing_comments"); 
        if self.comments_disabled {
            return;
        }
        let Some(source_file) = self.current_source_file.clone() else {
            return;
        };
        let comments = get_trailing_comment_ranges(source_file.text.as_str(), pos);
        if comments.is_empty() {
            return;
        }
        self.emit_comments(&comments, separator);
    }

    pub fn emit_trailing_comments_of_node(
        &mut self,
        node: &Arc<Node>,
        emit_flags: u32,
        comment_range: TextRange,
        container_pos: i64,
        container_end: i64,
        declaration_list_container_end: i64,
    ) { ::tsox_core::fntrace::enter("emit_trailing_comments_of_node"); 
        let _ = (
            node,
            container_pos,
            container_end,
            declaration_list_container_end,
        );
        if (emit_flags & super::m4o::EmitFlags::NO_TRAILING_COMMENTS.0) != 0 {
            return;
        }
        self.emit_trailing_comments(comment_range.end(), CommentSeparator::Before);
    }

    pub fn emit_trailing_synthetic_comments_of_node(&mut self, node: &Arc<Node>, emit_flags: u32) { ::tsox_core::fntrace::enter("emit_trailing_synthetic_comments_of_node"); 
        let _ = (node, emit_flags);
    }

    pub fn emit_pos(&mut self, _pos: usize) { ::tsox_core::fntrace::enter("emit_pos"); }

    pub fn write_comment_range(&mut self, comment: &CommentRange) { ::tsox_core::fntrace::enter("write_comment_range"); 
        let Some(source_file) = self.current_source_file.clone() else {
            return;
        };
        let text = source_file.text.as_str();
        let comment_text = text.get(comment.pos..comment.end).unwrap_or("");
        if let Some(writer) = self.writer.as_ref() {
            writer.write(comment_text);
        }
    }
}
