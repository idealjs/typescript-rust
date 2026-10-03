#![allow(dead_code, unused_imports, unused_variables)]

use std::sync::Arc;

use tsox_frontend::ast::node::Node;
use tsox_frontend::ast::node_data_generated as ndg;
use tsox_frontend::ast::node_data_generated::NodeData;
use tsox_frontend::ast::node_data_generated::{
    BlockData, ClassDeclarationData, DoStatementData, ForStatementData, FunctionDeclarationData,
    IfStatementData, LabeledStatementData, ShorthandPropertyAssignmentData, SwitchStatementData,
    VariableStatementData, WhileStatementData, WithStatementData,
};
use tsox_frontend::ast::node_flags::NodeFlags;
use tsox_frontend::ast::node::NodeList;
use tsox_frontend::ast::syntax_kind_generated::SyntaxKind;

use crate::mig::r33k6_shim::Visitor;
use crate::printer::NodeFactory;

pub trait R37K2NodeExt {
    fn as_function_declaration(&self) -> &FunctionDeclarationData;
    fn as_class_declaration(&self) -> &ClassDeclarationData;
    fn as_labeled_statement(&self) -> &LabeledStatementData;
    fn as_do_statement(&self) -> &DoStatementData;
    fn as_while_statement(&self) -> &WhileStatementData;
    fn as_with_statement(&self) -> &WithStatementData;
    fn as_if_statement(&self) -> &IfStatementData;
    fn as_switch_statement_data(&self);
    fn as_for_statement(&self) -> &ndg::ForStatementData;
    fn as_block(&self) -> &BlockData;
    fn as_shorthand_property_assignment(&self) -> &ShorthandPropertyAssignmentData;
}

macro_rules! node_data_accessor {
    ($name:ident, $variant:ident, $data:ident, $kind:expr) => {
        fn $name(&self) -> &$data {
            match &self.data {
                NodeData::$variant(d) => d,
                _ => panic!(concat!(stringify!($name), "() on {:?}"), self.kind),
            }
        }
    };
}

impl R37K2NodeExt for Node {
    node_data_accessor!(as_function_declaration, FunctionDeclaration, FunctionDeclarationData, SyntaxKind::FunctionDeclaration);
    node_data_accessor!(as_class_declaration, ClassDeclaration, ClassDeclarationData, SyntaxKind::ClassDeclaration);
    node_data_accessor!(as_labeled_statement, LabeledStatement, LabeledStatementData, SyntaxKind::LabeledStatement);
    node_data_accessor!(as_do_statement, DoStatement, DoStatementData, SyntaxKind::DoStatement);
    node_data_accessor!(as_while_statement, WhileStatement, WhileStatementData, SyntaxKind::WhileStatement);
    node_data_accessor!(as_with_statement, WithStatement, WithStatementData, SyntaxKind::WithStatement);
    node_data_accessor!(as_if_statement, IfStatement, IfStatementData, SyntaxKind::IfStatement);
    node_data_accessor!(as_for_statement, ForStatement, ForStatementData, SyntaxKind::ForStatement);
    node_data_accessor!(as_block, Block, BlockData, SyntaxKind::Block);
    node_data_accessor!(as_shorthand_property_assignment, ShorthandPropertyAssignment, ShorthandPropertyAssignmentData, SyntaxKind::ShorthandPropertyAssignment);

    fn as_switch_statement_data(&self) { ::tsox_core::fntrace::enter("as_switch_statement_data"); }
}

impl Visitor {
    pub fn visit_modifiers(
        &mut self,
        modifiers: Option<tsox_frontend::ast::node::ModifierList>,
    ) -> Option<Arc<tsox_frontend::ast::node::ModifierList>> { ::tsox_core::fntrace::enter("visit_modifiers"); 
        modifiers.map(|m| Arc::new(m))
    }
}

pub trait R37K2NodeVisitorExt {
    fn visit_each_child(&mut self, node: Arc<Node>) -> Option<Arc<Node>>;
    fn visit_embedded_statement(&mut self, node: Arc<Node>) -> Option<Arc<Node>>;
}

impl R37K2NodeVisitorExt for tsox_frontend::ast::visitor::NodeVisitor {
    fn visit_each_child(&mut self, node: Arc<Node>) -> Option<Arc<Node>> { ::tsox_core::fntrace::enter("visit_each_child"); 
        Some(node)
    }

    fn visit_embedded_statement(&mut self, node: Arc<Node>) -> Option<Arc<Node>> { ::tsox_core::fntrace::enter("visit_embedded_statement"); 
        Some(node)
    }
}

impl<'a> NodeFactory<'a> {
    pub fn new_empty_statement(&self) -> Arc<Node> { ::tsox_core::fntrace::enter("new_empty_statement"); 
        Arc::new(Node::new(SyntaxKind::EmptyStatement, NodeData::EmptyStatement))
    }

    pub fn update_do_statement(
        &self,
        node: &Arc<Node>,
        statement: &Arc<Node>,
        expression: &Arc<Node>,
    ) -> Arc<Node> { ::tsox_core::fntrace::enter("update_do_statement"); 
        let mut updated = Node::new(
            SyntaxKind::DoStatement,
            NodeData::DoStatement(ndg::DoStatementData {
                statement: statement.clone(),
                expression: expression.clone(),
            }),
        );
        updated.loc = node.loc;
        updated.flags = node.flags;
        Arc::new(updated)
    }

    pub fn update_while_statement(
        &self,
        node: &Arc<Node>,
        expression: &Arc<Node>,
        statement: &Arc<Node>,
    ) -> Arc<Node> { ::tsox_core::fntrace::enter("update_while_statement"); 
        let mut updated = Node::new(
            SyntaxKind::WhileStatement,
            NodeData::WhileStatement(ndg::WhileStatementData {
                expression: expression.clone(),
                statement: statement.clone(),
            }),
        );
        updated.loc = node.loc;
        updated.flags = node.flags;
        Arc::new(updated)
    }

    pub fn update_with_statement(
        &self,
        node: &Arc<Node>,
        expression: &Arc<Node>,
        statement: &Arc<Node>,
    ) -> Arc<Node> { ::tsox_core::fntrace::enter("update_with_statement"); 
        let mut updated = Node::new(
            SyntaxKind::WithStatement,
            NodeData::WithStatement(ndg::WithStatementData {
                expression: expression.clone(),
                statement: statement.clone(),
            }),
        );
        updated.loc = node.loc;
        updated.flags = node.flags;
        Arc::new(updated)
    }

    pub fn update_if_statement(
        &self,
        node: &Arc<Node>,
        expression: &Arc<Node>,
        then_statement: &Arc<Node>,
        else_statement: Option<&Arc<Node>>,
    ) -> Arc<Node> { ::tsox_core::fntrace::enter("update_if_statement"); 
        let mut updated = Node::new(
            SyntaxKind::IfStatement,
            NodeData::IfStatement(ndg::IfStatementData {
                expression: expression.clone(),
                then_statement: then_statement.clone(),
                else_statement: else_statement.cloned(),
            }),
        );
        updated.loc = node.loc;
        updated.flags = node.flags;
        Arc::new(updated)
    }

    pub fn update_switch_statement(
        &self,
        node: &Arc<Node>,
        expression: &Arc<Node>,
        case_block: &Arc<Node>,
    ) -> Arc<Node> { ::tsox_core::fntrace::enter("update_switch_statement"); 
        let mut updated = Node::new(
            SyntaxKind::SwitchStatement,
            NodeData::SwitchStatement(ndg::SwitchStatementData {
                expression: expression.clone(),
                case_block: case_block.clone(),
            }),
        );
        updated.loc = node.loc;
        updated.flags = node.flags;
        Arc::new(updated)
    }

    pub fn update_case_or_default_clause(
        &self,
        node: &Arc<Node>,
        expression: Option<&Arc<Node>>,
        statements: &NodeList,
    ) -> Arc<Node> { ::tsox_core::fntrace::enter("update_case_or_default_clause"); 
        let mut updated = Node::new(
            node.kind,
            NodeData::CaseOrDefaultClause(ndg::CaseOrDefaultClauseData {
                expression: expression.cloned().unwrap_or_else(|| {
                    Arc::new(Node::new(SyntaxKind::Unknown, NodeData::Token))
                }),
                statements: Arc::new(NodeList {
                    loc: statements.loc,
                    nodes: statements.nodes.clone(),
                }),
            }),
        );
        updated.loc = node.loc;
        updated.flags = node.flags;
        Arc::new(updated)
    }

    pub fn update_catch_clause(
        &self,
        node: &Arc<Node>,
        variable_declaration: Option<&Arc<Node>>,
        block: &Arc<Node>,
    ) -> Arc<Node> { ::tsox_core::fntrace::enter("update_catch_clause"); 
        let mut updated = Node::new(
            SyntaxKind::CatchClause,
            NodeData::CatchClause(ndg::CatchClauseData {
                variable_declaration: variable_declaration.cloned(),
                block: block.clone(),
            }),
        );
        updated.loc = node.loc;
        updated.flags = node.flags;
        Arc::new(updated)
    }
}
