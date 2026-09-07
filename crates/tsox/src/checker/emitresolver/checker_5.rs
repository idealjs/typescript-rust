#![allow(unused_imports)]

use super::*;

impl Checker {
    pub fn is_optional_parameter(&self, node: &Arc<Node>) -> bool {
        match &node.data {
            NodeData::ParameterDeclaration(data) => {
                data.question_token.is_some() || node.kind == SyntaxKind::RestType
            }
            _ => false,
        }
    }

    pub fn is_literal_const_declaration(&self, node: &Arc<Node>) -> bool {
        if node.kind != SyntaxKind::VariableDeclaration {
            return false;
        }
        let NodeData::VariableDeclaration(data) = &node.data else {
            return false;
        };

        if data.initializer.is_none() {
            return false;
        }
        let initializer = data.initializer.as_ref().unwrap();
        matches!(
            initializer.kind,
            SyntaxKind::StringLiteral
                | SyntaxKind::NumericLiteral
                | SyntaxKind::TrueKeyword
                | SyntaxKind::FalseKeyword
                | SyntaxKind::BigIntLiteral
                | SyntaxKind::PrefixUnaryExpression
        )
    }

    pub fn get_constant_value(&mut self, node: &Arc<Node>) -> Option<String> {
        if node.kind == SyntaxKind::EnumMember {
            return self.get_enum_member_value_string(node);
        }
        match node.kind {
            SyntaxKind::StringLiteral => {
                if let NodeData::StringLiteral(s) = &node.data {
                    Some(format!("\"{}\"", s.text))
                } else {
                    None
                }
            }
            SyntaxKind::NumericLiteral => {
                if let NodeData::NumericLiteral(n) = &node.data {
                    Some(n.text.clone())
                } else {
                    None
                }
            }
            SyntaxKind::TrueKeyword => Some("true".to_string()),
            SyntaxKind::FalseKeyword => Some("false".to_string()),
            SyntaxKind::NullKeyword => Some("null".to_string()),
            _ => None,
        }
    }

    pub fn is_referenced_alias_declaration(&self, node: &Arc<Node>) -> bool {
        if let Some(links) = self.declaration_links.get(node) {
            if links.is_visible.is_true() {
                return true;
            }
        }

        true
    }

    pub fn is_value_alias_declaration(&self, node: &Arc<Node>) -> bool {
        match &node.data {
            NodeData::ImportSpecifier(data) => !data.is_type_only,
            _ => true,
        }
    }

    pub fn get_effective_declaration_flags(&self, node: &Arc<Node>) -> u32 {
        node.syntactic_modifier_flags().bits()
    }

    pub fn get_symbol_of_declaration(&self, node: &Arc<Node>) -> Option<Arc<Symbol>> {
        self.program.symbol_map().symbol_of(node).cloned()
    }

    pub fn is_const_enum_member(&self, symbol: &Symbol) -> bool {
        symbol.flags.contains(SymbolFlags::ConstEnum)
    }
}

pub(crate) fn collect_children(node: &Arc<Node>) -> Vec<Arc<Node>> {
    let mut children: Vec<Arc<Node>> = Vec::new();
    for_each_child(node, |child| {
        children.push(Arc::clone(child));
        false
    });
    children
}
