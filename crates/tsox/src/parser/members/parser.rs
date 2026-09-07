#![allow(unused_imports)]

use super::*;

impl Parser {
    pub(crate) fn parse_template_expression(&mut self) -> Arc<Node> {
        let pos = self.token_pos();
        let head = self.create_token_node();
        self.next_token();
        let mut spans = Vec::new();
        loop {
            let expression = self.parse_expression();
            let literal = if self.token == SyntaxKind::CloseBraceToken {
                self.next_template_token();
                self.create_token_node()
            } else {
                break;
            };
            let span_pos = expression.pos();
            let span_end = literal.end();
            spans.push(Arc::new(Node::with_loc(
                SyntaxKind::TemplateSpan,
                NodeData::TemplateSpan(TemplateSpanData {
                    expression,
                    literal,
                }),
                TextRange::new(span_pos, span_end),
            )));
            if self.token == SyntaxKind::NoSubstitutionTemplateLiteral
                || self.token == SyntaxKind::TemplateTail
            {
                self.next_token();
                break;
            }
            self.next_token();
        }
        let end = self.token_pos();
        Arc::new(Node::with_loc(
            SyntaxKind::TemplateExpression,
            NodeData::TemplateExpression(TemplateExpressionData {
                head,
                template_spans: Arc::new(NodeList {
                    loc: TextRange::new(pos, end),
                    nodes: spans,
                }),
            }),
            TextRange::new(pos, end),
        ))
    }

    pub(crate) fn parse_optional_type_parameters(&mut self) -> Option<Arc<NodeList>> {
        if self.token != SyntaxKind::LessThanToken {
            return None;
        }
        let pos = self.token_pos();
        self.next_token();
        let params =
            self.parse_delimited_list(ParsingContext::TypeParameters, Parser::parse_type_parameter);

        self.re_scan_greater_than();

        if params.nodes.is_empty() {
            self.parse_error_at_range(
                crate::core::text::TextRange::new(pos, pos + 1),
                crate::diagnostics::messages_generated::TYPE_PARAMETER_LIST_CANNOT_BE_EMPTY,
                &[],
            );
        }
        self.expect(SyntaxKind::GreaterThanToken);
        let end = self.token_pos();
        Some(Arc::new(NodeList {
            loc: TextRange::new(pos, end),
            nodes: params.nodes,
        }))
    }

    pub(crate) fn parse_type_parameter_modifiers(&mut self) -> Option<Arc<ModifierList>> {
        let mut modifiers: Vec<(SyntaxKind, usize, usize)> = Vec::new();
        loop {
            if !matches!(
                self.token,
                SyntaxKind::InKeyword | SyntaxKind::OutKeyword | SyntaxKind::ConstKeyword
            ) {
                break;
            }

            let mut s = self.scanner.clone();
            s.scan();
            if s.has_preceding_line_break() || !Self::token_can_follow_modifier(s.token()) {
                break;
            }
            let kind = self.token;
            let pos = self.token_pos();
            let end = self.token_end();
            self.next_token();
            modifiers.push((kind, pos, end));
        }
        if modifiers.is_empty() {
            None
        } else {
            Some(self.make_modifier_list(modifiers))
        }
    }

    pub(crate) fn parse_type_parameter(&mut self) -> Arc<Node> {
        let pos = self.token_pos();

        let modifiers = self.parse_type_parameter_modifiers();
        let name = self.parse_identifier();
        let constraint = if self.parse_optional(SyntaxKind::ExtendsKeyword) {
            Some(self.parse_type())
        } else {
            None
        };
        let default_type = if self.parse_optional(SyntaxKind::EqualsToken) {
            Some(self.parse_type())
        } else {
            None
        };
        let end = default_type.as_ref().map_or_else(
            || constraint.as_ref().map_or(name.end(), |c| c.end()),
            |d| d.end(),
        );
        Arc::new(Node::with_loc(
            SyntaxKind::TypeParameter,
            NodeData::TypeParameterDeclaration(TypeParameterDeclarationData {
                modifiers,
                name,
                constraint,
                expression: None,
                default_type,
            }),
            TextRange::new(pos, end),
        ))
    }

    pub(crate) fn parse_parameter_list(&mut self) -> Arc<NodeList> {
        let pos = self.token_pos();
        self.expect(SyntaxKind::OpenParenToken);
        let params = self.parse_delimited_list(ParsingContext::Parameters, Parser::parse_parameter);
        self.expect(SyntaxKind::CloseParenToken);
        let end = self.token_pos();
        Arc::new(NodeList {
            loc: TextRange::new(pos, end),
            nodes: params.nodes,
        })
    }

    pub(crate) fn token_after_modifier_can_follow(&self, s: &mut crate::scanner::Scanner) -> bool {
        match self.token {
            SyntaxKind::ConstKeyword => s.token() == SyntaxKind::EnumKeyword,
            SyntaxKind::ExportKeyword => match s.token() {
                SyntaxKind::DefaultKeyword => {
                    let t = s.scan();
                    Self::token_can_follow_default_keyword(t, s)
                }
                SyntaxKind::TypeKeyword => {
                    let t = s.scan();
                    Self::can_follow_export_modifier(t)
                }
                _ => Self::can_follow_export_modifier(s.token()),
            },
            SyntaxKind::DefaultKeyword => Self::token_can_follow_default_keyword(s.token(), s),

            SyntaxKind::StaticKeyword => Self::token_can_follow_modifier(s.token()),
            _ => !s.has_preceding_line_break() && Self::token_can_follow_modifier(s.token()),
        }
    }

    pub(crate) fn parse_parameter_modifiers(&mut self) -> Option<Arc<ModifierList>> {
        enum Entry {
            Mod(SyntaxKind, usize, usize),
            Dec(Arc<Node>),
        }
        let mut entries: Vec<Entry> = Vec::new();
        let mut flags = ModifierFlags::empty();
        let mut has_leading_modifier = false;
        let mut has_trailing_decorator = false;
        let mut has_trailing_modifier = false;
        let mut has_static_modifier = false;
        loop {
            if self.token == SyntaxKind::AtToken && !has_trailing_modifier {
                let dec = self.parse_decorator();
                if has_leading_modifier {
                    has_trailing_decorator = true;
                }
                flags |= ModifierFlags::Decorator;
                entries.push(Entry::Dec(dec));
                continue;
            }

            if has_static_modifier && self.token == SyntaxKind::StaticKeyword {
                break;
            }
            if !is_modifier_kind(self.token) {
                break;
            }
            let mut s = self.scanner.clone();
            s.scan();
            if !self.token_after_modifier_can_follow(&mut s) {
                break;
            }
            let kind = self.token;
            let mpos = self.token_pos();
            let mend = self.token_end();
            self.next_token();
            if kind == SyntaxKind::StaticKeyword {
                has_static_modifier = true;
            }
            flags |= Self::modifier_flag(kind);
            entries.push(Entry::Mod(kind, mpos, mend));
            if has_trailing_decorator {
                has_trailing_modifier = true;
            } else {
                has_leading_modifier = true;
            }
        }
        if entries.is_empty() {
            return None;
        }
        let nodes = entries
            .into_iter()
            .map(|e| match e {
                Entry::Mod(kind, pos, end) => Arc::new(Node::with_loc(
                    kind,
                    NodeData::Token,
                    TextRange::new(pos, end),
                )),
                Entry::Dec(n) => n,
            })
            .collect();
        Some(Arc::new(ModifierList::new(nodes, flags)))
    }

    pub(crate) fn parse_parameter(&mut self) -> Arc<Node> {
        let pos = self.token_pos();

        let modifiers = self.parse_parameter_modifiers();

        let dot_dot_dot_token = self.parse_optional_token(SyntaxKind::DotDotDotToken);

        let name = self.parse_identifier_or_pattern_with_diagnostic(Some(
            &diagnostics::PRIVATE_IDENTIFIERS_CANNOT_BE_USED_AS_PARAMETERS,
        ));
        let question_token = self.parse_optional_token(SyntaxKind::QuestionToken);
        let type_node = self.parse_optional_type_annotation();
        let initializer = if self.token == SyntaxKind::EqualsToken {
            self.next_token();
            Some(self.parse_assignment_expression())
        } else {
            None
        };
        let end = initializer.as_ref().map_or_else(
            || type_node.as_ref().map_or(name.end(), |t| t.end()),
            |i| i.end(),
        );
        Arc::new(Node::with_loc(
            SyntaxKind::Parameter,
            NodeData::ParameterDeclaration(ParameterDeclarationData {
                modifiers,
                dot_dot_dot_token,
                name,
                question_token,
                type_node,
                initializer,
            }),
            TextRange::new(pos, end),
        ))
    }
}
