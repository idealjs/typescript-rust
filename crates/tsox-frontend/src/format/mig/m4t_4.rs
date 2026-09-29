use crate::ast::mig::m3b::is_type_or_js_type_alias_declaration;
use crate::ast::mig::m3b::{element_list, member_list, parameter_list, property_list};
use crate::ast::mig::m3c::{statement_list, type_argument_list, type_parameter_list};
use crate::ast::mig::m3g::is_modifier;
use crate::ast::node_data_generated::{
    for_each_child, is_arrow_function, is_call_expression, is_enum_member, is_function_expression,
    is_infer_type_node, is_interface_declaration, NodeData,
};
use crate::ast::node_node::Node;
use crate::ast::node_node_list::NodeList;
use crate::ast::syntax_kind_generated::SyntaxKind;
use crate::ast::utilities_expressions::skip_partially_emitted_expressions_arc;
use crate::ast::utilities_functions::{
    is_class_element, is_class_like, is_function_like, is_type_element,
};
use crate::ast::utilities_predicates::is_jsx_child;
use crate::ast::utilities_statements::{is_prologue_directive, is_statement};
use crate::ast::utilities_synthesized::node_is_synthesized;
use crate::ast::utilities_types::is_type_node;
use std::sync::Arc;

#[derive(Default)]
pub struct EmitContext;

impl EmitContext {
    pub fn most_original(&self, node: &Arc<Node>) -> Arc<Node> {
        node.clone()
    }
}

pub trait HasEnd {
    fn end(&self) -> i64;
}

pub fn sibling_node_positions_are_comparable(
    emit_context: &EmitContext,
    previous_node: &Arc<Node>,
    next_node: &Arc<Node>,
) -> bool {
    if next_node.pos() < previous_node.end() {
        return false;
    }

    let previous_node = emit_context.most_original(previous_node);
    let next_node = emit_context.most_original(next_node);
    let parent = match previous_node.parent() {
        Some(p) => p,
        None => return false,
    };
    if !next_node
        .parent()
        .as_ref()
        .is_some_and(|p| Arc::ptr_eq(p, &parent))
    {
        return false;
    }

    let parent_node_array = get_containing_node_array(&previous_node);
    if let Some(parent_node_array) = parent_node_array {
        let prev_node_index = parent_node_array
            .nodes
            .iter()
            .position(|n| Arc::ptr_eq(n, &previous_node));
        return match prev_node_index {
            Some(i) => parent_node_array
                .nodes
                .get(i + 1)
                .map(|n| Arc::ptr_eq(n, &next_node))
                .unwrap_or(false),
            None => false,
        };
    }

    false
}

pub fn get_containing_node_array(node: &Arc<Node>) -> Option<Arc<NodeList>> {
    let parent = node.parent()?;

    match node.kind {
        SyntaxKind::TypeParameter => {
            if is_function_like(&parent)
                || is_class_like(&parent)
                || is_interface_declaration(&parent)
                || is_type_or_js_type_alias_declaration(&parent)
            {
                return type_parameter_list(&parent).cloned();
            } else if is_infer_type_node(&parent) {
            } else {
                panic!("Unexpected TypeParameter parent: {:?}", parent.kind);
            }
        }
        SyntaxKind::Parameter => {
            return node
                .parent()
                .and_then(|p| parameter_list(&p).cloned());
        }
        SyntaxKind::TemplateLiteralTypeSpan => {
            return node.parent().and_then(|p| match &p.data {
                NodeData::TemplateLiteralTypeNode(d) => Some(d.template_spans.clone()),
                _ => None,
            });
        }
        SyntaxKind::TemplateSpan => {
            return node.parent().and_then(|p| match &p.data {
                NodeData::TemplateExpression(d) => Some(d.template_spans.clone()),
                _ => None,
            });
        }
        SyntaxKind::Decorator => {
            if crate::ast::utilities_declarations::can_have_decorators(&parent) {
                if let Some(modifiers) = parent.modifiers() {
                    return Some(Arc::new(NodeList {
                        loc: modifiers.list.loc,
                        nodes: modifiers.list.nodes.clone(),
                    }));
                }
            }
            return None;
        }
        SyntaxKind::HeritageClause => {
            if is_class_like(&parent) {
                match &parent.data {
                    NodeData::ClassDeclaration(d) => return d.heritage_clauses.clone(),
                    NodeData::ClassExpression(d) => return d.heritage_clauses.clone(),
                    _ => return None,
                }
            } else if let NodeData::InterfaceDeclaration(d) = &parent.data {
                return d.heritage_clauses.clone();
            } else {
                return None;
            }
        }
        _ => {}
    }

    match parent.kind {
        SyntaxKind::TypeLiteral | SyntaxKind::InterfaceDeclaration => {
            if is_type_element(node) {
                return member_list(&parent).cloned();
            }
        }
        SyntaxKind::UnionType => {
            if let NodeData::UnionTypeNode(d) = &parent.data {
                return Some(d.types.clone());
            }
        }
        SyntaxKind::IntersectionType => {
            if let NodeData::IntersectionTypeNode(d) = &parent.data {
                return Some(d.types.clone());
            }
        }
        SyntaxKind::ArrayLiteralExpression
        | SyntaxKind::TupleType
        | SyntaxKind::NamedImports
        | SyntaxKind::NamedExports => {
            return element_list(&parent).cloned();
        }
        SyntaxKind::ObjectLiteralExpression | SyntaxKind::JsxAttributes => {
            return property_list(&parent).cloned();
        }
        SyntaxKind::CallExpression => {
            if let NodeData::CallExpression(d) = &parent.data {
                if is_type_node(node) {
                    return d.type_arguments.clone();
                } else if !Arc::ptr_eq(node, &d.expression) {
                    return Some(d.arguments.clone());
                }
            }
        }
        SyntaxKind::NewExpression => {
            if let NodeData::NewExpression(d) = &parent.data {
                if is_type_node(node) {
                    return d.type_arguments.clone();
                } else if !Arc::ptr_eq(node, &d.expression) {
                    return d.arguments.clone();
                }
            }
        }
        SyntaxKind::JsxElement | SyntaxKind::JsxFragment => {
            if is_jsx_child(node) {
                let mut nodes = Vec::new();
                for_each_child(&parent, |child| {
                    nodes.push(child.clone());
                    false
                });
                return Some(Arc::new(NodeList::new(nodes)));
            }
        }
        SyntaxKind::JsxOpeningElement | SyntaxKind::JsxSelfClosingElement => {
            if is_type_node(node) {
                return type_argument_list(&parent).cloned();
            }
        }
        SyntaxKind::Block | SyntaxKind::ModuleBlock | SyntaxKind::CaseClause
        | SyntaxKind::DefaultClause => {
            return statement_list(&parent).cloned();
        }
        SyntaxKind::CaseBlock => {
            if let NodeData::CaseBlock(d) = &parent.data {
                return Some(d.clauses.clone());
            }
        }
        SyntaxKind::ClassDeclaration | SyntaxKind::ClassExpression => {
            if is_class_element(node) {
                return member_list(&parent).cloned();
            }
        }
        SyntaxKind::EnumDeclaration => {
            if is_enum_member(node) {
                return member_list(&parent).cloned();
            }
        }
        SyntaxKind::SourceFile => {
            if is_statement(node) {
                return statement_list(&parent).cloned();
            }
        }
        _ => {}
    }

    if is_modifier(node) {
        if let Some(modifiers) = parent.modifiers() {
            return Some(Arc::new(NodeList {
                loc: modifiers.list.loc,
                nodes: modifiers.list.nodes.clone(),
            }));
        }
    }

    None
}

pub fn original_nodes_have_same_parent(
    emit_context: &EmitContext,
    node_a: &Arc<Node>,
    node_b: &Arc<Node>,
) -> bool {
    let node_a = emit_context.most_original(node_a);
    if node_a.parent().is_some() {
        let node_b = emit_context.most_original(node_b);
        return match (node_a.parent(), node_b.parent()) {
            (Some(a), Some(b)) => Arc::ptr_eq(&a, &b),
            _ => false,
        };
    }
    false
}

pub fn try_get_end(node: Option<&dyn HasEnd>) -> Option<i64> {
    node.map(|n| n.end())
}

pub fn greatest_end(end: i64, nodes: &[Option<&dyn HasEnd>]) -> i64 {
    let mut end = end;
    for node in nodes.iter().rev() {
        if let Some(node_end) = try_get_end(*node) {
            if end < node_end {
                end = node_end;
            }
        }
    }
    end
}

pub fn skip_synthesized_parentheses(node: &Arc<Node>) -> Arc<Node> {
    let mut node = node.clone();
    while node.kind == SyntaxKind::ParenthesizedExpression && node_is_synthesized(&node) {
        match node.expression() {
            Some(expr) => node = expr.clone(),
            None => break,
        }
    }
    node
}

pub fn is_new_expression_without_arguments(node: &Arc<Node>) -> bool {
    matches!(
        &node.data,
        NodeData::NewExpression(d) if d.arguments.is_none()
    )
}

pub fn is_binary_operation(node: &Arc<Node>, token: SyntaxKind) -> bool {
    let node = skip_partially_emitted_expressions_arc(node);
    node.kind == SyntaxKind::BinaryExpression
        && matches!(
            &node.data,
            NodeData::BinaryExpression(d) if d.operator_token.kind == token
        )
}

pub fn mixing_binary_operators_requires_parentheses(a: SyntaxKind, b: SyntaxKind) -> bool {
    if a == SyntaxKind::QuestionQuestionToken {
        return b == SyntaxKind::AmpersandAmpersandToken || b == SyntaxKind::BarBarToken;
    }
    if b == SyntaxKind::QuestionQuestionToken {
        return a == SyntaxKind::AmpersandAmpersandToken || a == SyntaxKind::BarBarToken;
    }
    false
}

pub fn is_immediately_invoked_function_expression_or_arrow_function(node: &Arc<Node>) -> bool {
    let node = skip_partially_emitted_expressions_arc(node);
    if !is_call_expression(&node) {
        return false;
    }
    let expr = match node.expression() {
        Some(e) => e.clone(),
        None => return false,
    };
    let node = skip_partially_emitted_expressions_arc(&expr);
    is_function_expression(&node) || is_arrow_function(&node)
}
