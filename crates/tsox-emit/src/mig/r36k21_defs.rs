#![allow(dead_code, unused_imports, unused_variables)]

use std::sync::Arc;

use tsox_frontend::ast::node::Node;
use tsox_frontend::ast::node_data_generated::NodeData;
use tsox_frontend::ast::NodeList;
use tsox_frontend::ast::syntax_kind_generated::SyntaxKind;

pub struct TypedNode21<'a> {
    pub node: &'a Node,
}

impl<'a> TypedNode21<'a> {
    pub fn as_node(&self) -> &'a Node {
        self.node
    }

    pub fn head(&self) -> &'a Node {
        match &self.node.data {
            NodeData::TemplateExpression(d) => &d.head,
            NodeData::TemplateLiteralTypeNode(d) => &d.head,
            _ => panic!("head() on {:?}", self.node.kind),
        }
    }

    pub fn template_spans(&self) -> &'a NodeList {
        match &self.node.data {
            NodeData::TemplateExpression(d) => &d.template_spans,
            NodeData::TemplateLiteralTypeNode(d) => &d.template_spans,
            _ => panic!("templateSpans() on {:?}", self.node.kind),
        }
    }

    pub fn expression(&self) -> &'a Node {
        match &self.node.data {
            NodeData::TemplateSpan(d) => &d.expression,
            NodeData::ThrowStatement(d) => &d.expression,
            NodeData::SwitchStatement(d) => &d.expression,
            NodeData::PropertyAccessExpression(d) => &d.expression,
            _ => panic!("expression() on {:?}", self.node.kind),
        }
    }

    pub fn literal(&self) -> &'a Node {
        match &self.node.data {
            NodeData::TemplateSpan(d) => &d.literal,
            NodeData::TemplateLiteralTypeSpan(d) => &d.literal,
            _ => panic!("literal() on {:?}", self.node.kind),
        }
    }

    pub fn type_(&self) -> &'a Node {
        match &self.node.data {
            NodeData::TemplateLiteralTypeSpan(d) => &d.type_node,
            _ => panic!("type() on {:?}", self.node.kind),
        }
    }

    pub fn tag(&self) -> &'a Node {
        match &self.node.data {
            NodeData::TaggedTemplateExpression(d) => &d.tag,
            _ => panic!("tag() on {:?}", self.node.kind),
        }
    }

    pub fn template(&self) -> &'a Node {
        match &self.node.data {
            NodeData::TaggedTemplateExpression(d) => &d.template,
            _ => panic!("template() on {:?}", self.node.kind),
        }
    }

    pub fn type_arguments(&self) -> Option<&'a NodeList> {
        match &self.node.data {
            NodeData::TaggedTemplateExpression(d) => d.type_arguments.as_deref(),
            _ => panic!("typeArguments() on {:?}", self.node.kind),
        }
    }

    pub fn case_block(&self) -> &'a Node {
        match &self.node.data {
            NodeData::SwitchStatement(d) => &d.case_block,
            _ => panic!("caseBlock() on {:?}", self.node.kind),
        }
    }

    pub fn clauses(&self) -> &'a NodeList {
        match &self.node.data {
            NodeData::CaseBlock(d) => &d.clauses,
            _ => panic!("clauses() on {:?}", self.node.kind),
        }
    }

    pub fn name(&self) -> &'a Node {
        match &self.node.data {
            NodeData::VariableDeclaration(d) => &d.name,
            NodeData::ParameterDeclaration(d) => &d.name,
            _ => panic!("name() on {:?}", self.node.kind),
        }
    }
}

pub trait NodeAsExt21 {
    fn as_typed21(&self) -> TypedNode21<'_>;

    fn assert_kind21(&self, kind: SyntaxKind) -> &Self {
        if self.as_node().kind != kind {
            panic!("expected {:?}, got {:?}", kind, self.as_node().kind);
        }
        self
    }

    fn as_node(&self) -> &Node;

    fn as_template_expression(&self) -> TypedNode21<'_> {
        self.assert_kind21(SyntaxKind::TemplateExpression);
        self.as_typed21()
    }

    fn as_template_span(&self) -> TypedNode21<'_> {
        self.assert_kind21(SyntaxKind::TemplateSpan);
        self.as_typed21()
    }

    fn as_template_literal_type_node(&self) -> TypedNode21<'_> {
        self.assert_kind21(SyntaxKind::TemplateLiteralType);
        self.as_typed21()
    }

    fn as_template_literal_type_span(&self) -> &Node {
        self.assert_kind21(SyntaxKind::TemplateLiteralTypeSpan);
        self.as_node()
    }

    fn as_tagged_template_expression(&self) -> TypedNode21<'_> {
        self.assert_kind21(SyntaxKind::TaggedTemplateExpression);
        self.as_typed21()
    }

    fn as_switch_statement(&self) -> TypedNode21<'_> {
        self.assert_kind21(SyntaxKind::SwitchStatement);
        self.as_typed21()
    }

    fn as_throw_statement(&self) -> TypedNode21<'_> {
        self.assert_kind21(SyntaxKind::ThrowStatement);
        self.as_typed21()
    }

    fn as_property_access_expression(&self) -> TypedNode21<'_> {
        self.assert_kind21(SyntaxKind::PropertyAccessExpression);
        self.as_typed21()
    }

    fn as_template_head(&self) -> &Node {
        self.assert_kind21(SyntaxKind::TemplateHead);
        self.as_node()
    }

    fn as_template_middle(&self) -> &Node {
        self.assert_kind21(SyntaxKind::TemplateMiddle);
        self.as_node()
    }

    fn as_template_tail(&self) -> &Node {
        self.assert_kind21(SyntaxKind::TemplateTail);
        self.as_node()
    }

    fn as_case_block(&self) -> &Node {
        self.assert_kind21(SyntaxKind::CaseBlock);
        self.as_node()
    }

    fn as_syntax_list(&self) -> &tsox_frontend::ast::node_data_generated::SyntaxListData {
        match &self.as_node().data {
            NodeData::SyntaxList(d) => d,
            _ => panic!("AsSyntaxList on wrong node kind"),
        }
    }

    fn as_catch_clause(&self) -> &tsox_frontend::ast::node_data_generated::CatchClauseData {
        match &self.as_node().data {
            NodeData::CatchClause(d) => d,
            _ => panic!("AsCatchClause on wrong node kind"),
        }
    }

    fn as_variable_declaration(&self) -> TypedNode21<'_> {
        self.assert_kind21(SyntaxKind::VariableDeclaration);
        self.as_typed21()
    }
}

impl NodeAsExt21 for Node {
    fn as_typed21(&self) -> TypedNode21<'_> {
        TypedNode21 { node: self }
    }

    fn as_node(&self) -> &Node {
        self
    }
}
