#![allow(unused_imports)]

use super::*;

impl Parser {
    pub(crate) fn parse_function_type(&mut self) -> Arc<Node> {
        let pos = self.token_pos();
        let type_parameters = self.parse_optional_type_parameters();
        let parameters = self.parse_parameter_list();
        self.expect(SyntaxKind::EqualsGreaterThanToken);
        let type_node = if self.is_start_of_type() {
            Some(self.parse_type_or_type_predicate())
        } else {
            None
        };
        let end = type_node.as_ref().map_or(self.token_pos(), |n| n.end());
        Arc::new(Node::with_loc(
            SyntaxKind::FunctionType,
            NodeData::FunctionTypeNode(FunctionTypeNodeData {
                type_parameters,
                parameters,
                type_node,
            }),
            TextRange::new(pos, end),
        ))
    }

    pub(crate) fn parse_constructor_type(&mut self) -> Arc<Node> {
        let pos = self.token_pos();
        let modifiers = if self.token == SyntaxKind::AbstractKeyword {
            let modifier_pos = self.token_pos();
            let modifier_end = self.token_end();
            self.next_token();
            Some(self.make_modifier_list(vec![(
                SyntaxKind::AbstractKeyword,
                modifier_pos,
                modifier_end,
            )]))
        } else {
            None
        };
        self.expect(SyntaxKind::NewKeyword);
        let type_parameters = self.parse_optional_type_parameters();
        let parameters = self.parse_parameter_list();
        self.expect(SyntaxKind::EqualsGreaterThanToken);
        let type_node = if self.is_start_of_type() {
            Some(self.parse_type_or_type_predicate())
        } else {
            None
        };
        let end = type_node.as_ref().map_or(self.token_pos(), |n| n.end());
        Arc::new(Node::with_loc(
            SyntaxKind::ConstructorType,
            NodeData::ConstructorTypeNode(ConstructorTypeNodeData {
                modifiers,
                type_parameters,
                parameters,
                type_node,
            }),
            TextRange::new(pos, end),
        ))
    }
}
