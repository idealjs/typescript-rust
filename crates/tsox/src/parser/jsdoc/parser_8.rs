#![allow(unused_imports)]

use super::*;

impl crate::parser::Parser {
    pub(crate) fn parse_trailing_tag_comments(
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

    pub(crate) fn parse_tag_comments(
        &mut self,
        indent: usize,
        initial_margin: Option<&str>,
    ) -> Arc<NodeList> {
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

impl crate::parser::Parser {
    pub(crate) fn parse_child_parameter_or_property_tag(
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

    pub(crate) fn try_parse_child_tag(
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
