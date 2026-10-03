#![allow(unused_imports)]

use crate::parser::types::*;

impl Parser {
    pub(crate) fn parse_postfix_type_or_higher(&mut self) -> Arc<Node> { ::tsox_core::fntrace::enter("parse_postfix_type_or_higher"); 
        let pos = self.token_pos();
        let mut type_node = self.parse_non_array_type();
        while !self.has_preceding_line_break() {
            match self.token {
                SyntaxKind::QuestionToken => {
                    if self.look_ahead(Parser::next_is_start_of_type) {
                        return type_node;
                    }
                    self.next_token();
                    let end = self.node_pos();
                    type_node = Arc::new(Node::with_loc(
                        SyntaxKind::JSDocNullableType,
                        NodeData::JSDocNullableType(JSDocNullableTypeData { type_node }),
                        TextRange::new(pos, end),
                    ));
                }
                SyntaxKind::OpenBracketToken => {
                    self.next_token();
                    if self.token == SyntaxKind::CloseBracketToken {
                        self.next_token();
                        let end = self.node_pos();
                        type_node = Arc::new(Node::with_loc(
                            SyntaxKind::ArrayType,
                            NodeData::ArrayTypeNode(ArrayTypeNodeData {
                                element_type: type_node,
                            }),
                            TextRange::new(pos, end),
                        ));
                        continue;
                    }
                    let index_type = self.parse_type();
                    self.expect(SyntaxKind::CloseBracketToken);
                    let end = self.node_pos();
                    type_node = Arc::new(Node::with_loc(
                        SyntaxKind::IndexedAccessType,
                        NodeData::IndexedAccessTypeNode(IndexedAccessTypeNodeData {
                            object_type: type_node,
                            index_type,
                        }),
                        TextRange::new(pos, end),
                    ));
                }
                _ => return type_node,
            }
        }
        type_node
    }

    pub(crate) fn parse_jsdoc_nullable_type(&mut self) -> Arc<Node> { ::tsox_core::fntrace::enter("parse_jsdoc_nullable_type"); 
        let pos = self.token_pos();
        self.next_token();
        let type_node = self.parse_type_operator_or_higher();
        let end = type_node.end();
        Arc::new(Node::with_loc(
            SyntaxKind::JSDocNullableType,
            NodeData::JSDocNullableType(JSDocNullableTypeData { type_node }),
            TextRange::new(pos, end),
        ))
    }
}
