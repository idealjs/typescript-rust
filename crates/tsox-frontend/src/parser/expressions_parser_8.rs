#![allow(unused_imports)]

use crate::parser::expressions::*;

impl Parser {
    pub(crate) fn parse_object_literal_element_modifiers(&mut self) -> Option<Arc<ModifierList>> {
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
}
