use crate::ast::*;
use crate::core::text::TextRange;
use crate::diagnostics::{self, Message};
use crate::scanner::token_to_string;
use std::sync::Arc;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum JSDocState {
    BeginningOfLine,
    SawAsterisk,
    SavingComments,
    SavingBackticks,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct PropertyLikeParse(u8);

impl PropertyLikeParse {
    const PROPERTY: u8 = 1;
    const PARAMETER: u8 = 2;
    const CALLBACK_PARAMETER: u8 = 4;

    fn contains(self, flag: u8) -> bool {
        self.0 & flag != 0
    }
}

impl super::Parser {

    pub fn parse_jsdoc_comment(
        &mut self,
        start: usize,
        end: usize,
        full_start: usize,
    ) -> Option<Arc<Node>> {
        let text = self.scanner.text().to_string();

        if !is_jsdoc_like_text(&text[start..]) {
            return None;
        }

        let saved_scanner = self.scanner.clone();
        let saved_token = self.token;
        let saved_diagnostics_len = self.diagnostics.len();

        self.scanner.set_range(start + 3, end - 2);

        let last_newline = text[..start].rfind('\n').map(|i| i + 1).unwrap_or(0);
        let initial_indent = start + 4 - last_newline;

        let comment = self.parse_jsdoc_comment_worker(start, end, full_start, initial_indent);

        self.scanner = saved_scanner;
        self.token = saved_token;
        self.diagnostics.truncate(saved_diagnostics_len);

        Some(comment)
    }

    fn parse_jsdoc_comment_worker(
        &mut self,
        start: usize,
        end: usize,
        full_start: usize,
        indent: usize,
    ) -> Arc<Node> {
        let mut tags: Vec<Arc<Node>> = Vec::new();
        let mut tags_pos: usize = 0;
        let mut tags_end: usize = 0;
        let mut state = JSDocState::SawAsterisk;
        let mut backtick_count: u32 = 0;
        let mut in_fenced_code_block = false;
        let mut comment_parts: Vec<Arc<Node>> = Vec::new();
        let mut comments: Vec<String> = Vec::new();
        let mut comments_pos: usize = 0;
        let mut link_end: usize = start;
        let mut margin: i32 = -1;
        let mut indent = indent;

        self.next_token_jsdoc();
        while self.parse_optional_jsdoc(SyntaxKind::WhitespaceTrivia) {}
        if self.parse_optional_jsdoc(SyntaxKind::NewLineTrivia) {
            state = JSDocState::BeginningOfLine;
            indent = 0;
        }

        loop {

            if self.token != SyntaxKind::BacktickToken && backtick_count > 0 {
                if backtick_count >= 3 {
                    in_fenced_code_block = !in_fenced_code_block;
                }
                backtick_count = 0;
            }

            match self.token {
                SyntaxKind::AtToken => {
                    if in_fenced_code_block || !self.scanner.can_follow_jsdoc_at() {
                        if in_fenced_code_block {
                            state = JSDocState::SavingBackticks;
                        } else {
                            state = JSDocState::SavingComments;
                        }
                        push_comment(
                            &mut comments,
                            &mut indent,
                            &mut margin,
                            self.scanner.token_text(),
                        );
                    } else {
                        comments = remove_trailing_whitespace(comments);
                        if comments_pos == 0 {
                            comments_pos = self.token_pos();
                        }
                        let tag = self.parse_tag(indent);
                        if tags.is_empty() {
                            tags_pos = tag.pos();
                        }
                        tags_end = tag.end();
                        tags.push(tag);
                        state = JSDocState::BeginningOfLine;
                        margin = -1;
                    }
                }
                SyntaxKind::NewLineTrivia => {
                    comments.push(self.scanner.token_text().to_string());
                    state = JSDocState::BeginningOfLine;
                    indent = 0;
                }
                SyntaxKind::AsteriskToken => {
                    let asterisk = self.scanner.token_text().to_string();
                    if state == JSDocState::SawAsterisk {
                        state = JSDocState::SavingComments;
                        push_comment(&mut comments, &mut indent, &mut margin, &asterisk);
                    } else {

                        state = JSDocState::SawAsterisk;
                        indent += asterisk.len();
                    }
                }
                SyntaxKind::WhitespaceTrivia => {
                    let whitespace = self.scanner.token_text().to_string();
                    if margin > -1 && (indent as i32 + whitespace.len() as i32) > margin {
                        let mut existing_indent = margin - indent as i32;
                        if existing_indent < 0 {
                            existing_indent += whitespace.len() as i32;
                        }
                        if existing_indent < 0 {
                            existing_indent = 0;
                        }
                        let existing_indent = existing_indent as usize;
                        if existing_indent < whitespace.len() {
                            comments.push(whitespace[existing_indent..].to_string());
                        }
                    }
                    indent += whitespace.len();
                }
                SyntaxKind::EndOfFile => break,
                SyntaxKind::JSDocCommentTextToken => {
                    if state != JSDocState::SavingBackticks {
                        if in_fenced_code_block {
                            state = JSDocState::SavingBackticks;
                        } else {
                            state = JSDocState::SavingComments;
                        }
                    }
                    push_comment(
                        &mut comments,
                        &mut indent,
                        &mut margin,
                        &self.scanner.token_value(),
                    );
                }
                SyntaxKind::BacktickToken => {
                    backtick_count += 1;
                    if state == JSDocState::SavingBackticks {
                        state = JSDocState::SavingComments;
                    } else {
                        state = JSDocState::SavingBackticks;
                    }
                    push_comment(
                        &mut comments,
                        &mut indent,
                        &mut margin,
                        self.scanner.token_text(),
                    );
                }
                SyntaxKind::OpenBraceToken => {
                    if in_fenced_code_block {
                        state = JSDocState::SavingBackticks;
                        push_comment(
                            &mut comments,
                            &mut indent,
                            &mut margin,
                            self.scanner.token_text(),
                        );
                    } else {
                        state = JSDocState::SavingComments;
                        let comment_end = self.scanner.full_start_pos();
                        let link_start = self.scanner.token_end() - 1;
                        if let Some(link) = self.parse_jsdoc_link(link_start) {
                            if link_end == start {
                                comments = remove_leading_newlines(comments);
                            }
                            let jsdoc_text =
                                self.finish_jsdoc_text(&comments, link_end, comment_end);
                            comment_parts.push(jsdoc_text);
                            comment_parts.push(link);
                            comments.clear();
                            link_end = self.scanner.token_end();
                        } else {

                            if state != JSDocState::SavingBackticks {
                                if in_fenced_code_block {
                                    state = JSDocState::SavingBackticks;
                                } else {
                                    state = JSDocState::SavingComments;
                                }
                            }
                            push_comment(
                                &mut comments,
                                &mut indent,
                                &mut margin,
                                self.scanner.token_text(),
                            );
                        }
                    }
                }
                _ => {

                    if state != JSDocState::SavingBackticks {
                        if in_fenced_code_block {
                            state = JSDocState::SavingBackticks;
                        } else {
                            state = JSDocState::SavingComments;
                        }
                    }
                    push_comment(
                        &mut comments,
                        &mut indent,
                        &mut margin,
                        self.scanner.token_text(),
                    );
                }
            }

            if state == JSDocState::SavingComments || state == JSDocState::SavingBackticks {
                self.next_jsdoc_comment_text_token(state == JSDocState::SavingBackticks);
            } else {
                self.next_token_jsdoc();
            }
        }

        if comments_pos == 0 {
            comments_pos = self.scanner.full_start_pos();
        }

        if !comments.is_empty() {
            let last_idx = comments.len() - 1;
            comments[last_idx] = trim_end(&comments[last_idx]);
            let jsdoc_text = self.finish_jsdoc_text(&comments, link_end, comments_pos);
            comment_parts.push(jsdoc_text);
        }

        let tags_node_list = if !tags.is_empty() {
            Some(Arc::new(NodeList {
                loc: TextRange::new(tags_pos, tags_end),
                nodes: tags,
            }))
        } else {
            None
        };

        let comment_list = Arc::new(NodeList {
            loc: TextRange::new(start, comments_pos),
            nodes: comment_parts,
        });

        let jsdoc = Node::with_loc(
            SyntaxKind::JSDoc,
            NodeData::JSDoc(JSDocData {
                comment: comment_list,
                tags: tags_node_list,
            }),
            TextRange::new(full_start, end),
        );
        Arc::new(jsdoc)
    }

    fn finish_jsdoc_text(&self, comments: &[String], pos: usize, end: usize) -> Arc<Node> {
        Arc::new(Node::with_loc(
            SyntaxKind::JSDocText,
            NodeData::JSDocText(JSDocTextData {
                text: comments.to_vec(),
            }),
            TextRange::new(pos, end),
        ))
    }
}

impl super::Parser {

    fn next_token_jsdoc(&mut self) -> SyntaxKind {
        self.token = self.scanner.scan_jsdoc_token();
        self.token
    }

    fn next_jsdoc_comment_text_token(&mut self, in_backticks: bool) -> SyntaxKind {
        self.token = self.scanner.scan_jsdoc_comment_text_token(in_backticks);
        self.token
    }

    fn parse_optional_jsdoc(&mut self, kind: SyntaxKind) -> bool {
        if self.token == kind {
            self.next_token_jsdoc();
            true
        } else {
            false
        }
    }

    fn parse_expected_jsdoc(&mut self, kind: SyntaxKind) {
        if !self.parse_optional_jsdoc(kind) {
            self.parse_error_at_current_token(diagnostics::X_0_EXPECTED, &[token_to_string(kind)]);
        }
    }

    fn parse_expected_token_jsdoc(&mut self, kind: SyntaxKind) -> Arc<Node> {
        if self.token == kind {
            let node = self.create_token_node_jsdoc();
            self.next_token_jsdoc();
            node
        } else {
            self.parse_error_at_current_token(diagnostics::X_0_EXPECTED, &[token_to_string(kind)]);
            self.create_missing_node(kind, self.token_pos(), self.token_pos())
        }
    }

    fn create_token_node_jsdoc(&self) -> Arc<Node> {
        Arc::new(Node::with_loc(
            self.token,
            NodeData::Token,
            TextRange::new(self.token_pos(), self.token_end()),
        ))
    }

    fn create_missing_node(&self, kind: SyntaxKind, pos: usize, end: usize) -> Arc<Node> {
        Arc::new(Node::with_loc(
            kind,
            NodeData::Token,
            TextRange::new(pos, end),
        ))
    }
}

impl super::Parser {

    fn parse_tag(&mut self, margin: usize) -> Arc<Node> {
        debug_assert_eq!(self.token, SyntaxKind::AtToken);
        let start = self.token_pos();
        self.next_token_jsdoc();
        let tag_name = self.parse_jsdoc_identifier_name(Some(diagnostics::IDENTIFIER_EXPECTED));
        let indent_text = self.skip_whitespace_or_asterisk();
        let tag_text = tag_name.text().to_string();

        let tag = match tag_text.as_str() {
            "implements" => self.parse_implements_tag(start, tag_name, margin, &indent_text),
            "augments" | "extends" => {
                self.parse_augments_tag(start, tag_name, margin, &indent_text)
            }
            "public" => self.parse_simple_tag(
                start,
                tag_name,
                margin,
                &indent_text,
                SyntaxKind::JSDocPublicTag,
            ),
            "private" => self.parse_simple_tag(
                start,
                tag_name,
                margin,
                &indent_text,
                SyntaxKind::JSDocPrivateTag,
            ),
            "protected" => self.parse_simple_tag(
                start,
                tag_name,
                margin,
                &indent_text,
                SyntaxKind::JSDocProtectedTag,
            ),
            "readonly" => self.parse_simple_tag(
                start,
                tag_name,
                margin,
                &indent_text,
                SyntaxKind::JSDocReadonlyTag,
            ),
            "override" => self.parse_simple_tag(
                start,
                tag_name,
                margin,
                &indent_text,
                SyntaxKind::JSDocOverrideTag,
            ),
            "deprecated" => self.parse_deprecated_tag(start, tag_name, margin, &indent_text),
            "this" => self.parse_this_tag(start, tag_name, margin, &indent_text),
            "arg" | "argument" | "param" => self.parse_parameter_or_property_tag(
                start,
                tag_name,
                PropertyLikeParse(PropertyLikeParse::PARAMETER),
                margin,
            ),
            "return" | "returns" => self.parse_return_tag(start, tag_name, margin, &indent_text),
            "template" => self.parse_template_tag(start, tag_name, margin, &indent_text),
            "type" => self.parse_type_tag(start, tag_name, margin, &indent_text),
            "typedef" => self.parse_typedef_tag(start, tag_name, margin, &indent_text),
            "callback" => self.parse_callback_tag(start, tag_name, margin, &indent_text),
            "overload" => self.parse_overload_tag(start, tag_name, margin, &indent_text),
            "satisfies" => self.parse_satisfies_tag(start, tag_name, margin, &indent_text),
            "see" => self.parse_see_tag(start, tag_name, margin, &indent_text),
            "exception" | "throws" => self.parse_throws_tag(start, tag_name, margin, &indent_text),
            "import" => self.parse_import_tag(start, tag_name, margin, &indent_text),
            _ => self.parse_unknown_tag(start, tag_name, margin, &indent_text),
        };
        tag
    }

    fn parse_simple_tag(
        &mut self,
        start: usize,
        tag_name: Arc<Node>,
        margin: usize,
        indent_text: &str,
        kind: SyntaxKind,
    ) -> Arc<Node> {
        let comment = self.parse_trailing_tag_comments(
            self.token_pos(),
            self.token_end(),
            margin,
            indent_text,
        );
        let end = comment.end();
        let data = match kind {
            SyntaxKind::JSDocPublicTag => NodeData::JSDocPublicTag(JSDocPublicTagData {
                tag_name,
                comment: Some(comment),
            }),
            SyntaxKind::JSDocPrivateTag => NodeData::JSDocPrivateTag(JSDocPrivateTagData {
                tag_name,
                comment: Some(comment),
            }),
            SyntaxKind::JSDocProtectedTag => NodeData::JSDocProtectedTag(JSDocProtectedTagData {
                tag_name,
                comment: Some(comment),
            }),
            SyntaxKind::JSDocReadonlyTag => NodeData::JSDocReadonlyTag(JSDocReadonlyTagData {
                tag_name,
                comment: Some(comment),
            }),
            SyntaxKind::JSDocOverrideTag => NodeData::JSDocOverrideTag(JSDocOverrideTagData {
                tag_name,
                comment: Some(comment),
            }),
            _ => unreachable!(),
        };
        Arc::new(Node::with_loc(kind, data, TextRange::new(start, end)))
    }

    fn parse_deprecated_tag(
        &mut self,
        start: usize,
        tag_name: Arc<Node>,
        margin: usize,
        indent_text: &str,
    ) -> Arc<Node> {
        let comment = self.parse_trailing_tag_comments(
            self.token_pos(),
            self.token_end(),
            margin,
            indent_text,
        );
        let end = comment.end();
        Arc::new(Node::with_loc(
            SyntaxKind::JSDocDeprecatedTag,
            NodeData::JSDocDeprecatedTag(JSDocDeprecatedTagData {
                tag_name,
                comment: Some(comment),
            }),
            TextRange::new(start, end),
        ))
    }

    fn parse_unknown_tag(
        &mut self,
        start: usize,
        tag_name: Arc<Node>,
        margin: usize,
        indent_text: &str,
    ) -> Arc<Node> {
        let comment = self.parse_trailing_tag_comments(
            self.token_pos(),
            self.token_end(),
            margin,
            indent_text,
        );
        let end = comment.end();
        Arc::new(Node::with_loc(
            SyntaxKind::JSDocUnknownTag,
            NodeData::JSDocUnknownTag(JSDocUnknownTagData {
                tag_name,
                comment: Some(comment),
            }),
            TextRange::new(start, end),
        ))
    }

    fn parse_return_tag(
        &mut self,
        start: usize,
        tag_name: Arc<Node>,
        margin: usize,
        indent_text: &str,
    ) -> Arc<Node> {
        let type_expression = self.try_parse_type_expression();
        let comment = self.parse_trailing_tag_comments(
            self.token_pos(),
            self.token_end(),
            margin,
            indent_text,
        );
        let end = comment
            .end()
            .max(type_expression.as_ref().map(|t| t.end()).unwrap_or(0));
        Arc::new(Node::with_loc(
            SyntaxKind::JSDocReturnTag,
            NodeData::JSDocReturnTag(JSDocReturnTagData {
                tag_name,
                type_expression,
                comment: Some(comment),
            }),
            TextRange::new(start, end),
        ))
    }

    fn parse_type_tag(
        &mut self,
        start: usize,
        tag_name: Arc<Node>,
        margin: usize,
        indent_text: &str,
    ) -> Arc<Node> {
        let type_expression = self.parse_jsdoc_type_expression(true);
        let comment = if margin != usize::MAX {
            Some(self.parse_trailing_tag_comments(
                self.token_pos(),
                self.token_end(),
                margin,
                indent_text,
            ))
        } else {
            None
        };
        let end = comment
            .as_ref()
            .map(|c| c.end())
            .max(Some(type_expression.end()))
            .unwrap_or(type_expression.end());
        Arc::new(Node::with_loc(
            SyntaxKind::JSDocTypeTag,
            NodeData::JSDocTypeTag(JSDocTypeTagData {
                tag_name,
                type_expression,
                comment,
            }),
            TextRange::new(start, end),
        ))
    }

    fn parse_this_tag(
        &mut self,
        start: usize,
        tag_name: Arc<Node>,
        margin: usize,
        indent_text: &str,
    ) -> Arc<Node> {
        let type_expression = self.parse_jsdoc_type_expression(true);
        self.skip_whitespace();
        let comment = self.parse_trailing_tag_comments(
            self.token_pos(),
            self.token_end(),
            margin,
            indent_text,
        );
        let end = comment.end().max(type_expression.end());
        Arc::new(Node::with_loc(
            SyntaxKind::JSDocThisTag,
            NodeData::JSDocThisTag(JSDocThisTagData {
                tag_name,
                type_expression,
                comment: Some(comment),
            }),
            TextRange::new(start, end),
        ))
    }

    fn parse_satisfies_tag(
        &mut self,
        start: usize,
        tag_name: Arc<Node>,
        margin: usize,
        indent_text: &str,
    ) -> Arc<Node> {
        let type_expression = self.parse_jsdoc_type_expression(false);
        let comment = self.parse_trailing_tag_comments(
            self.token_pos(),
            self.token_end(),
            margin,
            indent_text,
        );
        let end = comment.end().max(type_expression.end());
        Arc::new(Node::with_loc(
            SyntaxKind::JSDocSatisfiesTag,
            NodeData::JSDocSatisfiesTag(JSDocSatisfiesTagData {
                tag_name,
                type_expression,
                comment: Some(comment),
            }),
            TextRange::new(start, end),
        ))
    }

    fn parse_throws_tag(
        &mut self,
        start: usize,
        tag_name: Arc<Node>,
        margin: usize,
        indent_text: &str,
    ) -> Arc<Node> {
        let type_expression = self.try_parse_type_expression();
        let comment = self.parse_trailing_tag_comments(
            self.token_pos(),
            self.token_end(),
            margin,
            indent_text,
        );
        let end = comment
            .end()
            .max(type_expression.as_ref().map(|t| t.end()).unwrap_or(0));
        Arc::new(Node::with_loc(
            SyntaxKind::JSDocThrowsTag,
            NodeData::JSDocThrowsTag(JSDocThrowsTagData {
                tag_name,
                type_expression,
                comment: Some(comment),
            }),
            TextRange::new(start, end),
        ))
    }

    fn parse_see_tag(
        &mut self,
        start: usize,
        tag_name: Arc<Node>,
        margin: usize,
        indent_text: &str,
    ) -> Arc<Node> {
        let name_expression = if self.is_identifier()
            || (self.token == SyntaxKind::OpenBraceToken && {

                let mut sc = self.scanner.clone();
                sc.scan_jsdoc_token();
                let is_id = matches!(
                    sc.token(),
                    SyntaxKind::Identifier
                        | SyntaxKind::ThisKeyword
                        | SyntaxKind::TrueKeyword
                        | SyntaxKind::FalseKeyword
                );
                is_id
            }) {
            Some(self.parse_jsdoc_name_reference())
        } else {
            None
        };
        let comment = self.parse_trailing_tag_comments(
            self.token_pos(),
            self.token_end(),
            margin,
            indent_text,
        );
        let end = comment
            .end()
            .max(name_expression.as_ref().map(|n| n.end()).unwrap_or(0));
        let name = name_expression.unwrap_or_else(|| {
            self.create_missing_node(SyntaxKind::Identifier, self.token_pos(), self.token_pos())
        });
        Arc::new(Node::with_loc(
            SyntaxKind::JSDocSeeTag,
            NodeData::JSDocSeeTag(JSDocSeeTagData {
                tag_name,
                name_expression: name,
                comment: Some(comment),
            }),
            TextRange::new(start, end),
        ))
    }

    fn parse_implements_tag(
        &mut self,
        start: usize,
        tag_name: Arc<Node>,
        margin: usize,
        indent_text: &str,
    ) -> Arc<Node> {
        let class_name = self.parse_expression_with_type_arguments_for_augments();
        let comment = self.parse_trailing_tag_comments(
            self.token_pos(),
            self.token_end(),
            margin,
            indent_text,
        );
        let end = comment.end().max(class_name.end());
        Arc::new(Node::with_loc(
            SyntaxKind::JSDocImplementsTag,
            NodeData::JSDocImplementsTag(JSDocImplementsTagData {
                tag_name,
                class_name,
                comment: Some(comment),
            }),
            TextRange::new(start, end),
        ))
    }

    fn parse_augments_tag(
        &mut self,
        start: usize,
        tag_name: Arc<Node>,
        margin: usize,
        indent_text: &str,
    ) -> Arc<Node> {
        let class_name = self.parse_expression_with_type_arguments_for_augments();
        let comment = self.parse_trailing_tag_comments(
            self.token_pos(),
            self.token_end(),
            margin,
            indent_text,
        );
        let end = comment.end().max(class_name.end());
        Arc::new(Node::with_loc(
            SyntaxKind::JSDocAugmentsTag,
            NodeData::JSDocAugmentsTag(JSDocAugmentsTagData {
                tag_name,
                class_name,
                comment: Some(comment),
            }),
            TextRange::new(start, end),
        ))
    }

    fn parse_parameter_or_property_tag(
        &mut self,
        start: usize,
        tag_name: Arc<Node>,
        target: PropertyLikeParse,
        indent: usize,
    ) -> Arc<Node> {
        let type_expression = self.try_parse_type_expression();
        let is_name_first = type_expression.is_none();
        self.skip_whitespace_or_asterisk();
        let (name, is_bracketed) = self.parse_bracket_name_in_property_and_param_tag(target);
        let indent_text = self.skip_whitespace_or_asterisk();

        let type_expression = if is_name_first && type_expression.is_none() {

            let _ = self.parse_jsdoc_link_prefix();
            self.try_parse_type_expression()
        } else {
            type_expression
        };

        let comment = self.parse_trailing_tag_comments(
            self.token_pos(),
            self.token_end(),
            indent,
            &indent_text,
        );

        let end = comment
            .end()
            .max(type_expression.as_ref().map(|t| t.end()).unwrap_or(0))
            .max(name.end());

        let kind = if target.contains(PropertyLikeParse::PARAMETER) {
            SyntaxKind::JSDocParameterTag
        } else {
            SyntaxKind::JSDocPropertyTag
        };

        Arc::new(Node::with_loc(
            kind,
            NodeData::JSDocParameterOrPropertyTag(JSDocParameterOrPropertyTagData {
                tag_name,
                name,
                is_bracketed,
                type_expression,
                is_name_first,
                comment: Some(comment),
            }),
            TextRange::new(start, end),
        ))
    }

    fn parse_template_tag(
        &mut self,
        start: usize,
        tag_name: Arc<Node>,
        margin: usize,
        indent_text: &str,
    ) -> Arc<Node> {
        let constraint = if self.token == SyntaxKind::OpenBraceToken {
            Some(self.parse_jsdoc_type_expression(false))
        } else {
            None
        };
        let type_parameters = self.parse_template_tag_type_parameters();
        let comment = self.parse_trailing_tag_comments(
            self.token_pos(),
            self.token_end(),
            margin,
            indent_text,
        );
        let end = comment.end().max(type_parameters.end());
        let constraint_node = constraint.unwrap_or_else(|| {
            self.create_missing_node(SyntaxKind::MissingDeclaration, start, start)
        });
        Arc::new(Node::with_loc(
            SyntaxKind::JSDocTemplateTag,
            NodeData::JSDocTemplateTag(JSDocTemplateTagData {
                tag_name,
                constraint: constraint_node,
                type_parameters,
                comment: Some(comment),
            }),
            TextRange::new(start, end),
        ))
    }

    fn parse_template_tag_type_parameters(&mut self) -> Arc<NodeList> {
        let pos = self.token_pos();
        let mut params = Vec::new();
        loop {
            params.push(self.parse_template_tag_type_parameter());
            self.skip_whitespace_or_asterisk();
            if !self.parse_optional_jsdoc(SyntaxKind::CommaToken) {
                break;
            }
        }
        let end = params.last().map(|p| p.end()).unwrap_or(pos);
        Arc::new(NodeList {
            loc: TextRange::new(pos, end),
            nodes: params,
        })
    }

    fn parse_template_tag_type_parameter(&mut self) -> Arc<Node> {
        let pos = self.token_pos();

        let modifiers = if self.token == SyntaxKind::ConstKeyword {
            let mod_node = self.create_token_node_jsdoc();
            self.next_token_jsdoc();
            Some(Arc::new(ModifierList::new(
                vec![mod_node],
                crate::ast::ModifierFlags::Const,
            )))
        } else {
            None
        };

        let is_bracketed = self.parse_optional_jsdoc(SyntaxKind::OpenBracketToken);
        let name = self.parse_jsdoc_identifier_name(Some(diagnostics::IDENTIFIER_EXPECTED));
        let default_type = if is_bracketed {
            self.skip_whitespace();
            let default = if self.parse_optional_jsdoc(SyntaxKind::EqualsToken) {
                Some(self.parse_type())
            } else {
                None
            };
            self.parse_expected_token_jsdoc(SyntaxKind::CloseBracketToken);
            default
        } else {
            None
        };

        let end = name.end();
        Arc::new(Node::with_loc(
            SyntaxKind::TypeParameter,
            NodeData::TypeParameterDeclaration(TypeParameterDeclarationData {
                modifiers,
                name,
                constraint: None,
                expression: None,
                default_type,
            }),
            TextRange::new(pos, end),
        ))
    }

    fn parse_typedef_tag(
        &mut self,
        start: usize,
        tag_name: Arc<Node>,
        margin: usize,
        _indent_text: &str,
    ) -> Arc<Node> {
        let type_expression = self.try_parse_type_expression();
        self.skip_whitespace_or_asterisk();
        let full_name = self.parse_jsdoc_type_name_with_namespace(false);
        let name = full_name.unwrap_or_else(|| {
            self.parse_jsdoc_identifier_name(Some(diagnostics::IDENTIFIER_EXPECTED))
        });
        self.skip_whitespace();
        let comment = self.parse_tag_comments(margin, None);
        let end = comment.end().max(name.end());
        Arc::new(Node::with_loc(
            SyntaxKind::JSDocTypedefTag,
            NodeData::JSDocTypedefTag(JSDocTypedefTagData {
                tag_name,
                type_expression,
                name: Some(name),
                comment: Some(comment),
            }),
            TextRange::new(start, end),
        ))
    }

    fn parse_callback_tag(
        &mut self,
        start: usize,
        tag_name: Arc<Node>,
        margin: usize,
        _indent_text: &str,
    ) -> Arc<Node> {
        let full_name = self.parse_jsdoc_type_name_with_namespace(false);
        let name = full_name.unwrap_or_else(|| {
            self.parse_jsdoc_identifier_name(Some(diagnostics::IDENTIFIER_EXPECTED))
        });
        self.skip_whitespace();
        let comment = self.parse_tag_comments(margin, None);
        let type_expression = self.parse_jsdoc_signature(start, margin);
        let end = type_expression.end().max(comment.end()).max(name.end());
        Arc::new(Node::with_loc(
            SyntaxKind::JSDocCallbackTag,
            NodeData::JSDocCallbackTag(JSDocCallbackTagData {
                tag_name,
                type_expression,
                name: Some(name),
                comment: Some(comment),
            }),
            TextRange::new(start, end),
        ))
    }

    fn parse_overload_tag(
        &mut self,
        start: usize,
        tag_name: Arc<Node>,
        margin: usize,
        _indent_text: &str,
    ) -> Arc<Node> {
        self.skip_whitespace();
        let comment = self.parse_tag_comments(margin, None);
        let type_expression = self.parse_jsdoc_signature(start, margin);
        let end = type_expression.end().max(comment.end());
        Arc::new(Node::with_loc(
            SyntaxKind::JSDocOverloadTag,
            NodeData::JSDocOverloadTag(JSDocOverloadTagData {
                tag_name,
                type_expression,
                comment: Some(comment),
            }),
            TextRange::new(start, end),
        ))
    }

    fn parse_jsdoc_signature(&mut self, start: usize, indent: usize) -> Arc<Node> {
        let parameters = self.parse_callback_tag_parameters(indent);
        let return_tag = if self.parse_optional_jsdoc(SyntaxKind::AtToken) {
            let tag = self.parse_tag(indent);
            if tag.kind == SyntaxKind::JSDocReturnTag {
                Some(tag)
            } else {
                None
            }
        } else {
            None
        };
        let end = return_tag
            .as_ref()
            .map(|t| t.end())
            .unwrap_or_else(|| parameters.end());
        Arc::new(Node::with_loc(
            SyntaxKind::JSDocSignature,
            NodeData::JSDocSignature(JSDocSignatureData {
                type_parameters: None,
                parameters,
                type_node: return_tag,
            }),
            TextRange::new(start, end),
        ))
    }

    fn parse_callback_tag_parameters(&mut self, indent: usize) -> Arc<NodeList> {
        let pos = self.token_pos();
        let mut params = Vec::new();
        loop {
            if self.token == SyntaxKind::EndOfFile {
                break;
            }
            if self.token == SyntaxKind::AtToken {
                if let Some(child) = self.parse_child_parameter_or_property_tag(
                    PropertyLikeParse(PropertyLikeParse::CALLBACK_PARAMETER),
                    indent,
                    None,
                ) {
                    if child.kind == SyntaxKind::JSDocParameterTag {
                        params.push(child);
                    }
                }
            } else {
                self.next_token_jsdoc();
            }
        }
        let end = params.last().map(|p| p.end()).unwrap_or(pos);
        Arc::new(NodeList {
            loc: TextRange::new(pos, end),
            nodes: params,
        })
    }

    fn parse_import_tag(
        &mut self,
        start: usize,
        tag_name: Arc<Node>,
        margin: usize,
        indent_text: &str,
    ) -> Arc<Node> {

        let comment = self.parse_trailing_tag_comments(
            self.token_pos(),
            self.token_end(),
            margin,
            indent_text,
        );
        let end = comment.end();
        let module_specifier = self.create_missing_node(
            SyntaxKind::StringLiteral,
            self.token_pos(),
            self.token_pos(),
        );
        Arc::new(Node::with_loc(
            SyntaxKind::JSDocImportTag,
            NodeData::JSDocImportTag(JSDocImportTagData {
                tag_name,
                import_clause: None,
                module_specifier,
                attributes: None,
                comment: Some(comment),
            }),
            TextRange::new(start, end),
        ))
    }
}

impl super::Parser {

    fn parse_jsdoc_type_expression(&mut self, may_omit_braces: bool) -> Arc<Node> {
        let pos = self.token_pos();
        let has_brace = if may_omit_braces {
            self.parse_optional(SyntaxKind::OpenBraceToken)
        } else {

            if self.token == SyntaxKind::OpenBraceToken {
                self.next_token();
                true
            } else {
                self.parse_error_at_current_token(
                    diagnostics::X_0_EXPECTED,
                    &[token_to_string(SyntaxKind::OpenBraceToken)],
                );
                false
            }
        };

        let t = self.parse_jsdoc_type();

        if has_brace {
            self.parse_expected_jsdoc(SyntaxKind::CloseBraceToken);
        }

        Arc::new(Node::with_loc(
            SyntaxKind::JSDocTypeExpression,
            NodeData::JSDocTypeExpression(JSDocTypeExpressionData { type_node: t }),
            TextRange::new(pos, self.token_pos()),
        ))
    }

    fn parse_jsdoc_type(&mut self) -> Arc<Node> {
        let pos = self.token_pos();
        let t = match self.token {
            SyntaxKind::AsteriskToken => {
                self.next_token_jsdoc();
                Arc::new(Node::with_loc(
                    SyntaxKind::JSDocAllType,
                    NodeData::JSDocAllType,
                    TextRange::new(pos, pos + 1),
                ))
            }
            SyntaxKind::QuestionToken => {
                self.next_token_jsdoc();
                let inner = self.parse_type();
                let end = inner.end();
                Arc::new(Node::with_loc(
                    SyntaxKind::JSDocNullableType,
                    NodeData::JSDocNullableType(JSDocNullableTypeData { type_node: inner }),
                    TextRange::new(pos, end),
                ))
            }
            SyntaxKind::ExclamationToken => {
                self.next_token_jsdoc();
                let inner = self.parse_type();
                let end = inner.end();
                Arc::new(Node::with_loc(
                    SyntaxKind::JSDocNonNullableType,
                    NodeData::JSDocNonNullableType(JSDocNonNullableTypeData { type_node: inner }),
                    TextRange::new(pos, end),
                ))
            }
            SyntaxKind::DotDotDotToken => {
                self.next_token_jsdoc();
                let inner = self.parse_type();
                let end = inner.end();
                Arc::new(Node::with_loc(
                    SyntaxKind::JSDocVariadicType,
                    NodeData::JSDocVariadicType(JSDocVariadicTypeData { type_node: inner }),
                    TextRange::new(pos, end),
                ))
            }
            _ => self.parse_type(),
        };

        if self.token == SyntaxKind::EqualsToken {
            self.next_token_jsdoc();
            let end = self.token_pos();
            Arc::new(Node::with_loc(
                SyntaxKind::JSDocOptionalType,
                NodeData::JSDocOptionalType(JSDocOptionalTypeData { type_node: t }),
                TextRange::new(pos, end),
            ))
        } else {
            t
        }
    }

    fn try_parse_type_expression(&mut self) -> Option<Arc<Node>> {
        self.skip_whitespace_or_asterisk();
        if self.token == SyntaxKind::OpenBraceToken {
            Some(self.parse_jsdoc_type_expression(false))
        } else {
            None
        }
    }

    fn parse_type_arguments_of_type_node(&mut self) -> Arc<NodeList> {
        let pos = self.token_pos();
        self.parse_expected_jsdoc(SyntaxKind::LessThanToken);
        let mut types: Vec<Arc<Node>> = Vec::new();
        loop {
            self.skip_whitespace();
            types.push(self.parse_jsdoc_type());
            self.skip_whitespace();
            if !self.parse_optional_jsdoc(SyntaxKind::CommaToken) {
                break;
            }
        }
        self.parse_expected_jsdoc(SyntaxKind::GreaterThanToken);
        let end = self.token_pos();
        Arc::new(NodeList {
            loc: TextRange::new(pos, end),
            nodes: types,
        })
    }

    fn parse_expression_with_type_arguments_for_augments(&mut self) -> Arc<Node> {
        let pos = self.token_pos();
        let has_brace = self.parse_optional_jsdoc(SyntaxKind::OpenBraceToken);
        let expression = self.parse_property_access_entity_name_expression();

        let saved_skip = self.scanner.skip_jsdoc_leading_asterisks_raw();
        self.scanner.set_skip_jsdoc_leading_asterisks(true);

        let type_arguments = if self.token == SyntaxKind::LessThanToken {
            Some(self.parse_type_arguments_of_type_node())
        } else {
            None
        };

        self.scanner
            .set_skip_jsdoc_leading_asterisks_raw(saved_skip);

        let end = if has_brace {
            self.skip_whitespace();
            self.parse_expected_jsdoc(SyntaxKind::CloseBraceToken);
            self.token_pos()
        } else {
            type_arguments
                .as_ref()
                .map(|ta| ta.end())
                .unwrap_or(expression.end())
        };

        Arc::new(Node::with_loc(
            SyntaxKind::ExpressionWithTypeArguments,
            NodeData::ExpressionWithTypeArguments(ExpressionWithTypeArgumentsData {
                expression,
                type_arguments,
            }),
            TextRange::new(pos, end),
        ))
    }

    fn parse_property_access_entity_name_expression(&mut self) -> Arc<Node> {
        let mut node = self.parse_jsdoc_identifier_name(None);
        while self.parse_optional_jsdoc(SyntaxKind::DotToken) {
            let name = self.parse_jsdoc_identifier_name(None);
            let end = name.end();
            let node_pos = node.pos();
            node = Arc::new(Node::with_loc(
                SyntaxKind::PropertyAccessExpression,
                NodeData::PropertyAccessExpression(PropertyAccessExpressionData {
                    expression: node,
                    question_dot_token: None,
                    name,
                }),
                TextRange::new(node_pos, end),
            ));
        }
        node
    }
}

impl super::Parser {

    fn parse_jsdoc_identifier_name(&mut self, diagnostic: Option<Message>) -> Arc<Node> {
        if !self.is_identifier() {
            if let Some(msg) = diagnostic {
                self.parse_error_at_current_token(msg, &[]);
            }

            return Arc::new(Node::with_loc(
                SyntaxKind::Identifier,
                NodeData::Identifier(IdentifierData {
                    text: String::new(),
                }),
                TextRange::new(self.token_pos(), self.token_pos()),
            ));
        }
        let text = self.scanner.token_text().to_string();
        let pos = self.token_pos();
        let end = self.token_end();
        self.next_token_jsdoc();
        Arc::new(Node::with_loc(
            SyntaxKind::Identifier,
            NodeData::Identifier(IdentifierData { text }),
            TextRange::new(pos, end),
        ))
    }

    fn parse_jsdoc_entity_name(&mut self, diagnostic: Option<Message>) -> Arc<Node> {
        let mut node = self.parse_jsdoc_identifier_name(diagnostic);
        while self.parse_optional_jsdoc(SyntaxKind::DotToken) {
            let right = self.parse_jsdoc_identifier_name(diagnostic);
            let end = right.end();
            let node_pos = node.pos();
            node = Arc::new(Node::with_loc(
                SyntaxKind::QualifiedName,
                NodeData::QualifiedName(QualifiedNameData { left: node, right }),
                TextRange::new(node_pos, end),
            ));

            self.parse_optional_jsdoc(SyntaxKind::OpenBracketToken);
            self.parse_optional_jsdoc(SyntaxKind::CloseBracketToken);
        }
        node
    }

    fn parse_jsdoc_name_reference(&mut self) -> Arc<Node> {
        let pos = self.token_pos();
        let has_brace = self.parse_optional_jsdoc(SyntaxKind::OpenBraceToken);
        let entity_name = self.parse_jsdoc_link_name();
        if has_brace {
            self.parse_expected_jsdoc(SyntaxKind::CloseBraceToken);
        }

        self.scanner
            .set_range(self.scanner.full_start_pos(), self.scanner.end());
        self.next_token_jsdoc();
        let end = self.token_pos();
        Arc::new(Node::with_loc(
            SyntaxKind::JSDocNameReference,
            NodeData::JSDocNameReference(JSDocNameReferenceData { name: entity_name }),
            TextRange::new(pos, end),
        ))
    }

    fn parse_jsdoc_link_name(&mut self) -> Arc<Node> {
        if !is_identifier_or_keyword_token(self.token) {
            return self.create_missing_node(
                SyntaxKind::Identifier,
                self.token_pos(),
                self.token_pos(),
            );
        }
        let mut node = self.parse_jsdoc_identifier_name(None);
        loop {
            if self.parse_optional_jsdoc(SyntaxKind::DotToken) {
                let right = if is_identifier_or_keyword_token(self.token) {
                    self.parse_jsdoc_identifier_name(None)
                } else {
                    self.create_missing_node(
                        SyntaxKind::Identifier,
                        self.token_pos(),
                        self.token_pos(),
                    )
                };
                let end = right.end();
                let node_pos = node.pos();
                node = Arc::new(Node::with_loc(
                    SyntaxKind::QualifiedName,
                    NodeData::QualifiedName(QualifiedNameData { left: node, right }),
                    TextRange::new(node_pos, end),
                ));
            } else if self.token == SyntaxKind::PrivateIdentifier {

                let text = self.scanner.token_text().to_string();
                let pos = self.token_pos();
                let end = self.token_end();
                self.next_token_jsdoc();
                let right = Arc::new(Node::with_loc(
                    SyntaxKind::PrivateIdentifier,
                    NodeData::PrivateIdentifier(PrivateIdentifierData { text }),
                    TextRange::new(pos, end),
                ));
                let end = right.end();
                let node_pos = node.pos();
                node = Arc::new(Node::with_loc(
                    SyntaxKind::QualifiedName,
                    NodeData::QualifiedName(QualifiedNameData { left: node, right }),
                    TextRange::new(node_pos, end),
                ));
            } else {
                break;
            }
        }
        node
    }

    fn parse_jsdoc_type_name_with_namespace(&mut self, nested: bool) -> Option<Arc<Node>> {
        if !is_identifier_or_keyword_token(self.token) {
            return None;
        }
        let pos = self.token_pos();
        let name = self.parse_jsdoc_identifier_name(None);
        let mut node = name;
        if self.parse_optional_jsdoc(SyntaxKind::DotToken) {
            if let Some(inner) = self.parse_jsdoc_type_name_with_namespace(true) {
                let end = inner.end();
                node = Arc::new(Node::with_loc(
                    SyntaxKind::ModuleDeclaration,
                    NodeData::ModuleDeclaration(ModuleDeclarationData {
                        modifiers: None,
                        keyword: SyntaxKind::NamespaceKeyword,
                        name: node,
                        body: Some(inner),
                    }),
                    TextRange::new(pos, end),
                ));
            }
        }

        let _ = nested;
        Some(node)
    }

    fn parse_bracket_name_in_property_and_param_tag(
        &mut self,
        target: PropertyLikeParse,
    ) -> (Arc<Node>, bool) {
        let is_bracketed = self.parse_optional_jsdoc(SyntaxKind::OpenBracketToken);
        if is_bracketed {
            self.skip_whitespace();
        }

        let backquoted = self.parse_optional_jsdoc(SyntaxKind::BacktickToken);

        let diagnostic = if target.contains(PropertyLikeParse::PARAMETER) {
            None
        } else {
            Some(diagnostics::IDENTIFIER_EXPECTED)
        };
        let name = self.parse_jsdoc_entity_name(diagnostic);

        if backquoted {
            self.parse_expected_token_jsdoc(SyntaxKind::BacktickToken);
        }

        if is_bracketed {
            self.skip_whitespace();

            if self.parse_optional_jsdoc(SyntaxKind::EqualsToken) {
                let _default = self.parse_type();
            }
            let _close = self.parse_expected_token_jsdoc(SyntaxKind::CloseBracketToken);
        }

        (name, is_bracketed)
    }
}

impl super::Parser {

    fn parse_jsdoc_link(&mut self, start: usize) -> Option<Arc<Node>> {
        let saved_scanner = self.scanner.clone();
        let saved_token = self.token;

        let (link_kind, is_link) = self.parse_jsdoc_link_prefix();
        if !is_link {

            self.scanner = saved_scanner;
            self.token = saved_token;
            return None;
        }

        self.next_token_jsdoc();
        self.skip_whitespace();

        let name = if is_identifier_or_keyword_token(self.token) {
            Some(self.parse_jsdoc_link_name())
        } else {
            None
        };

        let mut text_parts: Vec<String> = Vec::new();
        loop {
            match self.token {
                SyntaxKind::CloseBraceToken | SyntaxKind::NewLineTrivia | SyntaxKind::EndOfFile => {
                    break;
                }
                SyntaxKind::WhitespaceTrivia => {
                    text_parts.push(self.scanner.token_text().to_string());
                    self.next_token_jsdoc();
                }
                _ => {
                    text_parts.push(self.scanner.token_text().to_string());
                    self.next_jsdoc_comment_text_token(false);
                }
            }
        }

        if let Some(last) = text_parts.last_mut() {
            *last = trim_end(last);
        }

        let end = self.token_end();
        let (kind, data) = match link_kind.as_str() {
            "linkcode" => (
                SyntaxKind::JSDocLinkCode,
                NodeData::JSDocLinkCode(JSDocLinkCodeData {
                    name,
                    text: text_parts,
                }),
            ),
            "linkplain" => (
                SyntaxKind::JSDocLinkPlain,
                NodeData::JSDocLinkPlain(JSDocLinkPlainData {
                    name,
                    text: text_parts,
                }),
            ),
            _ => (
                SyntaxKind::JSDocLink,
                NodeData::JSDocLink(JSDocLinkData {
                    name,
                    text: text_parts,
                }),
            ),
        };

        Some(Arc::new(Node::with_loc(
            kind,
            data,
            TextRange::new(start, end),
        )))
    }

    fn parse_jsdoc_link_prefix(&mut self) -> (String, bool) {
        self.skip_whitespace_or_asterisk();
        if self.token != SyntaxKind::OpenBraceToken {
            return ("NONE".to_string(), false);
        }
        let mut sc = self.scanner.clone();
        sc.scan_jsdoc_token();
        if sc.token() != SyntaxKind::AtToken {
            return ("NONE".to_string(), false);
        }
        sc.scan_jsdoc_token();
        if !is_identifier_or_keyword_token(sc.token()) {
            return ("NONE".to_string(), false);
        }
        let kind = sc.token_text().to_string();
        if is_jsdoc_link_tag(&kind) {
            (kind, true)
        } else {
            ("NONE".to_string(), false)
        }
    }
}

impl super::Parser {

    fn parse_trailing_tag_comments(
        &mut self,
        pos: usize,
        end: usize,
        margin: usize,
        indent_text: &str,
    ) -> Arc<NodeList> {
        let margin = if indent_text.is_empty() {
            margin + end.saturating_sub(pos)
        } else {
            margin
        };
        let initial_margin = if margin < indent_text.len() {
            Some(&indent_text[margin..])
        } else {
            None
        };
        self.parse_tag_comments(margin, initial_margin)
    }

    fn parse_tag_comments(&mut self, indent: usize, initial_margin: Option<&str>) -> Arc<NodeList> {
        let pos = self.token_pos();
        let mut state = JSDocState::BeginningOfLine;
        let mut comments: Vec<String> = Vec::new();
        let mut parts: Vec<Arc<Node>> = Vec::new();
        let mut margin: i32 = -1;
        let mut indent = indent;
        let mut link_end = pos;

        if let Some(m) = initial_margin {
            if !m.is_empty() {
                push_comment(&mut comments, &mut indent, &mut margin, m);
                state = JSDocState::SawAsterisk;
            }
        }

        loop {
            match self.token {
                SyntaxKind::AtToken => {
                    if self.scanner.can_follow_jsdoc_at() {

                        self.scanner
                            .set_range(self.scanner.token_end() - 1, self.scanner.end());
                        break;
                    }
                    state = JSDocState::SavingComments;
                    push_comment(
                        &mut comments,
                        &mut indent,
                        &mut margin,
                        self.scanner.token_text(),
                    );
                }
                SyntaxKind::NewLineTrivia => {
                    comments.push(self.scanner.token_text().to_string());
                    state = JSDocState::BeginningOfLine;
                    indent = 0;
                }
                SyntaxKind::AsteriskToken => {
                    let asterisk = self.scanner.token_text().to_string();
                    if state == JSDocState::SawAsterisk {
                        state = JSDocState::SavingComments;
                        push_comment(&mut comments, &mut indent, &mut margin, &asterisk);
                    } else {
                        state = JSDocState::SawAsterisk;
                        indent += asterisk.len();
                    }
                }
                SyntaxKind::WhitespaceTrivia => {
                    let whitespace = self.scanner.token_text().to_string();
                    if margin > -1 && (indent as i32 + whitespace.len() as i32) > margin {
                        let mut existing = margin - indent as i32;
                        if existing < 0 {
                            existing += whitespace.len() as i32;
                        }
                        if existing < 0 {
                            existing = 0;
                        }
                        let existing = existing as usize;
                        if existing < whitespace.len() {
                            comments.push(whitespace[existing..].to_string());
                        }
                    }
                    indent += whitespace.len();
                }
                SyntaxKind::EndOfFile => break,
                SyntaxKind::JSDocCommentTextToken => {
                    if state != JSDocState::SavingBackticks {
                        state = JSDocState::SavingComments;
                    }
                    push_comment(
                        &mut comments,
                        &mut indent,
                        &mut margin,
                        &self.scanner.token_value(),
                    );
                }
                SyntaxKind::BacktickToken => {
                    if state == JSDocState::SavingBackticks {
                        state = JSDocState::SavingComments;
                    } else {
                        state = JSDocState::SavingBackticks;
                    }
                    push_comment(
                        &mut comments,
                        &mut indent,
                        &mut margin,
                        self.scanner.token_text(),
                    );
                }
                SyntaxKind::OpenBraceToken => {
                    state = JSDocState::SavingComments;
                    let comment_end = self.scanner.full_start_pos();
                    let link_start = self.scanner.token_end() - 1;
                    if let Some(link) = self.parse_jsdoc_link(link_start) {
                        if link_end == pos {
                            comments = remove_leading_newlines(comments);
                        }
                        let jsdoc_text = self.finish_jsdoc_text(&comments, link_end, comment_end);
                        parts.push(jsdoc_text);
                        parts.push(link);
                        comments.clear();
                        link_end = self.scanner.token_end();
                    } else {
                        push_comment(
                            &mut comments,
                            &mut indent,
                            &mut margin,
                            self.scanner.token_text(),
                        );
                    }
                }
                _ => {
                    state = JSDocState::SavingComments;
                    push_comment(
                        &mut comments,
                        &mut indent,
                        &mut margin,
                        self.scanner.token_text(),
                    );
                }
            }

            if state == JSDocState::SavingComments || state == JSDocState::SavingBackticks {
                self.next_jsdoc_comment_text_token(state == JSDocState::SavingBackticks);
            } else {
                self.next_token_jsdoc();
            }
        }

        comments = remove_leading_newlines(comments);
        comments = remove_trailing_whitespace(comments);
        if !comments.is_empty() {
            let jsdoc_text = self.finish_jsdoc_text(&comments, link_end, self.token_pos());
            parts.push(jsdoc_text);
        }

        let end = parts.last().map(|p| p.end()).unwrap_or(pos);
        Arc::new(NodeList {
            loc: TextRange::new(pos, end),
            nodes: parts,
        })
    }
}

impl super::Parser {

    fn parse_child_parameter_or_property_tag(
        &mut self,
        target: PropertyLikeParse,
        indent: usize,
        name: Option<Arc<Node>>,
    ) -> Option<Arc<Node>> {
        let mut can_parse_tag = false;
        let mut seen_asterisk = false;

        loop {
            match self.token {
                SyntaxKind::AtToken => {
                    if can_parse_tag && self.scanner.can_follow_jsdoc_at() {
                        return self.try_parse_child_tag(target, indent, name);
                    }
                }
                SyntaxKind::NewLineTrivia => {
                    can_parse_tag = true;
                    seen_asterisk = false;
                }
                SyntaxKind::AsteriskToken => {
                    if seen_asterisk {
                        can_parse_tag = false;
                    }
                    seen_asterisk = true;
                }
                SyntaxKind::Identifier => {
                    can_parse_tag = false;
                }
                SyntaxKind::EndOfFile => return None,
                _ => {}
            }
            self.next_token_jsdoc();
        }
    }

    fn try_parse_child_tag(
        &mut self,
        target: PropertyLikeParse,
        indent: usize,
        _name: Option<Arc<Node>>,
    ) -> Option<Arc<Node>> {
        debug_assert_eq!(self.token, SyntaxKind::AtToken);
        let start = self.token_pos();
        self.next_token_jsdoc();
        let tag_name = self.parse_jsdoc_identifier_name(None);
        self.skip_whitespace_or_asterisk();
        let tag_text = tag_name.text().to_string();

        let child_target = match tag_text.as_str() {
            "type" => {
                if target.contains(PropertyLikeParse::PROPERTY) {
                    return Some(self.parse_type_tag(start, tag_name, usize::MAX, ""));
                }
                return None;
            }
            "prop" | "property" => PropertyLikeParse(PropertyLikeParse::PROPERTY),
            "arg" | "argument" | "param" => PropertyLikeParse(
                PropertyLikeParse::PARAMETER | PropertyLikeParse::CALLBACK_PARAMETER,
            ),
            "template" => return Some(self.parse_template_tag(start, tag_name, indent, "")),
            "this" => return Some(self.parse_this_tag(start, tag_name, indent, "")),
            _ => return None,
        };

        if !target.contains(child_target.0) {
            return None;
        }
        Some(self.parse_parameter_or_property_tag(start, tag_name, target, indent))
    }
}

impl super::Parser {

    fn skip_whitespace(&mut self) {
        if self.token == SyntaxKind::WhitespaceTrivia || self.token == SyntaxKind::NewLineTrivia {

            if self.is_next_nonwhitespace_token_eof() {
                return;
            }
        }
        while self.token == SyntaxKind::WhitespaceTrivia || self.token == SyntaxKind::NewLineTrivia
        {
            self.next_token_jsdoc();
        }
    }

    fn is_next_nonwhitespace_token_eof(&mut self) -> bool {
        loop {
            self.next_token_jsdoc();
            if self.token == SyntaxKind::EndOfFile {
                return true;
            }
            if self.token != SyntaxKind::WhitespaceTrivia && self.token != SyntaxKind::NewLineTrivia
            {
                return false;
            }
        }
    }

    fn skip_whitespace_or_asterisk(&mut self) -> String {
        let mut indent_text = String::new();
        let mut preceding_line_break = false;
        let mut seen_line_break = false;

        loop {
            match self.token {
                SyntaxKind::WhitespaceTrivia => {
                    if preceding_line_break {
                        indent_text = String::new();
                        seen_line_break = true;
                    }
                    indent_text.push_str(self.scanner.token_text());
                    preceding_line_break = false;
                }
                SyntaxKind::NewLineTrivia => {
                    preceding_line_break = true;
                }
                SyntaxKind::AsteriskToken => {
                    preceding_line_break = false;
                }
                _ => break,
            }
            self.next_token_jsdoc();
        }

        if seen_line_break {
            indent_text
        } else {
            String::new()
        }
    }
}

fn is_jsdoc_like_text(text: &str) -> bool {
    text.starts_with("/**") && !text.starts_with("/**/")
}

fn is_jsdoc_link_tag(kind: &str) -> bool {
    matches!(kind, "link" | "linkcode" | "linkplain")
}

fn is_identifier_or_keyword_token(token: SyntaxKind) -> bool {
    token == SyntaxKind::Identifier || crate::ast::is_keyword_kind(token)
}

fn push_comment(comments: &mut Vec<String>, indent: &mut usize, margin: &mut i32, text: &str) {
    if *margin == -1 {

    }
    comments.push(text.to_string());
    *indent += text.len();
}

fn remove_leading_newlines(mut comments: Vec<String>) -> Vec<String> {
    let mut i = 0;
    while i < comments.len()
        && comments[i]
            .trim_matches(|c| c == '\r' || c == '\n')
            .is_empty()
    {
        i += 1;
    }
    comments.drain(..i);
    comments
}

fn trim_end(s: &str) -> String {
    s.trim_end_matches(|c: char| c.is_whitespace() || c == '\u{2028}' || c == '\u{2029}')
        .to_string()
}

fn remove_trailing_whitespace(mut comments: Vec<String>) -> Vec<String> {
    let mut end = comments.len();
    for i in (0..comments.len()).rev() {
        let trimmed = trim_end(&comments[i]);
        if trimmed.is_empty() {
            end = i;
        } else {
            comments[i] = trimmed;
            break;
        }
    }
    comments.truncate(end);
    comments
}

pub fn get_jsdoc_comment_ranges(text: &str, node: &Node) -> Vec<crate::scanner::CommentRange> {
    use crate::ast::SyntaxKind as SK;
    use crate::scanner::{get_leading_comment_ranges, get_trailing_comment_ranges};

    let token_pos = node.pos();

    let full_start = find_full_start(text, token_pos);

    let mut ranges = match node.kind {
        SK::Parameter
        | SK::TypeParameter
        | SK::FunctionExpression
        | SK::ArrowFunction
        | SK::ParenthesizedExpression
        | SK::VariableDeclaration
        | SK::ExportSpecifier => {
            let mut r = get_trailing_comment_ranges(text, token_pos);
            r.extend(get_leading_comment_ranges(text, full_start));
            r
        }
        _ => get_leading_comment_ranges(text, full_start),
    };

    let node_end = node.end();
    ranges.retain(|c| {
        let comment_start = c.pos;
        let comment_len = c.end.saturating_sub(comment_start);
        c.end <= node_end
            && comment_len >= 4
            && text.as_bytes().get(comment_start + 1) == Some(&b'*')
            && text.as_bytes().get(comment_start + 2) == Some(&b'*')
            && text.as_bytes().get(comment_start + 3) != Some(&b'/')
    });
    ranges
}

fn find_full_start(text: &str, token_pos: usize) -> usize {
    let bytes = text.as_bytes();
    let mut i = token_pos;

    while i > 0 {

        while i > 0
            && (bytes[i - 1] == b' '
                || bytes[i - 1] == b'\t'
                || bytes[i - 1] == b'\n'
                || bytes[i - 1] == b'\r')
        {
            i -= 1;
        }

        if i >= 2 && bytes[i - 2] == b'*' && bytes[i - 1] == b'/' {

            let mut j = i - 2;
            while j >= 2 {
                if bytes[j - 2] == b'/' && bytes[j - 1] == b'*' {
                    i = j - 2;
                    break;
                }
                j -= 1;
            }
            if j < 2 {

                break;
            }
        } else if i >= 2 && bytes[i - 2] == b'/' && bytes[i - 1] == b'/' {

            while i > 0 && bytes[i - 1] != b'\n' {
                i -= 1;
            }
        } else {

            break;
        }
    }
    i
}

pub fn parse_jsdoc_for_node(source_file: &crate::ast::SourceFile, node: &Node) -> Vec<Arc<Node>> {
    let text = &source_file.text;
    let ranges = get_jsdoc_comment_ranges(text, node);
    if ranges.is_empty() {
        return Vec::new();
    }

    let mut parser = super::Parser::new(text.clone());
    let mut jsdocs: Vec<Arc<Node>> = Vec::with_capacity(ranges.len());
    let mut pos = node.pos();
    for comment in &ranges {
        if let Some(parsed) = parser.parse_jsdoc_comment(comment.pos, comment.end, pos) {
            pos = parsed.end();
            jsdocs.push(parsed);
        }
    }
    jsdocs
}

#[cfg(test)]
mod tests {
    use super::*;

    fn parse_jsdoc(source: &str) -> Arc<Node> {
        let mut parser = super::super::Parser::new(source.to_string());

        let text = source;
        let start = text.find("/**").expect("no /** found");

        let end = text[start..].find("*/").expect("no */ found") + start + 2;
        parser
            .parse_jsdoc_comment(start, end, start)
            .expect("parse failed")
    }

    #[test]
    fn parse_empty_jsdoc() {
        let node = parse_jsdoc("/** */");
        assert_eq!(node.kind, SyntaxKind::JSDoc);
    }

    #[test]
    fn parse_simple_comment() {
        let node = parse_jsdoc("/** This is a comment */");
        assert_eq!(node.kind, SyntaxKind::JSDoc);
        if let NodeData::JSDoc(d) = &node.data {
            assert!(!d.comment.nodes.is_empty(), "should have comment text");
            assert!(d.tags.is_none(), "should have no tags");
        } else {
            panic!("not a JSDoc node");
        }
    }

    #[test]
    fn parse_param_tag() {
        let node = parse_jsdoc("/** @param {string} name The name */");
        assert_eq!(node.kind, SyntaxKind::JSDoc);
        if let NodeData::JSDoc(d) = &node.data {
            let tags = d.tags.as_ref().expect("should have tags");
            assert_eq!(tags.nodes.len(), 1);
            assert_eq!(tags.nodes[0].kind, SyntaxKind::JSDocParameterTag);
        }
    }

    #[test]
    fn parse_returns_tag() {
        let node = parse_jsdoc("/** @returns {number} The result */");
        if let NodeData::JSDoc(d) = &node.data {
            let tags = d.tags.as_ref().expect("should have tags");
            assert_eq!(tags.nodes.len(), 1);
            assert_eq!(tags.nodes[0].kind, SyntaxKind::JSDocReturnTag);
        }
    }

    #[test]
    fn parse_type_tag() {
        let node = parse_jsdoc("/** @type {string} */");
        if let NodeData::JSDoc(d) = &node.data {
            let tags = d.tags.as_ref().expect("should have tags");
            assert_eq!(tags.nodes.len(), 1);
            assert_eq!(tags.nodes[0].kind, SyntaxKind::JSDocTypeTag);
        }
    }

    #[test]
    fn parse_deprecated_tag() {
        let node = parse_jsdoc("/** @deprecated Use newThing instead */");
        if let NodeData::JSDoc(d) = &node.data {
            let tags = d.tags.as_ref().expect("should have tags");
            assert_eq!(tags.nodes.len(), 1);
            assert_eq!(tags.nodes[0].kind, SyntaxKind::JSDocDeprecatedTag);
        }
    }

    #[test]
    fn parse_multiple_tags() {
        let node = parse_jsdoc(
            "/**\n * @param {string} x First\n * @param {number} y Second\n * @returns {boolean}\n */",
        );
        if let NodeData::JSDoc(d) = &node.data {
            let tags = d.tags.as_ref().expect("should have tags");
            assert_eq!(tags.nodes.len(), 3);
            assert_eq!(tags.nodes[0].kind, SyntaxKind::JSDocParameterTag);
            assert_eq!(tags.nodes[1].kind, SyntaxKind::JSDocParameterTag);
            assert_eq!(tags.nodes[2].kind, SyntaxKind::JSDocReturnTag);
        }
    }

    #[test]
    fn parse_template_tag() {
        let node = parse_jsdoc("/** @template T */");
        if let NodeData::JSDoc(d) = &node.data {
            let tags = d.tags.as_ref().expect("should have tags");
            assert_eq!(tags.nodes.len(), 1);
            assert_eq!(tags.nodes[0].kind, SyntaxKind::JSDocTemplateTag);
        }
    }

    #[test]
    fn parse_typedef_tag() {
        let node = parse_jsdoc("/** @typedef {Object} MyType */");
        if let NodeData::JSDoc(d) = &node.data {
            let tags = d.tags.as_ref().expect("should have tags");
            assert_eq!(tags.nodes.len(), 1);
            assert_eq!(tags.nodes[0].kind, SyntaxKind::JSDocTypedefTag);
        }
    }

    #[test]
    fn parse_callback_tag() {
        let node = parse_jsdoc("/** @callback MyCallback */");
        if let NodeData::JSDoc(d) = &node.data {
            let tags = d.tags.as_ref().expect("should have tags");
            assert_eq!(tags.nodes.len(), 1);
            assert_eq!(tags.nodes[0].kind, SyntaxKind::JSDocCallbackTag);
        }
    }

    #[test]
    fn parse_see_tag() {
        let node = parse_jsdoc("/** @see OtherThing */");
        if let NodeData::JSDoc(d) = &node.data {
            let tags = d.tags.as_ref().expect("should have tags");
            assert_eq!(tags.nodes.len(), 1);
            assert_eq!(tags.nodes[0].kind, SyntaxKind::JSDocSeeTag);
        }
    }

    #[test]
    fn parse_simple_tags() {
        for (tag_str, expected_kind) in [
            ("@public", SyntaxKind::JSDocPublicTag),
            ("@private", SyntaxKind::JSDocPrivateTag),
            ("@protected", SyntaxKind::JSDocProtectedTag),
            ("@readonly", SyntaxKind::JSDocReadonlyTag),
            ("@override", SyntaxKind::JSDocOverrideTag),
        ] {
            let source = format!("/** {} */", tag_str);
            let node = parse_jsdoc(&source);
            if let NodeData::JSDoc(d) = &node.data {
                let tags = d.tags.as_ref().expect("should have tags");
                assert_eq!(tags.nodes.len(), 1, "tag {} should parse", tag_str);
                assert_eq!(
                    tags.nodes[0].kind, expected_kind,
                    "tag {} should be {:?}",
                    tag_str, expected_kind
                );
            }
        }
    }

    #[test]
    fn parse_unknown_tag() {
        let node = parse_jsdoc("/** @customtag some text */");
        if let NodeData::JSDoc(d) = &node.data {
            let tags = d.tags.as_ref().expect("should have tags");
            assert_eq!(tags.nodes.len(), 1);
            assert_eq!(tags.nodes[0].kind, SyntaxKind::JSDocUnknownTag);
        }
    }

    #[test]
    fn parse_throws_tag() {
        let node = parse_jsdoc("/** @throws {Error} When something goes wrong */");
        if let NodeData::JSDoc(d) = &node.data {
            let tags = d.tags.as_ref().expect("should have tags");
            assert_eq!(tags.nodes.len(), 1);
            assert_eq!(tags.nodes[0].kind, SyntaxKind::JSDocThrowsTag);
        }
    }

    #[test]
    fn parse_satisfies_tag() {
        let node = parse_jsdoc("/** @satisfies {string} */");
        if let NodeData::JSDoc(d) = &node.data {
            let tags = d.tags.as_ref().expect("should have tags");
            assert_eq!(tags.nodes.len(), 1);
            assert_eq!(tags.nodes[0].kind, SyntaxKind::JSDocSatisfiesTag);
        }
    }

    #[test]
    fn parse_this_tag() {
        let node = parse_jsdoc("/** @this {MyClass} */");
        if let NodeData::JSDoc(d) = &node.data {
            let tags = d.tags.as_ref().expect("should have tags");
            assert_eq!(tags.nodes.len(), 1);
            assert_eq!(tags.nodes[0].kind, SyntaxKind::JSDocThisTag);
        }
    }

    #[test]
    fn parse_param_with_brackets() {
        let node = parse_jsdoc("/** @param {string} [name] Optional */");
        if let NodeData::JSDoc(d) = &node.data {
            let tags = d.tags.as_ref().expect("should have tags");
            assert_eq!(tags.nodes.len(), 1);
            let tag = &tags.nodes[0];
            assert_eq!(tag.kind, SyntaxKind::JSDocParameterTag);
            if let NodeData::JSDocParameterOrPropertyTag(td) = &tag.data {
                assert!(td.is_bracketed, "should be bracketed");
            }
        }
    }

    #[test]
    fn parse_multiline_comment_with_tags() {
        let source = "/**
 * Description here.
 *
 * @param {string} name - The name
 * @param {number} age - The age
 * @returns {Person} A person object
 */";
        let node = parse_jsdoc(source);
        if let NodeData::JSDoc(d) = &node.data {
            let tags = d.tags.as_ref().expect("should have tags");
            assert_eq!(tags.nodes.len(), 3);
        }
    }

    #[test]
    fn parse_link_in_comment() {
        let node = parse_jsdoc("/** See {@link Foo} for details */");
        assert_eq!(node.kind, SyntaxKind::JSDoc);

        if let NodeData::JSDoc(d) = &node.data {
            assert!(!d.comment.nodes.is_empty());
        }
    }

    #[test]
    fn parse_implements_tag() {
        let node = parse_jsdoc("/** @implements {IFoo} */");
        if let NodeData::JSDoc(d) = &node.data {
            let tags = d.tags.as_ref().expect("should have tags");
            assert_eq!(tags.nodes.len(), 1);
            assert_eq!(tags.nodes[0].kind, SyntaxKind::JSDocImplementsTag);
        }
    }

    #[test]
    fn parse_augments_tag() {
        let node = parse_jsdoc("/** @augments {Base} */");
        if let NodeData::JSDoc(d) = &node.data {
            let tags = d.tags.as_ref().expect("should have tags");
            assert_eq!(tags.nodes.len(), 1);
            assert_eq!(tags.nodes[0].kind, SyntaxKind::JSDocAugmentsTag);
        }
    }

    #[test]
    fn parse_overload_tag() {
        let node = parse_jsdoc("/** @overload */");
        if let NodeData::JSDoc(d) = &node.data {
            let tags = d.tags.as_ref().expect("should have tags");
            assert_eq!(tags.nodes.len(), 1);
            assert_eq!(tags.nodes[0].kind, SyntaxKind::JSDocOverloadTag);
        }
    }

    #[test]
    fn parse_template_with_constraint() {
        let node = parse_jsdoc("/** @template {string} T */");
        if let NodeData::JSDoc(d) = &node.data {
            let tags = d.tags.as_ref().expect("should have tags");
            assert_eq!(tags.nodes.len(), 1);
            assert_eq!(tags.nodes[0].kind, SyntaxKind::JSDocTemplateTag);
        }
    }

    #[test]
    fn parse_template_multiple() {
        let node = parse_jsdoc("/** @template T,U,V */");
        if let NodeData::JSDoc(d) = &node.data {
            let tags = d.tags.as_ref().expect("should have tags");
            assert_eq!(tags.nodes.len(), 1);
            assert_eq!(tags.nodes[0].kind, SyntaxKind::JSDocTemplateTag);
            if let NodeData::JSDocTemplateTag(td) = &tags.nodes[0].data {
                assert_eq!(td.type_parameters.nodes.len(), 3);
            }
        }
    }

    #[test]
    fn parse_param_name_first() {

        let node = parse_jsdoc("/** @param name {string} */");
        if let NodeData::JSDoc(d) = &node.data {
            let tags = d.tags.as_ref().expect("should have tags");
            assert_eq!(tags.nodes.len(), 1);
            assert_eq!(tags.nodes[0].kind, SyntaxKind::JSDocParameterTag);
            if let NodeData::JSDocParameterOrPropertyTag(td) = &tags.nodes[0].data {
                assert!(td.is_name_first, "should be name first");
                assert!(td.type_expression.is_some(), "should have type");
            }
        }
    }

    #[test]
    fn parse_jsdoc_like_text_detection() {
        assert!(is_jsdoc_like_text("/** comment */"));
        assert!(!is_jsdoc_like_text("/**/"));
        assert!(!is_jsdoc_like_text("/* not jsdoc */"));
    }

    #[test]
    fn parse_remove_trailing_whitespace() {
        let comments = vec!["hello".to_string(), "  ".to_string(), "\n".to_string()];
        let result = remove_trailing_whitespace(comments);
        assert_eq!(result.len(), 1);
        assert_eq!(result[0], "hello");
    }

    #[test]
    fn parse_remove_leading_newlines() {
        let comments = vec!["\n".to_string(), "\r\n".to_string(), "hello".to_string()];
        let result = remove_leading_newlines(comments);
        assert_eq!(result.len(), 1);
        assert_eq!(result[0], "hello");
    }

    fn parse_source(source: &str) -> crate::ast::SourceFile {
        super::super::Parser::parse_source_file_text("test.ts", source.to_string())
    }

    fn first_statement(file: &crate::ast::SourceFile) -> Arc<Node> {
        use crate::ast::node_data_generated::*;
        match &file.node.data {
            NodeData::SourceFile(d) => d.statements.nodes[0].clone(),
            _ => panic!("expected SourceFile"),
        }
    }

    #[test]
    fn get_jsdoc_comment_ranges_finds_leading_jsdoc() {
        let text = "/** Hello */\nconst x = 1;";
        let file = parse_source(text);
        let stmt = first_statement(&file);
        let ranges = get_jsdoc_comment_ranges(&file.text, &stmt);
        assert_eq!(ranges.len(), 1);
        assert!(text[ranges[0].pos..ranges[0].end].starts_with("/**"));
    }

    #[test]
    fn get_jsdoc_comment_ranges_skips_non_jsdoc_comments() {
        let text = "/* not jsdoc */\nconst x = 1;";
        let file = parse_source(text);
        let stmt = first_statement(&file);
        let ranges = get_jsdoc_comment_ranges(&file.text, &stmt);
        assert_eq!(ranges.len(), 0, "plain /* */ comments are not JSDoc");
    }

    #[test]
    fn get_jsdoc_comment_ranges_skips_empty_jsdoc() {
        let text = "/**/\nconst x = 1;";
        let file = parse_source(text);
        let stmt = first_statement(&file);
        let ranges = get_jsdoc_comment_ranges(&file.text, &stmt);
        assert_eq!(ranges.len(), 0, "/**/ is not JSDoc");
    }

    #[test]
    fn parse_jsdoc_for_node_returns_parsed_tags() {
        let text = "/**\n * @param {string} name\n * @returns {void}\n */\nfunction f(name) {}\n";
        let file = parse_source(text);
        let stmt = first_statement(&file);
        let jsdocs = parse_jsdoc_for_node(&file, &stmt);
        assert_eq!(jsdocs.len(), 1);
        assert_eq!(jsdocs[0].kind, SyntaxKind::JSDoc);

        if let NodeData::JSDoc(d) = &jsdocs[0].data {
            let tags = d.tags.as_ref().expect("should have tags");
            assert_eq!(tags.nodes.len(), 2);
        }
    }

    #[test]
    fn parse_jsdoc_for_node_no_comments_returns_empty() {
        let text = "const x = 1;";
        let file = parse_source(text);
        let stmt = first_statement(&file);
        let jsdocs = parse_jsdoc_for_node(&file, &stmt);
        assert!(jsdocs.is_empty());
    }

    #[test]
    fn resolve_jsdoc_caches_result() {
        let text = "/** Doc */\nconst x = 1;";
        let file = parse_source(text);
        let stmt = first_statement(&file);

        let jsdocs1 = file.resolve_jsdoc(&stmt);
        assert_eq!(jsdocs1.len(), 1);

        let jsdocs2 = file.resolve_jsdoc(&stmt);
        assert_eq!(jsdocs2.len(), 1);
        assert_eq!(jsdocs1[0].kind, jsdocs2[0].kind);
    }

    #[test]
    fn resolve_jsdoc_multiple_jsdoc_comments() {
        let text = "/** First */\n/** Second */\nconst x = 1;";
        let file = parse_source(text);
        let stmt = first_statement(&file);
        let jsdocs = file.resolve_jsdoc(&stmt);
        assert_eq!(jsdocs.len(), 2, "should find both JSDoc comments");
    }

    #[test]
    fn node_jsdoc_returns_empty_without_flag() {
        let text = "/** Doc */\nconst x = 1;";
        let file = parse_source(text);
        let stmt = first_statement(&file);

        let jsdocs = stmt.jsdoc(&file);
        assert!(
            jsdocs.is_empty(),
            "jsdoc() should return empty without HasJSDoc flag"
        );
    }
}
