use crate::ast::mig::m3g_3::{skip_outer_expressions, OuterExpressionKinds};
use crate::ast::{Node, SyntaxKind};
use std::sync::Arc;

pub fn is_array_binding_pattern(node: &Node) -> bool { ::tsox_core::fntrace::enter("is_array_binding_pattern"); 
    node.kind == SyntaxKind::ArrayBindingPattern
}

pub fn is_object_binding_pattern(node: &Node) -> bool { ::tsox_core::fntrace::enter("is_object_binding_pattern"); 
    node.kind == SyntaxKind::ObjectBindingPattern
}

pub fn is_case_clause(node: &Node) -> bool { ::tsox_core::fntrace::enter("is_case_clause"); 
    node.kind == SyntaxKind::CaseClause
}

pub fn is_for_in_statement(node: &Node) -> bool { ::tsox_core::fntrace::enter("is_for_in_statement"); 
    node.kind == SyntaxKind::ForInStatement
}

pub fn is_for_of_statement(node: &Node) -> bool { ::tsox_core::fntrace::enter("is_for_of_statement"); 
    node.kind == SyntaxKind::ForOfStatement
}

pub fn is_js_type_alias_declaration(node: &Node) -> bool { ::tsox_core::fntrace::enter("is_js_type_alias_declaration"); 
    node.kind == SyntaxKind::JSTypeAliasDeclaration
}

pub fn skip_outer_expression_all(node: &Arc<Node>) -> Arc<Node> { ::tsox_core::fntrace::enter("skip_outer_expression_all"); 
    skip_outer_expressions(node, OuterExpressionKinds::ALL)
}
