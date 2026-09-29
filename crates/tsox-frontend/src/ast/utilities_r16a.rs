use crate::ast::mig::m3g_3::{skip_outer_expressions, OuterExpressionKinds};
use crate::ast::{Node, SyntaxKind};
use std::sync::Arc;

pub fn is_array_binding_pattern(node: &Node) -> bool {
    node.kind == SyntaxKind::ArrayBindingPattern
}

pub fn is_object_binding_pattern(node: &Node) -> bool {
    node.kind == SyntaxKind::ObjectBindingPattern
}

pub fn is_case_clause(node: &Node) -> bool {
    node.kind == SyntaxKind::CaseClause
}

pub fn is_for_in_statement(node: &Node) -> bool {
    node.kind == SyntaxKind::ForInStatement
}

pub fn is_for_of_statement(node: &Node) -> bool {
    node.kind == SyntaxKind::ForOfStatement
}

pub fn is_js_type_alias_declaration(node: &Node) -> bool {
    node.kind == SyntaxKind::JSTypeAliasDeclaration
}

pub fn skip_outer_expression_all(node: &Arc<Node>) -> Arc<Node> {
    skip_outer_expressions(node, OuterExpressionKinds::ALL)
}
