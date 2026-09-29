#![allow(unused_imports, dead_code)]

use std::sync::Arc;

use tsox_frontend::ast::node::Node;
use tsox_frontend::ast::node_data_generated::{
    ElementAccessExpressionData, ModuleBlockData, ModuleDeclarationData, NodeData,
    PrivateIdentifierData, PropertyDeclarationData, StringLiteralData, SyntaxListData,
    TemplateExpressionData, TemplateHeadData, TemplateSpanData, TemplateTailData,
};
use tsox_frontend::ast::node_flags::NodeFlags;
use tsox_frontend::ast::syntax_kind_generated::SyntaxKind;
use tsox_frontend::ast::{ModifierList, NodeList};
use tsox_frontend::scanner::TokenFlags;

use crate::printer::NodeFactory;

impl<'a> NodeFactory<'a> {
    pub fn new_string_literal(&self, text: &str, token_flags: TokenFlags) -> Arc<Node> {
        Arc::new(Node::new(
            SyntaxKind::StringLiteral,
            NodeData::StringLiteral(StringLiteralData {
                text: text.to_string(),
                token_flags,
            }),
        ))
    }

    pub fn new_private_identifier(&self, text: &str) -> Arc<Node> {
        Arc::new(Node::new(
            SyntaxKind::PrivateIdentifier,
            NodeData::PrivateIdentifier(PrivateIdentifierData {
                text: text.to_string(),
            }),
        ))
    }

    pub fn new_property_declaration(
        &self,
        modifiers: Option<Arc<ModifierList>>,
        name: &Arc<Node>,
        postfix_token: Option<&Arc<Node>>,
        type_node: Option<&Arc<Node>>,
        initializer: Option<&Arc<Node>>,
    ) -> Arc<Node> {
        Arc::new(Node::new(
            SyntaxKind::PropertyDeclaration,
            NodeData::PropertyDeclaration(PropertyDeclarationData {
                modifiers,
                name: name.clone(),
                postfix_token: postfix_token.cloned(),
                type_node: type_node.cloned(),
                initializer: initializer.cloned(),
            }),
        ))
    }

    pub fn new_element_access_expression(
        &self,
        expression: &Arc<Node>,
        question_dot_token: Option<&Arc<Node>>,
        argument_expression: &Arc<Node>,
        flags: NodeFlags,
    ) -> Arc<Node> {
        let mut node = Node::new(
            SyntaxKind::ElementAccessExpression,
            NodeData::ElementAccessExpression(ElementAccessExpressionData {
                expression: expression.clone(),
                question_dot_token: question_dot_token.cloned(),
                argument_expression: argument_expression.clone(),
            }),
        );
        node.flags = flags;
        Arc::new(node)
    }

    pub fn new_module_declaration(
        &self,
        modifiers: Option<Arc<ModifierList>>,
        keyword: SyntaxKind,
        name: &Arc<Node>,
        attributes: Option<&Arc<Node>>,
        body: Option<&Arc<Node>>,
    ) -> Arc<Node> {
        Arc::new(Node::new(
            SyntaxKind::ModuleDeclaration,
            NodeData::ModuleDeclaration(ModuleDeclarationData {
                modifiers,
                keyword,
                name: name.clone(),
                attributes: attributes.cloned(),
                body: body.cloned(),
            }),
        ))
    }

    pub fn new_module_block(&self, statements: Arc<NodeList>) -> Arc<Node> {
        Arc::new(Node::new(
            SyntaxKind::ModuleBlock,
            NodeData::ModuleBlock(ModuleBlockData {
                statements,
            }),
        ))
    }

    pub fn new_syntax_list(&self, children: Vec<Arc<Node>>) -> Arc<Node> {
        Arc::new(Node::new(
            SyntaxKind::SyntaxList,
            NodeData::SyntaxList(SyntaxListData { children }),
        ))
    }

    pub fn new_template_head(
        &self,
        text: &str,
        raw_text: &str,
        template_flags: TokenFlags,
    ) -> Arc<Node> {
        Arc::new(Node::new(
            SyntaxKind::TemplateHead,
            NodeData::TemplateHead(TemplateHeadData {
                text: text.to_string(),
                raw_text: raw_text.to_string(),
                template_flags,
            }),
        ))
    }

    pub fn new_template_tail(
        &self,
        text: &str,
        raw_text: &str,
        template_flags: TokenFlags,
    ) -> Arc<Node> {
        Arc::new(Node::new(
            SyntaxKind::TemplateTail,
            NodeData::TemplateTail(TemplateTailData {
                text: text.to_string(),
                raw_text: raw_text.to_string(),
                template_flags,
            }),
        ))
    }

    pub fn new_template_span(&self, expression: &Arc<Node>, literal: &Arc<Node>) -> Arc<Node> {
        Arc::new(Node::new(
            SyntaxKind::TemplateSpan,
            NodeData::TemplateSpan(TemplateSpanData {
                expression: expression.clone(),
                literal: literal.clone(),
            }),
        ))
    }

    pub fn new_template_expression(
        &self,
        head: &Arc<Node>,
        template_spans: Arc<NodeList>,
    ) -> Arc<Node> {
        Arc::new(Node::new(
            SyntaxKind::TemplateExpression,
            NodeData::TemplateExpression(TemplateExpressionData {
                head: head.clone(),
                template_spans,
            }),
        ))
    }

    pub fn new_import_star_helper(&self, expression: &Arc<Node>) -> Arc<Node> {
        self.new_call_expression(
            &self.new_unscoped_helper_name("__importStar"),
            None,
            None,
            self.new_node_list(vec![expression.clone()]),
            NodeFlags::empty(),
        )
    }
}
