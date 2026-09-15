#![allow(unused_imports)]

use crate::parser::members::*;

impl Parser {
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
        let end = self.node_pos();
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
        // Go shouldParseReturnType(isType=true)：类型位返回型分隔符易把
        // ':' 写反成 '=>'，报 ':' expected 后照常取类型（JSDoc 调用签名
        // `() => string`）
        let type_node = if self.parse_optional(SyntaxKind::ColonToken) {
            Some(self.parse_type_or_type_predicate())
        } else if self.token == SyntaxKind::EqualsGreaterThanToken {
            self.parse_error_at_current_token(
                tsox_core::diagnostics::X_0_EXPECTED,
                &[crate::parser::token_to_string(SyntaxKind::ColonToken)],
            );
            self.next_token();
            Some(self.parse_type_or_type_predicate())
        } else {
            None
        };
        self.parse_type_member_semicolon();
        let end = self.node_pos();
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
        // Go parseClassDeclaration：缺 '{' 时不解析类体，成员为缺失列表
        //（后续语句不被类体吞掉）
        if self.expect_report(SyntaxKind::OpenBraceToken) {
            let members = self.parse_list(ParsingContext::ClassMembers, Parser::parse_class_member);
            self.expect(SyntaxKind::CloseBraceToken);
            members
        } else {
            NodeList::new(Vec::new())
        }
    }
}
