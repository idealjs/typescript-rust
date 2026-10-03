#![allow(unused_imports)]

use crate::parser::types::*;

impl Parser {
    pub(crate) fn parse_function_type(&mut self) -> Arc<Node> { ::tsox_core::fntrace::enter("parse_function_type"); 
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

    pub(crate) fn parse_constructor_type(&mut self) -> Arc<Node> { ::tsox_core::fntrace::enter("parse_constructor_type"); 
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

    pub(crate) fn parse_function_or_constructor_type_to_error(
        &mut self,
        is_union_type: bool,
        parse_constituent: fn(&mut Self) -> Arc<Node>,
    ) -> Arc<Node> { ::tsox_core::fntrace::enter("parse_function_or_constructor_type_to_error"); 
        if !self.is_start_of_function_type_or_constructor_type() {
            return parse_constituent(self);
        }
        let type_node = match self.token {
            SyntaxKind::NewKeyword | SyntaxKind::AbstractKeyword => self.parse_constructor_type(),
            _ => self.parse_function_type(),
        };
        let message = match (type_node.kind, is_union_type) {
            (SyntaxKind::FunctionType, true) => {
                tsox_core::diagnostics::messages_generated::FUNCTION_TYPE_NOTATION_MUST_BE_PARENTHESIZED_WHEN_USED_IN_A_UNION_TYPE
            }
            (SyntaxKind::FunctionType, false) => {
                tsox_core::diagnostics::messages_generated::FUNCTION_TYPE_NOTATION_MUST_BE_PARENTHESIZED_WHEN_USED_IN_AN_INTERSECTION_TYPE
            }
            (_, true) => {
                tsox_core::diagnostics::messages_generated::CONSTRUCTOR_TYPE_NOTATION_MUST_BE_PARENTHESIZED_WHEN_USED_IN_A_UNION_TYPE
            }
            (_, false) => {
                tsox_core::diagnostics::messages_generated::CONSTRUCTOR_TYPE_NOTATION_MUST_BE_PARENTHESIZED_WHEN_USED_IN_AN_INTERSECTION_TYPE
            }
        };
        self.parse_error_at_range(type_node.loc, message, &[]);
        type_node
    }
}
