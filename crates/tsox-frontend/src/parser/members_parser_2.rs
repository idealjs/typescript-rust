#![allow(unused_imports)]

use crate::parser::members::*;

impl Parser {
    pub(crate) fn parse_heritage_clauses(&mut self) -> Option<Arc<NodeList>> {
        self.parse_heritage_clauses_is_interface(false)
    }

    /// Go parseHeritageClauses(isInterface)：interface 的 extends 元素经
    /// parseTypeHeritageClauseElement 产出 TypeReferenceNode（`string` 等
    /// 关键字类型名合法，语义层 TS2312 报「只能扩展对象类型」），class 的
    /// extends / implements 走表达式形态
    pub(crate) fn parse_heritage_clauses_is_interface(
        &mut self,
        is_interface: bool,
    ) -> Option<Arc<NodeList>> {
        let mut clauses = Vec::new();
        while matches!(
            self.token,
            SyntaxKind::ExtendsKeyword | SyntaxKind::ImplementsKeyword
        ) {
            let pos = self.token_pos();
            let kind = self.token;
            self.next_token();
            let element = if is_interface && kind == SyntaxKind::ExtendsKeyword {
                Parser::parse_type_heritage_clause_element
            } else {
                Parser::parse_heritage_clause_element
            };
            let types =
                self.parse_delimited_list(ParsingContext::HeritageClauseElement, element);
            let end = self.node_pos();
            clauses.push(Arc::new(Node::with_loc(
                SyntaxKind::HeritageClause,
                NodeData::HeritageClause(HeritageClauseData {
                    token: kind,
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

    /// Go parseTypeHeritageClauseElement：表达式形态的实体名转类型引用
    ///（KeywordType 亦可成名），无实参时直接产出 TypeReferenceNode
    pub(crate) fn parse_type_heritage_clause_element(&mut self) -> Arc<Node> {
        let element = self.parse_heritage_clause_element();
        let (expression, type_arguments) = match &element.data {
            NodeData::ExpressionWithTypeArguments(d) => {
                (Arc::clone(&d.expression), d.type_arguments.clone())
            }
            _ => return element,
        };
        let type_name = convert_entity_name_expression_to_entity_name(&expression);
        let pos = element.pos();
        let end = element.end();
        Arc::new(Node::with_loc(
            SyntaxKind::TypeReference,
            NodeData::TypeReferenceNode(TypeReferenceNodeData {
                type_name,
                type_arguments,
            }),
            TextRange::new(pos, end),
        ))
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
            let end = self.node_pos();
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
        let end = self.node_pos();
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
                | SyntaxKind::AbstractKeyword
                | SyntaxKind::OverrideKeyword
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

    /// Go scanTypeMemberStart：类型成员起始前瞻（isListElement 的
    /// TypeMembers 判定）。非成员 token（`{`、`=>`、jsdoc 杂文等）返回
    /// false，交给 abortParsingListOrMoveToNextToken 强制进展，否则
    /// parse_type_member 对不可成名 token 无消费循环
    pub(crate) fn look_ahead_type_member_start(&self) -> bool {
        if matches!(
            self.token,
            SyntaxKind::OpenParenToken
                | SyntaxKind::LessThanToken
                | SyntaxKind::GetKeyword
                | SyntaxKind::SetKeyword
        ) {
            return true;
        }
        let mut s = self.scanner.clone();
        let mut t = self.token;
        let mut id_token = false;
        while is_modifier_kind(t) {
            id_token = true;
            t = s.scan();
        }
        if t == SyntaxKind::OpenBracketToken {
            return true;
        }
        if is_identifier_or_keyword(t)
            || matches!(
                t,
                SyntaxKind::StringLiteral
                    | SyntaxKind::NumericLiteral
                    | SyntaxKind::BigIntLiteral
                    | SyntaxKind::PrivateIdentifier
            )
        {
            id_token = true;
            t = s.scan();
        }
        if id_token {
            return matches!(
                t,
                SyntaxKind::OpenParenToken
                    | SyntaxKind::LessThanToken
                    | SyntaxKind::QuestionToken
                    | SyntaxKind::ColonToken
                    | SyntaxKind::CommaToken
                    | SyntaxKind::SemicolonToken
                    | SyntaxKind::CloseBraceToken
                    | SyntaxKind::EndOfFile
            ) || s.has_preceding_line_break();
        }
        false
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
}

/// Go convertEntityNameExpressionToEntityName：表达式形态实体名（Identifier/
/// PropertyAccessExpression）转限定名（QualifiedName）
pub(crate) fn convert_entity_name_expression_to_entity_name(node: &Arc<Node>) -> Arc<Node> {
    if node.kind == SyntaxKind::Identifier {
        return Arc::clone(node);
    }
    if let NodeData::PropertyAccessExpression(pa) = &node.data {
        let left = convert_entity_name_expression_to_entity_name(&pa.expression);
        let end = node.end();
        return Arc::new(Node::with_loc(
            SyntaxKind::QualifiedName,
            NodeData::QualifiedName(QualifiedNameData {
                left,
                right: Arc::clone(&pa.name),
            }),
            TextRange::new(node.pos(), end),
        ));
    }
    Arc::clone(node)
}
