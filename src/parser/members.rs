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

    pub(crate) fn token_after_modifier_can_follow(
        &self,
        s: &mut crate::scanner::Scanner,
    ) -> bool {
        match self.token {

            SyntaxKind::ConstKeyword => s.token() == SyntaxKind::EnumKeyword,
            SyntaxKind::ExportKeyword => {
                match s.token() {
                    SyntaxKind::DefaultKeyword => {
                        let t = s.scan();
                        Self::token_can_follow_default_keyword(t, s)
                    }
                    SyntaxKind::TypeKeyword => {
                        let t = s.scan();
                        Self::can_follow_export_modifier(t)
                    }
                    _ => Self::can_follow_export_modifier(s.token()),
                }
            }
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

    pub(crate) fn parse_heritage_clauses(&mut self) -> Option<Arc<NodeList>> {
        let mut clauses = Vec::new();
        let mut pos = self.token_pos();
        if self.parse_optional(SyntaxKind::ExtendsKeyword) {
            let types = self.parse_delimited_list(
                ParsingContext::HeritageClauseElement,
                Parser::parse_heritage_clause_element,
            );
            let end = self.token_pos();
            clauses.push(Arc::new(Node::with_loc(
                SyntaxKind::HeritageClause,
                NodeData::HeritageClause(HeritageClauseData {
                    token: SyntaxKind::ExtendsKeyword,
                    types: Arc::new(types),
                }),
                TextRange::new(pos, end),
            )));
        }
        if self.token == SyntaxKind::ImplementsKeyword {
            pos = self.token_pos();
            self.next_token();
            let types = self.parse_delimited_list(
                ParsingContext::HeritageClauseElement,
                Parser::parse_heritage_clause_element,
            );
            let end = self.token_pos();
            clauses.push(Arc::new(Node::with_loc(
                SyntaxKind::HeritageClause,
                NodeData::HeritageClause(HeritageClauseData {
                    token: SyntaxKind::ImplementsKeyword,
                    types: Arc::new(types),
                }),
                TextRange::new(pos, end),
            )));
        }
        if clauses.is_empty() {
            None
        } else {
            let end = clauses.last().unwrap().end();
            Some(Arc::new(NodeList {
                loc: TextRange::new(clauses[0].pos(), end),
                nodes: clauses,
            }))
        }
    }

    pub(crate) fn parse_heritage_clause_element(&mut self) -> Arc<Node> {
        let pos = self.token_pos();
        let expression = self.parse_left_hand_side_expression();
        let type_arguments = self.parse_optional_type_arguments();
        let end = type_arguments
            .as_ref()
            .map_or(expression.end(), |ta| ta.end());
        Arc::new(Node::with_loc(
            SyntaxKind::ExpressionWithTypeArguments,
            NodeData::ExpressionWithTypeArguments(ExpressionWithTypeArgumentsData {
                expression,
                type_arguments,
            }),
            TextRange::new(pos, end),
        ))
    }

    pub(crate) fn parse_type_member(&mut self) -> Arc<Node> {
        if self.token == SyntaxKind::OpenParenToken || self.token == SyntaxKind::LessThanToken {
            return self.parse_signature_member(SyntaxKind::CallSignature);
        }

        if self.token == SyntaxKind::NewKeyword && {
            let mut s = self.scanner.clone();
            let t = s.scan();
            t == SyntaxKind::OpenParenToken || t == SyntaxKind::LessThanToken
        } {
            return self.parse_signature_member(SyntaxKind::ConstructSignature);
        }

        let pos = self.token_pos();
        let modifiers = self.parse_type_member_modifiers();

        if self.token == SyntaxKind::GetKeyword || self.token == SyntaxKind::SetKeyword {
            let mut s = self.scanner.clone();
            s.scan();
            if Self::token_can_follow_get_or_set(s.token()) {
                let accessor_kind = self.token;
                return self.parse_accessor_declaration(pos, modifiers, accessor_kind);
            }
        }
        if self.is_index_signature_start() {
            return self.parse_index_signature(pos, modifiers);
        }

        let name = self.parse_property_name();
        let postfix_token = self.parse_optional_token(SyntaxKind::QuestionToken);
        let type_parameters = self.parse_optional_type_parameters();
        if self.token == SyntaxKind::OpenParenToken {
            let parameters = self.parse_parameter_list();
            let type_node = self.parse_optional_return_type();
            self.parse_type_member_semicolon();
            let end = self.token_pos();
            return Arc::new(Node::with_loc(
                SyntaxKind::MethodSignature,
                NodeData::MethodSignatureDeclaration(MethodSignatureDeclarationData {
                    modifiers,
                    name,
                    postfix_token,
                    type_parameters,
                    parameters,
                    type_node,
                }),
                TextRange::new(pos, end),
            ));
        }

        let type_node = self
            .parse_optional_type_annotation()
            .unwrap_or_else(|| self.missing_node(self.token_pos()));
        let initializer = if self.parse_optional(SyntaxKind::EqualsToken) {
            self.parse_type()
        } else {
            self.missing_node(self.token_pos())
        };
        self.parse_type_member_semicolon();
        let end = self.token_pos();
        Arc::new(Node::with_loc(
            SyntaxKind::PropertySignature,
            NodeData::PropertySignatureDeclaration(PropertySignatureDeclarationData {
                modifiers,
                name,
                postfix_token,
                type_node,
                initializer,
            }),
            TextRange::new(pos, end),
        ))
    }

    pub(crate) fn parse_type_member_modifiers(&mut self) -> Option<Arc<ModifierList>> {
        let mut modifiers = Vec::new();

        while matches!(
            self.token,
            SyntaxKind::ReadonlyKeyword
                | SyntaxKind::StaticKeyword
                | SyntaxKind::PublicKeyword
                | SyntaxKind::PrivateKeyword
                | SyntaxKind::ProtectedKeyword
        ) {
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

    pub(crate) fn token_can_follow_modifier(token: SyntaxKind) -> bool {
        token == SyntaxKind::OpenBracketToken
            || token == SyntaxKind::OpenBraceToken
            || token == SyntaxKind::AsteriskToken
            || token == SyntaxKind::DotDotDotToken
            || is_identifier_or_keyword(token)
            || token == SyntaxKind::StringLiteral
            || token == SyntaxKind::NumericLiteral
            || token == SyntaxKind::BigIntLiteral
    }

    pub(crate) fn token_can_follow_get_or_set(token: SyntaxKind) -> bool {
        token == SyntaxKind::OpenBracketToken
            || is_identifier_or_keyword(token)
            || token == SyntaxKind::StringLiteral
            || token == SyntaxKind::NumericLiteral
            || token == SyntaxKind::BigIntLiteral
    }

    pub(crate) fn can_follow_export_modifier(token: SyntaxKind) -> bool {
        token == SyntaxKind::AtToken
            || (token != SyntaxKind::AsteriskToken
                && token != SyntaxKind::AsKeyword
                && token != SyntaxKind::OpenBraceToken
                && Self::token_can_follow_modifier(token))
    }

    pub(crate) fn token_can_follow_default_keyword(
        t: SyntaxKind,
        s: &mut crate::scanner::Scanner,
    ) -> bool {
        match t {
            SyntaxKind::ClassKeyword
            | SyntaxKind::FunctionKeyword
            | SyntaxKind::InterfaceKeyword
            | SyntaxKind::AtToken => true,
            SyntaxKind::AbstractKeyword => {
                s.scan() == SyntaxKind::ClassKeyword && !s.has_preceding_line_break()
            }
            SyntaxKind::AsyncKeyword => {
                s.scan() == SyntaxKind::FunctionKeyword && !s.has_preceding_line_break()
            }
            _ => false,
        }
    }

    pub(crate) fn is_class_member_modifier(token: SyntaxKind) -> bool {
        matches!(
            token,
            SyntaxKind::PublicKeyword
                | SyntaxKind::PrivateKeyword
                | SyntaxKind::ProtectedKeyword
                | SyntaxKind::ReadonlyKeyword
                | SyntaxKind::StaticKeyword
                | SyntaxKind::OverrideKeyword
                | SyntaxKind::AccessorKeyword
        )
    }

    pub(crate) fn look_ahead_class_member_start(&self) -> bool {
        if self.token == SyntaxKind::AtToken {
            return true;
        }
        let mut id_token = SyntaxKind::Unknown;
        let mut s = self.scanner.clone();
        let mut t = self.token;

        while is_modifier_kind(t) {
            id_token = t;
            if Self::is_class_member_modifier(id_token) {
                return true;
            }
            t = s.scan();
        }
        if t == SyntaxKind::AsteriskToken {
            return true;
        }

        if is_identifier_or_keyword(t)
            || t == SyntaxKind::PrivateIdentifier
            || t == SyntaxKind::StringLiteral
            || t == SyntaxKind::NumericLiteral
            || t == SyntaxKind::BigIntLiteral
        {
            id_token = t;
            t = s.scan();
        }

        if t == SyntaxKind::OpenBracketToken {
            return true;
        }

        if id_token != SyntaxKind::Unknown {

            if !is_keyword_kind(id_token)
                || id_token == SyntaxKind::SetKeyword
                || id_token == SyntaxKind::GetKeyword
            {
                return true;
            }

            match t {
                SyntaxKind::OpenParenToken
                | SyntaxKind::LessThanToken
                | SyntaxKind::ExclamationToken
                | SyntaxKind::ColonToken
                | SyntaxKind::EqualsToken
                | SyntaxKind::QuestionToken => return true,
                _ => {}
            }

            return t == SyntaxKind::SemicolonToken
                || t == SyntaxKind::CloseBraceToken
                || t == SyntaxKind::EndOfFile
                || s.has_preceding_line_break();
        }
        false
    }

    pub(crate) fn is_index_signature_start(&self) -> bool {

        if self.token != SyntaxKind::OpenBracketToken {
            return false;
        }
        let mut s = self.scanner.clone();
        s.scan();
        let t1 = s.token();

        if t1 == SyntaxKind::DotDotDotToken || t1 == SyntaxKind::CloseBracketToken {
            return true;
        }

        if is_modifier_kind(t1) {
            s.scan();
            return Self::token_is_identifier(&s);
        }

        if !Self::token_is_identifier(&s) {
            return false;
        }

        s.scan();

        let t2 = s.token();
        if t2 == SyntaxKind::ColonToken || t2 == SyntaxKind::CommaToken {
            return true;
        }

        if t2 != SyntaxKind::QuestionToken {
            return false;
        }
        s.scan();
        matches!(
            s.token(),
            SyntaxKind::ColonToken | SyntaxKind::CommaToken | SyntaxKind::CloseBracketToken
        )
    }

    pub(crate) fn token_is_identifier(scanner: &crate::scanner::Scanner) -> bool {
        let t = scanner.token();
        if t == SyntaxKind::Identifier {
            return true;
        }

        (t as i16) > (SyntaxKind::WithKeyword as i16)
    }

    pub(crate) fn parse_index_signature(
        &mut self,
        pos: usize,
        modifiers: Option<Arc<ModifierList>>,
    ) -> Arc<Node> {
        let parameters = self.parse_bracketedList(
            ParsingContext::Parameters,
            Parser::parse_parameter,
            SyntaxKind::OpenBracketToken,
            SyntaxKind::CloseBracketToken,
        );
        let type_node = self
            .parse_optional_type_annotation()
            .unwrap_or_else(|| self.missing_node(self.token_pos()));
        self.parse_type_member_semicolon();
        let end = self.token_pos();
        Arc::new(Node::with_loc(
            SyntaxKind::IndexSignature,
            NodeData::IndexSignatureDeclaration(IndexSignatureDeclarationData {
                modifiers,
                parameters: Arc::new(parameters),
                type_node,
            }),
            TextRange::new(pos, end),
        ))
    }

    pub(crate) fn parse_signature_member(&mut self, kind: SyntaxKind) -> Arc<Node> {
        let pos = self.token_pos();
        if kind == SyntaxKind::ConstructSignature {
            self.expect(SyntaxKind::NewKeyword);
        }
        let type_parameters = self.parse_optional_type_parameters();
        let parameters = self.parse_parameter_list();
        let type_node = self.parse_optional_return_type();
        self.parse_type_member_semicolon();
        let end = self.token_pos();
        if kind == SyntaxKind::CallSignature {
            Arc::new(Node::with_loc(
                SyntaxKind::CallSignature,
                NodeData::CallSignatureDeclaration(CallSignatureDeclarationData {
                    type_parameters,
                    parameters,
                    type_node,
                }),
                TextRange::new(pos, end),
            ))
        } else {
            Arc::new(Node::with_loc(
                SyntaxKind::ConstructSignature,
                NodeData::ConstructSignatureDeclaration(ConstructSignatureDeclarationData {
                    type_parameters,
                    parameters,
                    type_node,
                }),
                TextRange::new(pos, end),
            ))
        }
    }

    pub(crate) fn parse_type_member_semicolon(&mut self) {
        if !self.parse_optional(SyntaxKind::SemicolonToken) {
            self.parse_optional(SyntaxKind::CommaToken);
        }
    }

    pub(crate) fn parse_class_members(&mut self) -> NodeList {
        let pos = self.token_pos();
        self.expect(SyntaxKind::OpenBraceToken);
        let members = self.parse_list(ParsingContext::ClassMembers, Parser::parse_class_member);
        self.expect(SyntaxKind::CloseBraceToken);
        let end = self.token_pos();
        NodeList {
            loc: TextRange::new(pos, end),
            nodes: members.nodes,
        }
    }

    pub(crate) fn parse_class_member(&mut self) -> Arc<Node> {
        let pos = self.token_pos();

        if self.token == SyntaxKind::SemicolonToken {
            let end = self.token_end();
            self.next_token();
            return Arc::new(Node::with_loc(
                SyntaxKind::SemicolonClassElement,
                NodeData::Token,
                TextRange::new(pos, end),
            ));
        }

        let mut decorators: Vec<Arc<Node>> = Vec::new();
        let mut modifiers: Vec<(SyntaxKind, usize, usize)> = Vec::new();
        loop {
            if self.token == SyntaxKind::AtToken {
                decorators.push(self.parse_decorator());
                continue;
            }
            if !matches!(
                self.token,
                SyntaxKind::PublicKeyword
                    | SyntaxKind::PrivateKeyword
                    | SyntaxKind::ProtectedKeyword
                    | SyntaxKind::StaticKeyword
                    | SyntaxKind::ReadonlyKeyword
                    | SyntaxKind::AbstractKeyword
                    | SyntaxKind::AsyncKeyword
                    | SyntaxKind::OverrideKeyword
                    | SyntaxKind::AccessorKeyword

                    | SyntaxKind::ConstKeyword

                    | SyntaxKind::ExportKeyword
                    | SyntaxKind::DefaultKeyword

                    | SyntaxKind::DeclareKeyword
            ) {
                break;
            }

            let mut s = self.scanner.clone();
            s.scan();
            if s.has_preceding_line_break() {
                break;
            }
            if self.token == SyntaxKind::StaticKeyword && s.token() == SyntaxKind::OpenBraceToken {
                break;
            }

            let can_follow = match self.token {
                SyntaxKind::ExportKeyword => {
                    let next = s.token();
                    match next {
                        SyntaxKind::DefaultKeyword => {
                            let t = s.scan();
                            Self::token_can_follow_default_keyword(t, &mut s)
                        }
                        SyntaxKind::TypeKeyword => {
                            let t = s.scan();
                            Self::can_follow_export_modifier(t)
                        }
                        _ => Self::can_follow_export_modifier(next),
                    }
                }
                SyntaxKind::DefaultKeyword => {
                    Self::token_can_follow_default_keyword(s.token(), &mut s)
                }
                _ => Self::token_can_follow_modifier(s.token()),
            };
            if !can_follow {
                break;
            }
            let kind = self.token;
            let mpos = self.token_pos();
            let mend = self.token_end();
            self.next_token();
            modifiers.push((kind, mpos, mend));
        }
        let modifiers = if decorators.is_empty() && modifiers.is_empty() {
            None
        } else if decorators.is_empty() {
            Some(self.make_modifier_list(modifiers))
        } else {
            Some(self.make_modifier_list_with_decorators(modifiers, decorators))
        };

        if self.token == SyntaxKind::StaticKeyword {

            let mut s = self.scanner.clone();
            s.scan();
            if !s.has_preceding_line_break() && s.token() == SyntaxKind::OpenBraceToken {
                let pos = self.token_pos();
                self.next_token();
                let body = self.parse_block();
                let end = body.end();
                return Arc::new(Node::with_loc(
                    SyntaxKind::ClassStaticBlockDeclaration,
                    NodeData::ClassStaticBlockDeclaration(ClassStaticBlockDeclarationData {
                        modifiers: None,
                        body,
                    }),
                    TextRange::new(pos, end),
                ));
            }
        }

        if self.is_index_signature_start() {
            return self.parse_index_signature(pos, modifiers);
        }

        if self.token == SyntaxKind::GetKeyword || self.token == SyntaxKind::SetKeyword {
            let mut s = self.scanner.clone();
            s.scan();
            let next = s.token();
            let is_accessor = Self::token_can_follow_get_or_set(next);
            if is_accessor {
                let accessor_kind = self.token;
                return self.parse_accessor_declaration(pos, modifiers, accessor_kind);
            }
        }

        let asterisk_token = self.parse_optional_token(SyntaxKind::AsteriskToken);
        let name = self.parse_property_name();
        let postfix_token = self
            .parse_optional_token(SyntaxKind::QuestionToken)
            .or_else(|| self.parse_optional_token(SyntaxKind::ExclamationToken));

        if self.token == SyntaxKind::OpenParenToken
            || self.token == SyntaxKind::LessThanToken
            || asterisk_token.is_some()
        {

            let is_constructor =
                name.kind == SyntaxKind::Identifier && name.text() == "constructor";
            let type_parameters = self.parse_optional_type_parameters();

            let prev_yield = self.yield_context;
            let prev_await = self.await_context;
            if asterisk_token.is_some() {
                self.yield_context = true;
            }

            let parameters = self.parse_parameter_list();
            let type_node = self.parse_optional_return_type();
            let body = if self.token == SyntaxKind::OpenBraceToken {
                Some(self.parse_block())
            } else {
                self.parse_semicolon();
                None
            };
            self.yield_context = prev_yield;
            self.await_context = prev_await;

            let end = body.as_ref().map_or(self.token_pos(), |b| b.end());
            if is_constructor {
                return Arc::new(Node::with_loc(
                    SyntaxKind::Constructor,
                    NodeData::ConstructorDeclaration(ConstructorDeclarationData {
                        modifiers,
                        type_parameters,
                        parameters,
                        type_node,
                        full_signature: None,
                        body,
                    }),
                    TextRange::new(pos, end),
                ));
            }
            return Arc::new(Node::with_loc(
                SyntaxKind::MethodDeclaration,
                NodeData::MethodDeclaration(MethodDeclarationData {
                    modifiers,
                    asterisk_token,
                    name,
                    postfix_token,
                    type_parameters,
                    parameters,
                    type_node,
                    full_signature: None,
                    body,
                }),
                TextRange::new(pos, end),
            ));
        }

        let type_node = self.parse_optional_type_annotation();
        let initializer = if self.token == SyntaxKind::EqualsToken {
            self.next_token();
            Some(self.parse_assignment_expression())
        } else {
            None
        };
        self.parse_semicolon_after_property_name(&name, type_node.as_ref(), initializer.as_ref());
        let end = self.token_pos();
        Arc::new(Node::with_loc(
            SyntaxKind::PropertyDeclaration,
            NodeData::PropertyDeclaration(PropertyDeclarationData {
                modifiers,
                name,
                postfix_token,
                type_node,
                initializer,
            }),
            TextRange::new(pos, end),
        ))
    }

    pub(crate) fn parse_semicolon_after_property_name(
        &mut self,
        name: &Arc<Node>,
        type_node: Option<&Arc<Node>>,
        initializer: Option<&Arc<Node>>,
    ) {
        if self.token == SyntaxKind::AtToken && !self.has_preceding_line_break() {
            self.parse_error_at_current_token(
                diagnostics::DECORATORS_MUST_PRECEDE_THE_NAME_AND_ALL_KEYWORDS_OF_PROPERTY_DECLARATIONS,
                &[],
            );
            return;
        }
        if self.token == SyntaxKind::OpenParenToken {
            self.parse_error_at_current_token(
                diagnostics::CANNOT_START_A_FUNCTION_CALL_IN_A_TYPE_ANNOTATION,
                &[],
            );
            self.next_token();
            return;
        }
        if type_node.is_some() && !self.can_parse_semicolon() {
            if initializer.is_some() {
                self.parse_error_at_current_token(diagnostics::X_0_EXPECTED, &[";"]);
            } else {
                self.parse_error_at_current_token(
                    diagnostics::EXPECTED_FOR_PROPERTY_INITIALIZER,
                    &[],
                );
            }
            return;
        }
        if self.try_parse_semicolon() {
            return;
        }
        if initializer.is_some() {
            self.parse_error_at_current_token(diagnostics::X_0_EXPECTED, &[";"]);
            return;
        }
        self.parse_error_for_missing_semicolon_after(name);
    }

    pub(crate) fn parse_error_for_missing_semicolon_after(&mut self, node: &Arc<Node>) {

        let expression_text = if node.kind == SyntaxKind::Identifier {
            node.text().to_string()
        } else {
            String::new()
        };
        if expression_text.is_empty() {
            self.parse_error_at_current_token(diagnostics::X_0_EXPECTED, &[";"]);
            return;
        }

        let pos = node.loc.pos();
        match expression_text.as_str() {
            "const" | "let" | "var" => {
                self.parse_error_at(
                    pos,
                    node.end(),
                    diagnostics::VARIABLE_DECLARATION_NOT_ALLOWED_AT_THIS_LOCATION,
                    &[],
                );
            }

            "declare" => {}
            "interface" => {
                self.parse_error_for_invalid_name(
                    diagnostics::INTERFACE_NAME_CANNOT_BE_0,
                    diagnostics::INTERFACE_MUST_BE_GIVEN_A_NAME,
                );
            }
            "is" => {
                self.parse_error_at(
                    pos,
                    self.token_pos(),
                    diagnostics::A_TYPE_PREDICATE_IS_ONLY_ALLOWED_IN_RETURN_TYPE_POSITION_FOR_FUNCTIONS_AND_METHODS,
                    &[],
                );
            }
            "module" | "namespace" => {
                self.parse_error_for_invalid_name(
                    diagnostics::NAMESPACE_NAME_CANNOT_BE_0,
                    diagnostics::NAMESPACE_MUST_BE_GIVEN_A_NAME,
                );
            }
            "type" => {
                self.parse_error_for_invalid_name(
                    diagnostics::TYPE_ALIAS_NAME_CANNOT_BE_0,
                    diagnostics::TYPE_ALIAS_MUST_BE_GIVEN_A_NAME,
                );
            }
            _ => {

                if self.token == SyntaxKind::Unknown {
                    return;
                }

                let expression_text = if node.kind == SyntaxKind::Identifier {
                    node.text().to_string()
                } else {
                    String::new()
                };
                let followed_by_identifier = {
                    let text = self.scanner.text();
                    let mut i = node.end();
                    let bytes = text.as_bytes();
                    while i < bytes.len() && (bytes[i] == b' ' || bytes[i] == b'\t') {
                        i += 1;
                    }
                    i < bytes.len()
                        && (bytes[i].is_ascii_alphabetic()
                            || bytes[i] == b'_'
                            || bytes[i] == b'$')
                };
                if !expression_text.is_empty() && followed_by_identifier {
                    let lower = expression_text.to_ascii_lowercase();
                    let mut best: Option<(usize, String)> = None;
                    for kw in KEYWORD_SUGGESTIONS {
                        if kw.len() <= 2 {
                            continue;
                        }
                        let d = crate::checker::edit_distance(
                            &lower,
                            &kw.to_ascii_lowercase(),
                        );
                        let budget = (expression_text.len() as f64 * 0.4).floor() + 0.9;
                        if d as f64 > budget {
                            continue;
                        }
                        if best.as_ref().is_none_or(|(bd, _)| d < *bd) {
                            best = Some((d, kw.to_string()));
                        }
                    }

                    let space_sugg = best.is_none().then(|| {
                        KEYWORD_SUGGESTIONS
                            .iter()
                            .find(|kw| {
                                kw.len() > 2
                                    && expression_text.len() > kw.len() + 2
                                    && expression_text.starts_with(*kw)
                            })
                            .map(|kw| format!("{kw} {}", &expression_text[kw.len()..]))
                    })
                            .flatten();
                    if let Some((_, sugg)) = best {
                        self.parse_error_at(
                            pos,
                            node.end(),
                            diagnostics::UNKNOWN_KEYWORD_OR_IDENTIFIER_DID_YOU_MEAN_0,
                            &[&sugg],
                        );
                        return;
                    }
                    if let Some(sugg) = space_sugg {
                        self.parse_error_at(
                            pos,
                            node.end(),
                            diagnostics::UNKNOWN_KEYWORD_OR_IDENTIFIER_DID_YOU_MEAN_0,
                            &[&sugg],
                        );
                        return;
                    }
                }
                self.parse_error_at(
                    pos,
                    node.end(),
                    diagnostics::UNEXPECTED_KEYWORD_OR_IDENTIFIER,
                    &[],
                );
            }
        }
    }

    pub(crate) fn parse_error_for_invalid_name(
        &mut self,
        name_diagnostic: Message,
        blank_diagnostic: Message,
    ) {
        if self.token == SyntaxKind::OpenBraceToken {
            self.parse_error_at_current_token(blank_diagnostic, &[]);
        } else {
            let arg = self.scanner.token_text().to_string();
            self.parse_error_at_current_token(name_diagnostic, &[&arg]);
        }
    }

    pub(crate) fn parse_accessor_declaration(
        &mut self,
        pos: usize,
        modifiers: Option<Arc<ModifierList>>,
        accessor_kind: SyntaxKind,
    ) -> Arc<Node> {

        self.next_token();
        let name = self.parse_property_name();
        let type_parameters = self.parse_optional_type_parameters();
        let parameters = self.parse_parameter_list();
        let type_node = self.parse_optional_return_type();
        let body = if self.token == SyntaxKind::OpenBraceToken {
            Some(self.parse_block())
        } else {
            self.parse_semicolon();
            None
        };

        let end = body
            .as_ref()
            .map_or(self.scanner.full_start_pos(), |b| b.end());
        let range = TextRange::new(pos, end);
        match accessor_kind {
            SyntaxKind::GetKeyword => Arc::new(Node::with_loc(
                SyntaxKind::GetAccessor,
                NodeData::GetAccessorDeclaration(GetAccessorDeclarationData {
                    modifiers,
                    name,
                    type_parameters,
                    parameters,
                    type_node,
                    full_signature: None,
                    body,
                }),
                range,
            )),
            _ => Arc::new(Node::with_loc(
                SyntaxKind::SetAccessor,
                NodeData::SetAccessorDeclaration(SetAccessorDeclarationData {
                    modifiers,
                    name,
                    type_parameters,
                    parameters,
                    type_node,
                    full_signature: None,
                    body,
                }),
                range,
            )),
        }
    }

    pub fn diagnostics(&self) -> &[ParserDiagnostic] {
        &self.diagnostics
    }
}
