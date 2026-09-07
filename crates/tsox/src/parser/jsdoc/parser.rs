#![allow(unused_imports)]

use super::*;

impl crate::parser::Parser {
    pub fn parse_jsdoc_comment(
        &mut self,
        start: usize,
        end: usize,
        full_start: usize,
    ) -> Option<Arc<Node>> {
        {
            let text = self.scanner.text();
            if !is_jsdoc_like_text(&text[start..]) {
                return None;
            }
        }
        let last_newline = {
            let text = self.scanner.text();
            text[..start].rfind('\n').map(|i| i + 1).unwrap_or(0)
        };
        let initial_indent = start + 4 - last_newline;

        let saved_scanner = self.scanner.save_state();
        let saved_token = self.token;
        let saved_diagnostics_len = self.diagnostics.len();

        self.scanner.set_range(start + 3, end - 2);

        let comment = self.parse_jsdoc_comment_worker(start, end, full_start, initial_indent);

        self.scanner.restore_state(saved_scanner);
        self.token = saved_token;
        self.diagnostics.truncate(saved_diagnostics_len);

        Some(comment)
    }

    pub(crate) fn parse_jsdoc_comment_worker(
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

    pub(crate) fn finish_jsdoc_text(
        &self,
        comments: &[String],
        pos: usize,
        end: usize,
    ) -> Arc<Node> {
        Arc::new(Node::with_loc(
            SyntaxKind::JSDocText,
            NodeData::JSDocText(JSDocTextData {
                text: comments.to_vec(),
            }),
            TextRange::new(pos, end),
        ))
    }
}
