#![allow(unused_imports, dead_code)]

use std::sync::Arc;

use tsox_frontend::ast::node::Node;
use tsox_frontend::ast::NodeList;
use tsox_frontend::ast::node_data_generated as ndg;
use tsox_frontend::ast::node_data_generated::NodeData;
use tsox_frontend::ast::syntax_kind_generated::SyntaxKind;
use tsox_frontend::scanner::TOKEN_FLAGS_NONE;

use crate::printer::generated_identifier_flags::NodeFactory;

impl<'a> NodeFactory<'a> {
    pub fn new_not_emitted_statement(&self) -> Arc<Node> { ::tsox_core::fntrace::enter("new_not_emitted_statement"); 
        Arc::new(Node::new(
            SyntaxKind::NotEmittedStatement,
            NodeData::NotEmittedStatement,
        ))
    }

    pub fn new_conditional_expression(
        &self,
        condition: &Arc<Node>,
        question_token: &Arc<Node>,
        when_true: &Arc<Node>,
        colon_token: &Arc<Node>,
        when_false: &Arc<Node>,
    ) -> Arc<Node> { ::tsox_core::fntrace::enter("new_conditional_expression"); 
        Arc::new(Node::new(
            SyntaxKind::ConditionalExpression,
            NodeData::ConditionalExpression(ndg::ConditionalExpressionData {
                condition: condition.clone(),
                question_token: question_token.clone(),
                when_true: when_true.clone(),
                colon_token: colon_token.clone(),
                when_false: when_false.clone(),
            }),
        ))
    }

    pub fn new_strict_equality_expression(&self, left: &Arc<Node>, right: &Arc<Node>) -> Arc<Node> { ::tsox_core::fntrace::enter("new_strict_equality_expression"); 
        self.new_binary_expression(
            None,
            left,
            None,
            &self.new_token(SyntaxKind::EqualsEqualsEqualsToken),
            right,
        )
    }

    pub fn new_type_check(&self, value: &Arc<Node>, tag: &str) -> Arc<Node> { ::tsox_core::fntrace::enter("new_type_check"); 
        if tag == "null" {
            self.new_strict_equality_expression(value, &self.new_keyword_expression(SyntaxKind::NullKeyword))
        } else if tag == "undefined" {
            self.new_strict_equality_expression(value, &self.new_void_zero_expression())
        } else {
            self.new_strict_equality_expression(
                &self.new_type_of_expression(value),
                &self.new_string_literal(tag, TOKEN_FLAGS_NONE),
            )
        }
    }

    pub fn update_yield_expression(
        &self,
        node: &Arc<Node>,
        asterisk_token: Option<&Arc<Node>>,
        expression: Option<&Arc<Node>>,
    ) -> Arc<Node> { ::tsox_core::fntrace::enter("update_yield_expression"); 
        let mut updated = Node::new(
            SyntaxKind::YieldExpression,
            NodeData::YieldExpression(ndg::YieldExpressionData {
                asterisk_token: asterisk_token.cloned(),
                expression: expression.cloned(),
            }),
        );
        updated.loc = node.loc;
        Arc::new(updated)
    }

    pub fn update_return_statement(
        &self,
        node: &Arc<Node>,
        expression: Option<&Arc<Node>>,
    ) -> Arc<Node> { ::tsox_core::fntrace::enter("update_return_statement"); 
        let mut updated = Node::new(
            SyntaxKind::ReturnStatement,
            NodeData::ReturnStatement(ndg::ReturnStatementData {
                expression: expression.cloned(),
            }),
        );
        updated.loc = node.loc;
        Arc::new(updated)
    }

    pub fn update_labeled_statement(
        &self,
        node: &Arc<Node>,
        label: &Arc<Node>,
        statement: &Arc<Node>,
    ) -> Arc<Node> { ::tsox_core::fntrace::enter("update_labeled_statement"); 
        let mut updated = Node::new(
            SyntaxKind::LabeledStatement,
            NodeData::LabeledStatement(ndg::LabeledStatementData {
                label: label.clone(),
                statement: statement.clone(),
            }),
        );
        updated.loc = node.loc;
        Arc::new(updated)
    }

    pub fn restore_enclosing_label(
        &self,
        node: &Arc<Node>,
        outermost_labeled_statement: Option<&Arc<Node>>,
    ) -> Arc<Node> { ::tsox_core::fntrace::enter("restore_enclosing_label"); 
        let Some(outermost_labeled_statement) = outermost_labeled_statement else {
            return node.clone();
        };
        let (outer_label, outer_statement) = labeled_statement_parts(outermost_labeled_statement);
        let inner_label =
            if tsox_frontend::ast::node_data_generated::is_labeled_statement(outer_statement) {
                self.restore_enclosing_label(node, Some(outer_statement))
            } else {
                node.clone()
            };
        self.update_labeled_statement(outermost_labeled_statement, outer_label, &inner_label)
    }
}

fn labeled_statement_parts(node: &Arc<Node>) -> (&Arc<Node>, &Arc<Node>) { ::tsox_core::fntrace::enter("labeled_statement_parts"); 
    match &node.data {
        NodeData::LabeledStatement(d) => (&d.label, &d.statement),
        _ => panic!("unexpected LabeledStatement: {:?}", node.kind),
    }
}
