//! r33k7: tsox-emit mig m4g 族缺失符号的按 Go 移植(访问器/枚举/trait)
use std::sync::Arc;
use tsox_frontend::ast::node::{Node, NodeList};
use tsox_frontend::ast::node_data_generated::NodeData;
use tsox_frontend::ast::{has_syntactic_modifier, ModifierFlags};

pub fn has_decorators(node: &Node) -> bool {
    has_syntactic_modifier(node, ModifierFlags::Decorator)
}

pub enum PrivateIdentifierKind {
    Field,
    Method,
    Accessor,
    Untransformed,
}

pub trait ReferenceResolver {
    fn get_referenced_export_container(
        &self,
        node: &Arc<Node>,
        prefix_locals: bool,
    ) -> Option<Arc<Node>>;

    fn get_referenced_import_declaration(&self, node: &Arc<Node>) -> Option<Arc<Node>>;

    fn get_referenced_value_declaration(&self, node: &Arc<Node>) -> Option<Arc<Node>>;

    fn get_referenced_value_declarations(&self, node: &Arc<Node>) -> Vec<Arc<Node>>;

    fn get_element_access_expression_name(&self, expression: &Arc<Node>) -> String;

    fn get_referenced_member_value_declaration(&self, node: &Arc<Node>) -> Option<Arc<Node>>;
}

pub fn clone_node<F>(node: &Arc<Node>, _factory: &F) -> Arc<Node> {
    node.clone()
}

pub fn class_like_name(node: &Node) -> Option<&Arc<Node>> {
    match &node.data {
        NodeData::ClassDeclaration(d) => d.name.as_ref(),
        NodeData::ClassExpression(d) => d.name.as_ref(),
        _ => None,
    }
}

pub fn for_statement_initializer(node: &Node) -> Option<&Arc<Node>> {
    match &node.data {
        NodeData::ForStatement(d) => d.initializer.as_ref(),
        _ => None,
    }
}

pub fn for_statement_condition(node: &Node) -> Option<&Arc<Node>> {
    match &node.data {
        NodeData::ForStatement(d) => d.condition.as_ref(),
        _ => None,
    }
}

pub fn for_statement_incrementor(node: &Node) -> Option<&Arc<Node>> {
    match &node.data {
        NodeData::ForStatement(d) => d.incrementor.as_ref(),
        _ => None,
    }
}

pub fn for_statement_statement(node: &Node) -> &Arc<Node> {
    match &node.data {
        NodeData::ForStatement(d) => &d.statement,
        _ => panic!("expected ForStatement"),
    }
}

pub fn expression_statement_expression(node: &Node) -> Option<&Arc<Node>> {
    match &node.data {
        NodeData::ExpressionStatement(d) => Some(&d.expression),
        _ => None,
    }
}

pub fn computed_property_name_expression(node: &Node) -> &Arc<Node> {
    match &node.data {
        NodeData::ComputedPropertyName(d) => &d.expression,
        _ => panic!("expected ComputedPropertyName"),
    }
}

pub fn property_access_name(node: &Node) -> &Arc<Node> {
    match &node.data {
        NodeData::PropertyAccessExpression(d) => &d.name,
        _ => panic!("expected PropertyAccessExpression"),
    }
}

pub fn property_access_expression(node: &Node) -> &Arc<Node> {
    match &node.data {
        NodeData::PropertyAccessExpression(d) => &d.expression,
        _ => panic!("expected PropertyAccessExpression"),
    }
}

pub fn property_access_question_dot_token(node: &Node) -> Option<&Arc<Node>> {
    match &node.data {
        NodeData::PropertyAccessExpression(d) => d.question_dot_token.as_ref(),
        _ => None,
    }
}

pub fn element_access_argument(node: &Node) -> Option<&Arc<Node>> {
    match &node.data {
        NodeData::ElementAccessExpression(d) => Some(&d.argument_expression),
        _ => None,
    }
}

pub fn binary_left(node: &Node) -> &Arc<Node> {
    match &node.data {
        NodeData::BinaryExpression(d) => &d.left,
        _ => panic!("expected BinaryExpression"),
    }
}

pub fn binary_right(node: &Node) -> &Arc<Node> {
    match &node.data {
        NodeData::BinaryExpression(d) => &d.right,
        _ => panic!("expected BinaryExpression"),
    }
}

pub fn binary_operator_token(node: &Node) -> &Arc<Node> {
    match &node.data {
        NodeData::BinaryExpression(d) => &d.operator_token,
        _ => panic!("expected BinaryExpression"),
    }
}

pub fn call_expression_expression(node: &Node) -> &Arc<Node> {
    match &node.data {
        NodeData::CallExpression(d) => &d.expression,
        _ => panic!("expected CallExpression"),
    }
}

pub fn call_expression_arguments(node: &Node) -> &Arc<NodeList> {
    match &node.data {
        NodeData::CallExpression(d) => &d.arguments,
        _ => panic!("expected CallExpression"),
    }
}

pub fn tagged_template_tag(node: &Node) -> &Arc<Node> {
    match &node.data {
        NodeData::TaggedTemplateExpression(d) => &d.tag,
        _ => panic!("expected TaggedTemplateExpression"),
    }
}

pub fn tagged_template_template(node: &Node) -> &Arc<Node> {
    match &node.data {
        NodeData::TaggedTemplateExpression(d) => &d.template,
        _ => panic!("expected TaggedTemplateExpression"),
    }
}

pub fn parenthesized_expression(node: &Node) -> &Arc<Node> {
    match &node.data {
        NodeData::ParenthesizedExpression(d) => &d.expression,
        _ => panic!("expected ParenthesizedExpression"),
    }
}

pub fn node_body(node: &Node) -> Option<&Arc<Node>> {
    match &node.data {
        NodeData::MethodDeclaration(d) => d.body.as_ref(),
        NodeData::FunctionDeclaration(d) => d.body.as_ref(),
        NodeData::FunctionExpression(d) => Some(&d.body),
        NodeData::GetAccessorDeclaration(d) => d.body.as_ref(),
        NodeData::SetAccessorDeclaration(d) => d.body.as_ref(),
        NodeData::ConstructorDeclaration(d) => d.body.as_ref(),
        _ => None,
    }
}

pub fn node_body_data_asterisk_token(node: &Node) -> Option<&Arc<Node>> {
    match &node.data {
        NodeData::MethodDeclaration(d) => d.asterisk_token.as_ref(),
        NodeData::FunctionDeclaration(d) => d.asterisk_token.as_ref(),
        NodeData::FunctionExpression(d) => d.asterisk_token.as_ref(),
        _ => None,
    }
}

pub fn node_parameter_list(node: &Node) -> &Arc<NodeList> {
    match &node.data {
        NodeData::MethodDeclaration(d) => &d.parameters,
        NodeData::FunctionDeclaration(d) => &d.parameters,
        NodeData::FunctionExpression(d) => &d.parameters,
        NodeData::GetAccessorDeclaration(d) => &d.parameters,
        NodeData::SetAccessorDeclaration(d) => &d.parameters,
        NodeData::ConstructorDeclaration(d) => &d.parameters,
        _ => panic!("expected function-like node"),
    }
}

pub fn class_members(node: &Node) -> &[Arc<Node>] {
    match &node.data {
        NodeData::ClassDeclaration(d) => &d.members.nodes,
        NodeData::ClassExpression(d) => &d.members.nodes,
        _ => panic!("expected ClassDeclaration or ClassExpression"),
    }
}
