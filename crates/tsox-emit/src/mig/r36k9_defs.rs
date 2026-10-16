#![allow(unused_imports, dead_code)]

use std::sync::Arc;

use tsox_frontend::ast::node::Node;
use tsox_frontend::ast::NodeList;
use tsox_frontend::ast::node_data_generated::{IdentifierData, NodeData};
use tsox_frontend::ast::node_flags::NodeFlags;
use tsox_frontend::ast::syntax_kind_generated::SyntaxKind;
use tsox_frontend::ast::ModifierList;

use tsox_frontend::ast::node_data_generated as ndg;

use crate::printer::generated_identifier_flags::{AutoGenerateOptions, GeneratedName, NodeFactory};
use tsox_frontend::ast::mig::m3e::FunctionFlags;

pub trait FunctionFlagsExt {
    fn intersects(self, other: FunctionFlags) -> bool;
}

impl FunctionFlagsExt for FunctionFlags {
    fn intersects(self, other: FunctionFlags) -> bool { ::tsox_core::fntrace::enter("intersects"); 
        self.0 & other.0 != 0
    }
}

impl<'a> NodeFactory<'a> {
    pub fn generated_name_node(&self, name: &GeneratedName) -> Arc<Node> { ::tsox_core::fntrace::enter("generated_name_node"); 
        self.new_identifier(name.text())
    }

    pub fn new_token(&self, kind: SyntaxKind) -> Arc<Node> { ::tsox_core::fntrace::enter("new_token"); 
        Arc::new(Node::new(kind, NodeData::Token))
    }

    pub fn new_node_list(&self, nodes: Vec<Arc<Node>>) -> Arc<NodeList> { ::tsox_core::fntrace::enter("new_node_list"); 
        Arc::new(NodeList::new(nodes))
    }

    pub fn new_identifier(&self, text: &str) -> Arc<Node> { ::tsox_core::fntrace::enter("new_identifier"); 
        Arc::new(Node::new(
            SyntaxKind::Identifier,
            NodeData::Identifier(IdentifierData {
                text: text.to_string(),
            }),
        ))
    }

    pub fn new_keyword_expression(&self, kind: SyntaxKind) -> Arc<Node> { ::tsox_core::fntrace::enter("new_keyword_expression"); 
        self.new_token(kind)
    }

    pub fn new_expression_statement(&self, expression: &Arc<Node>) -> Arc<Node> { ::tsox_core::fntrace::enter("new_expression_statement"); 
        Arc::new(Node::new(
            SyntaxKind::ExpressionStatement,
            NodeData::ExpressionStatement(ndg::ExpressionStatementData {
                expression: expression.clone(),
            }),
        ))
    }

    pub fn new_block(&self, statements: &NodeList, multi_line: bool) -> Arc<Node> { ::tsox_core::fntrace::enter("new_block"); 
        Arc::new(Node::new(
            SyntaxKind::Block,
            NodeData::Block(ndg::BlockData {
                statements: Arc::new(crate::mig::m4h_4::r36k9_defs::cloned_node_list(statements)),
                multi_line,
            }),
        ))
    }

    pub fn update_block(&self, node: &Arc<Node>, statements: &NodeList, multi_line: bool) -> Arc<Node> { ::tsox_core::fntrace::enter("update_block"); 
        let mut updated = Node::new(
            SyntaxKind::Block,
            NodeData::Block(ndg::BlockData {
                statements: Arc::new(crate::mig::m4h_4::r36k9_defs::cloned_node_list(statements)),
                multi_line,
            }),
        );
        updated.loc = node.loc;
        updated.flags = node.flags;
        Arc::new(updated)
    }

    pub fn new_binary_expression(
        &self,
        modifiers: Option<Arc<ModifierList>>,
        left: &Arc<Node>,
        type_node: Option<Arc<Node>>,
        operator_token: &Arc<Node>,
        right: &Arc<Node>,
    ) -> Arc<Node> { ::tsox_core::fntrace::enter("new_binary_expression"); 
        Arc::new(Node::new(
            SyntaxKind::BinaryExpression,
            NodeData::BinaryExpression(ndg::BinaryExpressionData {
                modifiers,
                left: left.clone(),
                type_node,
                operator_token: operator_token.clone(),
                right: right.clone(),
            }),
        ))
    }

    pub fn new_property_access_expression(
        &self,
        expression: &Arc<Node>,
        question_dot_token: Option<Arc<Node>>,
        name: &Arc<Node>,
        flags: NodeFlags,
    ) -> Arc<Node> { ::tsox_core::fntrace::enter("new_property_access_expression"); 
        let mut node = Node::new(
            SyntaxKind::PropertyAccessExpression,
            NodeData::PropertyAccessExpression(
                ndg::PropertyAccessExpressionData {
                    expression: expression.clone(),
                    question_dot_token,
                    name: name.clone(),
                },
            ),
        );
        node.flags = flags;
        Arc::new(node)
    }

    pub fn new_call_expression(
        &self,
        expression: &Arc<Node>,
        question_dot_token: Option<Arc<Node>>,
        type_arguments: Option<Arc<NodeList>>,
        arguments: Arc<NodeList>,
        flags: NodeFlags,
    ) -> Arc<Node> { ::tsox_core::fntrace::enter("new_call_expression"); 
        let mut node = Node::new(
            SyntaxKind::CallExpression,
            NodeData::CallExpression(ndg::CallExpressionData {
                expression: expression.clone(),
                question_dot_token,
                type_arguments,
                arguments,
            }),
        );
        node.flags = flags;
        Arc::new(node)
    }

    pub fn new_void_expression(&self, expression: &Arc<Node>) -> Arc<Node> { ::tsox_core::fntrace::enter("new_void_expression"); 
        Arc::new(Node::new(
            SyntaxKind::VoidExpression,
            NodeData::VoidExpression(ndg::VoidExpressionData {
                expression: expression.clone(),
            }),
        ))
    }

    pub fn new_void_zero_expression(&self) -> Arc<Node> { ::tsox_core::fntrace::enter("new_void_zero_expression"); 
        self.new_void_expression(&Arc::new(Node::new(
            SyntaxKind::NumericLiteral,
            NodeData::NumericLiteral(ndg::NumericLiteralData {
                text: "0".to_string(),
                token_flags: Default::default(),
            }),
        )))
    }

    pub fn new_yield_expression(
        &self,
        asterisk_token: Option<&Arc<Node>>,
        expression: &Arc<Node>,
    ) -> Arc<Node> { ::tsox_core::fntrace::enter("new_yield_expression"); 
        Arc::new(Node::new(
            SyntaxKind::YieldExpression,
            NodeData::YieldExpression(ndg::YieldExpressionData {
                asterisk_token: asterisk_token.cloned(),
                expression: Some(expression.clone()),
            }),
        ))
    }

    pub fn new_await_expression(&self, expression: &Arc<Node>) -> Arc<Node> { ::tsox_core::fntrace::enter("new_await_expression"); 
        Arc::new(Node::new(
            SyntaxKind::AwaitExpression,
            NodeData::AwaitExpression(ndg::AwaitExpressionData {
                expression: expression.clone(),
            }),
        ))
    }

    pub fn new_variable_declaration(
        &self,
        name: &Arc<Node>,
        exclamation_token: Option<&Arc<Node>>,
        type_node: Option<&Arc<Node>>,
        initializer: Option<&Arc<Node>>,
    ) -> Arc<Node> { ::tsox_core::fntrace::enter("new_variable_declaration"); 
        Arc::new(Node::new(
            SyntaxKind::VariableDeclaration,
            NodeData::VariableDeclaration(ndg::VariableDeclarationData {
                name: name.clone(),
                exclamation_token: exclamation_token.cloned(),
                type_node: type_node.cloned(),
                initializer: initializer.cloned(),
            }),
        ))
    }

    pub fn new_variable_declaration_list(
        &self,
        declarations: &NodeList,
        flags: NodeFlags,
    ) -> Arc<Node> { ::tsox_core::fntrace::enter("new_variable_declaration_list"); 
        let mut node = Node::new(
            SyntaxKind::VariableDeclarationList,
            NodeData::VariableDeclarationList(
                ndg::VariableDeclarationListData {
                    declarations: Arc::new(crate::mig::m4h_4::r36k9_defs::cloned_node_list(declarations)),
                },
            ),
        );
        node.flags = flags;
        Arc::new(node)
    }

    pub fn new_prefix_unary_expression(&self, operator: SyntaxKind, operand: &Arc<Node>) -> Arc<Node> { ::tsox_core::fntrace::enter("new_prefix_unary_expression"); 
        Arc::new(Node::new(
            SyntaxKind::PrefixUnaryExpression,
            NodeData::PrefixUnaryExpression(
                ndg::PrefixUnaryExpressionData {
                    operator,
                    operand: operand.clone(),
                },
            ),
        ))
    }

    pub fn new_for_statement(
        &self,
        initializer: Option<&Arc<Node>>,
        condition: Option<&Arc<Node>>,
        incrementor: Option<&Arc<Node>>,
        statement: &Arc<Node>,
    ) -> Arc<Node> { ::tsox_core::fntrace::enter("new_for_statement"); 
        Arc::new(Node::new(
            SyntaxKind::ForStatement,
            NodeData::ForStatement(ndg::ForStatementData {
                initializer: initializer.cloned(),
                condition: condition.cloned(),
                incrementor: incrementor.cloned(),
                statement: statement.clone(),
            }),
        ))
    }

    pub fn new_if_statement(
        &self,
        expression: &Arc<Node>,
        then_statement: &Arc<Node>,
        else_statement: Option<&Arc<Node>>,
    ) -> Arc<Node> { ::tsox_core::fntrace::enter("new_if_statement"); 
        Arc::new(Node::new(
            SyntaxKind::IfStatement,
            NodeData::IfStatement(ndg::IfStatementData {
                expression: expression.clone(),
                then_statement: then_statement.clone(),
                else_statement: else_statement.cloned(),
            }),
        ))
    }

    pub fn new_throw_statement(&self, expression: &Arc<Node>) -> Arc<Node> { ::tsox_core::fntrace::enter("new_throw_statement"); 
        Arc::new(Node::new(
            SyntaxKind::ThrowStatement,
            NodeData::ThrowStatement(ndg::ThrowStatementData {
                expression: expression.clone(),
            }),
        ))
    }

    pub fn new_return_statement(&self, expression: Option<&Arc<Node>>) -> Arc<Node> { ::tsox_core::fntrace::enter("new_return_statement"); 
        Arc::new(Node::new(
            SyntaxKind::ReturnStatement,
            NodeData::ReturnStatement(ndg::ReturnStatementData {
                expression: expression.cloned(),
            }),
        ))
    }

    pub fn new_try_statement(
        &self,
        try_block: &Arc<Node>,
        catch_clause: Option<&Arc<Node>>,
        finally_block: Option<&Arc<Node>>,
    ) -> Arc<Node> { ::tsox_core::fntrace::enter("new_try_statement"); 
        Arc::new(Node::new(
            SyntaxKind::TryStatement,
            NodeData::TryStatement(ndg::TryStatementData {
                try_block: try_block.clone(),
                catch_clause: catch_clause.cloned(),
                finally_block: finally_block.cloned(),
            }),
        ))
    }

    pub fn new_catch_clause(
        &self,
        variable_declaration: Option<&Arc<Node>>,
        block: &Arc<Node>,
    ) -> Arc<Node> { ::tsox_core::fntrace::enter("new_catch_clause"); 
        Arc::new(Node::new(
            SyntaxKind::CatchClause,
            NodeData::CatchClause(ndg::CatchClauseData {
                variable_declaration: variable_declaration.cloned(),
                block: block.clone(),
            }),
        ))
    }

    pub fn new_object_literal_expression(
        &self,
        properties: &NodeList,
        multi_line: bool,
    ) -> Arc<Node> { ::tsox_core::fntrace::enter("new_object_literal_expression"); 
        Arc::new(Node::new(
            SyntaxKind::ObjectLiteralExpression,
            NodeData::ObjectLiteralExpression(
                ndg::ObjectLiteralExpressionData {
                    properties: Arc::new(crate::mig::m4h_4::r36k9_defs::cloned_node_list(properties)),
                    multi_line,
                },
            ),
        ))
    }

    pub fn new_parameter_declaration(
        &self,
        modifiers: Option<Arc<ModifierList>>,
        dot_dot_dot_token: Option<&Arc<Node>>,
        name: &Arc<Node>,
        question_token: Option<&Arc<Node>>,
        type_node: Option<&Arc<Node>>,
        initializer: Option<&Arc<Node>>,
    ) -> Arc<Node> { ::tsox_core::fntrace::enter("new_parameter_declaration"); 
        Arc::new(Node::new(
            SyntaxKind::Parameter,
            NodeData::ParameterDeclaration(ndg::ParameterDeclarationData {
                modifiers,
                dot_dot_dot_token: dot_dot_dot_token.cloned(),
                name: name.clone(),
                question_token: question_token.cloned(),
                type_node: type_node.cloned(),
                initializer: initializer.cloned(),
            }),
        ))
    }

    pub fn new_function_expression(
        &self,
        modifiers: Option<Arc<ModifierList>>,
        asterisk_token: Option<&Arc<Node>>,
        name: Option<&Arc<Node>>,
        type_parameters: Option<Arc<NodeList>>,
        parameters: &NodeList,
        type_node: Option<Arc<Node>>,
        full_signature: Option<Arc<Node>>,
        body: &Arc<Node>,
    ) -> Arc<Node> { ::tsox_core::fntrace::enter("new_function_expression"); 
        Arc::new(Node::new(
            SyntaxKind::FunctionExpression,
            NodeData::FunctionExpression(ndg::FunctionExpressionData {
                modifiers,
                asterisk_token: asterisk_token.cloned(),
                name: name.cloned(),
                type_parameters,
                parameters: Arc::new(crate::mig::m4h_4::r36k9_defs::cloned_node_list(parameters)),
                type_node,
                full_signature,
                body: body.clone(),
            }),
        ))
    }

    pub fn update_method_declaration(
        &self,
        node: &Arc<Node>,
        modifiers: Option<Arc<ModifierList>>,
        asterisk_token: Option<&Arc<Node>>,
        name: &Arc<Node>,
        postfix_token: Option<Arc<Node>>,
        type_parameters: Option<Arc<NodeList>>,
        parameters: &NodeList,
        type_node: Option<Arc<Node>>,
        full_signature: Option<Arc<Node>>,
        body: Option<&Arc<Node>>,
    ) -> Arc<Node> {
        Arc::new(Node::new(
            SyntaxKind::MethodDeclaration,
            NodeData::MethodDeclaration(ndg::MethodDeclarationData {
                modifiers,
                asterisk_token: asterisk_token.cloned(),
                name: name.clone(),
                postfix_token,
                type_parameters,
                parameters: Arc::new(crate::mig::m4h_4::r36k9_defs::cloned_node_list(parameters)),
                type_node,
                full_signature,
                body: body.cloned(),
            }),
        ))
    }

    pub fn update_function_declaration(
        &self,
        node: &Arc<Node>,
        modifiers: Option<Arc<ModifierList>>,
        asterisk_token: Option<&Arc<Node>>,
        name: Option<&Arc<Node>>,
        type_parameters: Option<Arc<NodeList>>,
        parameters: &NodeList,
        type_node: Option<Arc<Node>>,
        full_signature: Option<Arc<Node>>,
        body: Option<&Arc<Node>>,
    ) -> Arc<Node> { ::tsox_core::fntrace::enter("update_function_declaration"); 
        Arc::new(Node::new(
            SyntaxKind::FunctionDeclaration,
            NodeData::FunctionDeclaration(ndg::FunctionDeclarationData {
                modifiers,
                asterisk_token: asterisk_token.cloned(),
                name: name.cloned(),
                type_parameters,
                parameters: Arc::new(crate::mig::m4h_4::r36k9_defs::cloned_node_list(parameters)),
                type_node,
                full_signature,
                body: body.cloned(),
            }),
        ))
    }

    pub fn update_function_expression(
        &self,
        node: &Arc<Node>,
        modifiers: Option<Arc<ModifierList>>,
        asterisk_token: Option<&Arc<Node>>,
        name: Option<&Arc<Node>>,
        type_parameters: Option<Arc<NodeList>>,
        parameters: &NodeList,
        type_node: Option<Arc<Node>>,
        full_signature: Option<Arc<Node>>,
        body: Option<&Arc<Node>>,
    ) -> Arc<Node> { ::tsox_core::fntrace::enter("update_function_expression"); 
        Arc::new(Node::new(
            SyntaxKind::FunctionExpression,
            NodeData::FunctionExpression(ndg::FunctionExpressionData {
                modifiers,
                asterisk_token: asterisk_token.cloned(),
                name: name.cloned(),
                type_parameters,
                parameters: Arc::new(crate::mig::m4h_4::r36k9_defs::cloned_node_list(parameters)),
                type_node,
                full_signature,
                body: body
                    .cloned()
                    .unwrap_or_else(|| Arc::new(Node::new(SyntaxKind::Block, NodeData::Token))),
            }),
        ))
    }

    pub fn update_constructor_declaration(
        &self,
        node: &Arc<Node>,
        modifiers: Option<Arc<ModifierList>>,
        type_parameters: Option<Arc<NodeList>>,
        parameters: &NodeList,
        type_node: Option<Arc<Node>>,
        full_signature: Option<Arc<Node>>,
        body: Option<&Arc<Node>>,
    ) -> Arc<Node> { ::tsox_core::fntrace::enter("update_constructor_declaration"); 
        Arc::new(Node::new(
            SyntaxKind::Constructor,
            NodeData::ConstructorDeclaration(ndg::ConstructorDeclarationData {
                modifiers,
                type_parameters,
                parameters: Arc::new(crate::mig::m4h_4::r36k9_defs::cloned_node_list(parameters)),
                type_node,
                full_signature,
                body: body.cloned(),
            }),
        ))
    }

    pub fn update_get_accessor_declaration(
        &self,
        node: &Arc<Node>,
        modifiers: Option<Arc<ModifierList>>,
        name: &Arc<Node>,
        type_parameters: Option<Arc<NodeList>>,
        parameters: &NodeList,
        type_node: Option<Arc<Node>>,
        full_signature: Option<Arc<Node>>,
        body: Option<&Arc<Node>>,
    ) -> Arc<Node> { ::tsox_core::fntrace::enter("update_get_accessor_declaration"); 
        Arc::new(Node::new(
            SyntaxKind::GetAccessor,
            NodeData::GetAccessorDeclaration(ndg::GetAccessorDeclarationData {
                modifiers,
                name: name.clone(),
                type_parameters,
                parameters: Arc::new(crate::mig::m4h_4::r36k9_defs::cloned_node_list(parameters)),
                type_node,
                full_signature,
                body: body.cloned(),
            }),
        ))
    }

    pub fn update_set_accessor_declaration(
        &self,
        node: &Arc<Node>,
        modifiers: Option<Arc<ModifierList>>,
        name: &Arc<Node>,
        type_parameters: Option<Arc<NodeList>>,
        parameters: &NodeList,
        type_node: Option<Arc<Node>>,
        full_signature: Option<Arc<Node>>,
        body: Option<&Arc<Node>>,
    ) -> Arc<Node> { ::tsox_core::fntrace::enter("update_set_accessor_declaration"); 
        Arc::new(Node::new(
            SyntaxKind::SetAccessor,
            NodeData::SetAccessorDeclaration(ndg::SetAccessorDeclarationData {
                modifiers,
                name: name.clone(),
                type_parameters,
                parameters: Arc::new(crate::mig::m4h_4::r36k9_defs::cloned_node_list(parameters)),
                type_node,
                full_signature,
                body: body.cloned(),
            }),
        ))
    }

    pub fn update_arrow_function(
        &self,
        node: &Arc<Node>,
        modifiers: Option<Arc<ModifierList>>,
        type_parameters: Option<Arc<NodeList>>,
        parameters: &NodeList,
        type_node: Option<Arc<Node>>,
        full_signature: Option<Arc<Node>>,
        equals_greater_than_token: &Arc<Node>,
        body: &Arc<Node>,
    ) -> Arc<Node> { ::tsox_core::fntrace::enter("update_arrow_function"); 
        Arc::new(Node::new(
            SyntaxKind::ArrowFunction,
            NodeData::ArrowFunction(ndg::ArrowFunctionData {
                modifiers,
                type_parameters,
                parameters: Arc::new(crate::mig::m4h_4::r36k9_defs::cloned_node_list(parameters)),
                type_node,
                full_signature,
                equals_greater_than_token: equals_greater_than_token.clone(),
                body: body.clone(),
            }),
        ))
    }
}

pub fn cloned_node_list(list: &NodeList) -> NodeList { ::tsox_core::fntrace::enter("cloned_node_list"); 
    NodeList {
        loc: list.loc,
        nodes: list.nodes.clone(),
    }
}
