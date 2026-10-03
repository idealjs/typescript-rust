#![allow(unused_imports, dead_code)]

use std::sync::Arc;

use tsox_core::core::text::TextRange;
use tsox_frontend::ast::node::Node;
use tsox_frontend::ast::node_data_generated::{
    BinaryExpressionData, NodeData, SyntaxListData, TypeOfExpressionData,
};
use tsox_frontend::ast::syntax_kind_generated::SyntaxKind;
use tsox_frontend::ast::{ModifierList, NodeList};
use tsox_frontend::format::mig::m4o::ListFormat;

use crate::mig::m4p::Printer;
use crate::mig::m4r::EmitTextWriter;
use crate::printer::generated_identifier_flags::NodeFactory;

pub const HERITAGE_CLAUSES: ListFormat = ListFormat(ListFormat::SPACE_BETWEEN_SIBLINGS.0);
pub const SINGLE_LINE_FUNCTION_BODY_STATEMENTS: ListFormat =
    ListFormat(ListFormat::SPACE_BETWEEN_SIBLINGS.0 | ListFormat::SPACE_BETWEEN_BRACES.0);
pub const MULTI_LINE_FUNCTION_BODY_STATEMENTS: ListFormat = ListFormat(ListFormat::MULTI_LINE.0);
pub const INTERFACE_MEMBERS: ListFormat =
    ListFormat(ListFormat::INDENTED.0 | ListFormat::MULTI_LINE.0);
pub const ENUM_MEMBERS: ListFormat = ListFormat(
    ListFormat::COMMA_DELIMITED.0 | ListFormat::INDENTED.0 | ListFormat::MULTI_LINE.0,
);

impl<'a> NodeFactory<'a> {
    pub fn new_omitted_expression(&self) -> Arc<Node> { ::tsox_core::fntrace::enter("new_omitted_expression"); 
        Arc::new(Node::new(SyntaxKind::OmittedExpression, NodeData::OmittedExpression))
    }

    pub fn new_type_of_expression(&self, expression: &Arc<Node>) -> Arc<Node> { ::tsox_core::fntrace::enter("new_type_of_expression"); 
        Arc::new(Node::new(
            SyntaxKind::TypeOfExpression,
            NodeData::TypeOfExpression(TypeOfExpressionData {
                expression: expression.clone(),
            }),
        ))
    }

    pub fn new_strict_inequality_expression(
        &self,
        left: &Arc<Node>,
        right: &Arc<Node>,
    ) -> Arc<Node> { ::tsox_core::fntrace::enter("new_strict_inequality_expression"); 
        self.new_binary_expression(
            None,
            left,
            None,
            &self.new_token(SyntaxKind::ExclamationEqualsEqualsToken),
            right,
        )
    }

    pub fn new_logical_and_expression(&self, left: &Arc<Node>, right: &Arc<Node>) -> Arc<Node> { ::tsox_core::fntrace::enter("new_logical_and_expression"); 
        self.new_binary_expression(
            None,
            left,
            None,
            &self.new_token(SyntaxKind::AmpersandAmpersandToken),
            right,
        )
    }

    pub fn new_shorthand_property_assignment(
        &self,
        modifiers: Option<Arc<ModifierList>>,
        name: &Arc<Node>,
        postfix_token: Option<&Arc<Node>>,
        type_node: Option<&Arc<Node>>,
        equals_token: Option<&Arc<Node>>,
        object_assignment_initializer: Option<Arc<Node>>,
    ) -> Arc<Node> { ::tsox_core::fntrace::enter("new_shorthand_property_assignment"); 
        let end = name.loc.end();
        let type_node = type_node.cloned().unwrap_or_else(|| {
            Arc::new(Node::with_loc(
                SyntaxKind::Unknown,
                NodeData::Token,
                TextRange::new(end, end),
            ))
        });
        Arc::new(Node::new(
            SyntaxKind::ShorthandPropertyAssignment,
            NodeData::ShorthandPropertyAssignment(
                tsox_frontend::ast::node_data_generated::ShorthandPropertyAssignmentData {
                    modifiers,
                    name: name.clone(),
                    postfix_token: postfix_token.cloned(),
                    type_node,
                    equals_token: equals_token.cloned(),
                    object_assignment_initializer,
                },
            ),
        ))
    }
}
