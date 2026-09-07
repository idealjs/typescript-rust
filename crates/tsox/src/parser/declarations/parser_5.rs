#![allow(unused_imports)]

use super::*;

impl Parser {
    pub(crate) fn parse_export_specifier(&mut self) -> Arc<Node> {
        let pos = self.token_pos();
        let (is_type_only, property_name, name) = self.parse_import_or_export_specifier(false);
        self.parse_optional(SyntaxKind::CommaToken);
        let end = name.end();
        Arc::new(Node::with_loc(
            SyntaxKind::ExportSpecifier,
            NodeData::ExportSpecifier(ExportSpecifierData {
                is_type_only,
                property_name,
                name,
            }),
            TextRange::new(pos, end),
        ))
    }

    pub(crate) fn parse_enum_member(&mut self) -> Arc<Node> {
        let pos = self.token_pos();
        let name = self.parse_property_name();
        let initializer = if self.token == SyntaxKind::EqualsToken {
            self.next_token();
            Some(self.parse_assignment_expression())
        } else {
            None
        };

        let end = initializer.as_ref().map_or(name.end(), |i| i.end());
        Arc::new(Node::with_loc(
            SyntaxKind::EnumMember,
            NodeData::EnumMember(EnumMemberData { name, initializer }),
            TextRange::new(pos, end),
        ))
    }
}
