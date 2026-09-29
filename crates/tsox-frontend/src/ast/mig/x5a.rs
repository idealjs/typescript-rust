#![allow(unused_imports)]
#![allow(dead_code)]

use std::sync::Arc;

use crate::ast::dynamic_imports::is_require_call;
use crate::ast::mig::m3h_2::is_variable_declaration_initialized_with_require_helper;
use crate::ast::mig::w5::get_leftmost_access_expression;
use crate::ast::node::Node;
use crate::ast::node_data_generated::NodeData;
use crate::ast::syntax_kind_generated::SyntaxKind;
use crate::ast::utilities::*;

pub fn get_module_specifier_of_bare_or_accessed_require(node: &Arc<Node>) -> Option<Arc<Node>> {
    if is_variable_declaration_initialized_with_require_helper(node, false) {
        let NodeData::VariableDeclaration(d) = &node.data else {
            return None;
        };
        let initializer = d.initializer.as_ref()?;
        let NodeData::CallExpression(call) = &initializer.data else {
            return None;
        };
        return Some(Arc::clone(&call.arguments.nodes[0]));
    }
    if is_variable_declaration_initialized_with_require_helper(node, true) {
        let NodeData::VariableDeclaration(d) = &node.data else {
            return None;
        };
        let initializer = d.initializer.as_ref()?;
        let leftmost = get_leftmost_access_expression(initializer);
        if is_require_call(&leftmost, true) {
            let NodeData::CallExpression(call) = &leftmost.data else {
                return None;
            };
            return Some(Arc::clone(&call.arguments.nodes[0]));
        }
    }
    None
}

pub fn get_next_jsdoc_comment_location(node: &Arc<Node>) -> Option<Arc<Node>> {
    if let Some(parent) = node.parent() {
        match parent.kind {
            SyntaxKind::PropertyAssignment
            | SyntaxKind::ExportAssignment
            | SyntaxKind::PropertyDeclaration
            | SyntaxKind::VariableDeclaration
            | SyntaxKind::SatisfiesExpression
            | SyntaxKind::ReturnStatement
            | SyntaxKind::VariableStatement
            | SyntaxKind::ExpressionStatement => return Some(Arc::clone(&parent)),
            SyntaxKind::VariableDeclarationList => {
                let NodeData::VariableDeclarationList(list) = &parent.data else {
                    return None;
                };
                let first = list.declarations.nodes.first()?;
                if Arc::ptr_eq(first, node) {
                    return Some(parent);
                }
            }
            _ => {}
        }
    }
    None
}

pub fn get_type_annotation_node(node: &Arc<Node>) -> Option<Arc<Node>> {
    match node.kind {
        SyntaxKind::VariableDeclaration
        | SyntaxKind::Parameter
        | SyntaxKind::PropertySignature
        | SyntaxKind::PropertyDeclaration
        | SyntaxKind::TypePredicate
        | SyntaxKind::ParenthesizedType
        | SyntaxKind::TypeOperator
        | SyntaxKind::MappedType
        | SyntaxKind::TypeAssertionExpression
        | SyntaxKind::AsExpression
        | SyntaxKind::SatisfiesExpression
        | SyntaxKind::TypeAliasDeclaration
        | SyntaxKind::JSTypeAliasDeclaration
        | SyntaxKind::NamedTupleMember
        | SyntaxKind::OptionalType
        | SyntaxKind::RestType
        | SyntaxKind::TemplateLiteralTypeSpan
        | SyntaxKind::JSDocTypeExpression
        | SyntaxKind::JSDocPropertyTag
        | SyntaxKind::JSDocNullableType
        | SyntaxKind::JSDocNonNullableType
        | SyntaxKind::JSDocOptionalType => node.type_node().map(Arc::clone),
        _ => function_like_type_node(node).map(Arc::clone),
    }
}

fn function_like_type_node(node: &Node) -> Option<&Arc<Node>> {
    match &node.data {
        NodeData::FunctionDeclaration(d) => d.type_node.as_ref(),
        NodeData::ConstructorDeclaration(d) => d.type_node.as_ref(),
        NodeData::GetAccessorDeclaration(d) => d.type_node.as_ref(),
        NodeData::SetAccessorDeclaration(d) => d.type_node.as_ref(),
        NodeData::MethodDeclaration(d) => d.type_node.as_ref(),
        NodeData::ArrowFunction(d) => d.type_node.as_ref(),
        NodeData::FunctionExpression(d) => d.type_node.as_ref(),
        _ => None,
    }
}
