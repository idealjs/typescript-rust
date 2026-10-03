#![allow(unused_imports)]

use crate::parser::expressions::*;

impl Parser {
    pub(crate) fn parse_object_literal_element_modifiers(&mut self) -> Option<Arc<ModifierList>> { ::tsox_core::fntrace::enter("parse_object_literal_element_modifiers"); 
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
                    | SyntaxKind::DeclareKeyword
            ) {
                break;
            }
            let mut s = self.scanner.clone();
            s.scan();
            if s.has_preceding_line_break()
                || !Self::token_can_follow_modifier(s.token())
                || s.token() == SyntaxKind::OpenParenToken
                || s.token() == SyntaxKind::LessThanToken
            {
                break;
            }
            let kind = self.token;
            let pos = self.token_pos();
            let end = self.token_end();
            self.next_token();
            modifiers.push((kind, pos, end));
        }
        if decorators.is_empty() && modifiers.is_empty() {
            None
        } else if decorators.is_empty() {
            Some(self.make_modifier_list(modifiers))
        } else {
            Some(self.make_modifier_list_with_decorators(modifiers, decorators))
        }
    }

    pub(crate) fn parse_decorated_expression(&mut self) -> Arc<Node> { ::tsox_core::fntrace::enter("parse_decorated_expression");
        let pos = self.node_pos();
        let modifiers = self.parse_expression_modifiers();
        if self.token == SyntaxKind::ClassKeyword {
            return self.parse_class_expression_with_modifiers(pos, modifiers);
        }
        let end = self.node_pos();
        self.parse_error_at(end, end, tsox_core::diagnostics::EXPRESSION_EXPECTED, &[]);
        Arc::new(Node::with_loc(
            SyntaxKind::MissingDeclaration,
            NodeData::MissingDeclaration(MissingDeclarationData { modifiers }),
            TextRange::new(pos, end),
        ))
    }

    fn parse_expression_modifiers(&mut self) -> Option<Arc<ModifierList>> { ::tsox_core::fntrace::enter("parse_expression_modifiers");
        let mut decorators: Vec<Arc<Node>> = Vec::new();
        let mut modifiers: Vec<(SyntaxKind, usize, usize)> = Vec::new();
        let mut has_leading_modifier = false;
        let mut has_trailing_decorator = false;
        let mut has_trailing_modifier = false;
        let mut has_static_modifier = false;
        loop {
            if self.token == SyntaxKind::AtToken && !has_trailing_modifier {
                decorators.push(self.parse_decorator());
                if has_leading_modifier {
                    has_trailing_decorator = true;
                }
                continue;
            }
            match self.try_parse_expression_modifier(has_static_modifier) {
                Some((kind, pos, end)) => {
                    if kind == SyntaxKind::StaticKeyword {
                        has_static_modifier = true;
                    }
                    modifiers.push((kind, pos, end));
                    if has_trailing_decorator {
                        has_trailing_modifier = true;
                    } else {
                        has_leading_modifier = true;
                    }
                }
                None => break,
            }
        }
        if decorators.is_empty() && modifiers.is_empty() {
            None
        } else if decorators.is_empty() {
            Some(self.make_modifier_list(modifiers))
        } else {
            Some(self.make_modifier_list_with_decorators(modifiers, decorators))
        }
    }

    fn try_parse_expression_modifier(
        &mut self,
        has_seen_static: bool,
    ) -> Option<(SyntaxKind, usize, usize)> { ::tsox_core::fntrace::enter("try_parse_expression_modifier");
        if has_seen_static && self.token == SyntaxKind::StaticKeyword {
            return None;
        }
        if !matches!(
            self.token,
            SyntaxKind::ExportKeyword
                | SyntaxKind::DeclareKeyword
                | SyntaxKind::DefaultKeyword
                | SyntaxKind::AbstractKeyword
                | SyntaxKind::AsyncKeyword
                | SyntaxKind::ReadonlyKeyword
                | SyntaxKind::PublicKeyword
                | SyntaxKind::PrivateKeyword
                | SyntaxKind::ProtectedKeyword
                | SyntaxKind::StaticKeyword
                | SyntaxKind::ConstKeyword
                | SyntaxKind::AccessorKeyword
                | SyntaxKind::OverrideKeyword
        ) {
            return None;
        }
        let mut s = self.scanner.clone();
        s.scan();
        let can_follow = if self.token == SyntaxKind::ConstKeyword {
            s.token() == SyntaxKind::EnumKeyword
        } else {
            !s.has_preceding_line_break() && Self::token_can_follow_modifier(s.token())
        };
        if !can_follow {
            return None;
        }
        let kind = self.token;
        let pos = self.token_pos();
        let end = self.token_end();
        self.next_token();
        Some((kind, pos, end))
    }
}
