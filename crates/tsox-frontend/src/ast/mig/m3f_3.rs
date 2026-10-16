use crate::ast::*;
use crate::ast::node_node_list::NodeList;
use std::sync::Arc;

use super::m3e_4::{expression_is_alias, get_assignment_declaration_kind, JsDeclarationKind};
use super::m3f_4::{is_call_or_new_expression, is_catch_clause_variable_declaration_or_binding_element};
use super::m3g::is_literal_like_element_access;
use super::m3g_3::{is_variable_declaration_initialized_to_require, skip_outer_expressions, OuterExpressionKinds};
use super::m3h::{climb_past_property_access, climb_past_property_or_element_access};

pub fn is_alias_symbol_declaration(node: &Arc<Node>) -> bool { ::tsox_core::fntrace::enter("is_alias_symbol_declaration"); 
    match node.kind {
        SyntaxKind::ImportEqualsDeclaration
        | SyntaxKind::NamespaceExportDeclaration
        | SyntaxKind::NamespaceImport
        | SyntaxKind::NamespaceExport
        | SyntaxKind::ImportSpecifier
        | SyntaxKind::ExportSpecifier => true,
        SyntaxKind::ImportClause => match &node.data {
            NodeData::ImportClause(d) => d.name.is_some(),
            _ => false,
        },
        SyntaxKind::ExportAssignment => node
            .expression()
            .is_some_and(|expr| expression_is_alias(expr)),
        SyntaxKind::VariableDeclaration | SyntaxKind::BindingElement => {
            is_variable_declaration_initialized_to_require(node)
        }
        SyntaxKind::BinaryExpression => match get_assignment_declaration_kind(node) {
            JsDeclarationKind::ModuleExports | JsDeclarationKind::ExportsProperty => node
                .expression()
                .is_some_and(|expr| expression_is_alias(expr)),
            _ => false,
        },
        _ => false,
    }
}

pub fn is_argument_expression_of_element_access(node: &Arc<Node>) -> bool { ::tsox_core::fntrace::enter("is_argument_expression_of_element_access"); 
    match node.parent() {
        Some(parent) if parent.kind == SyntaxKind::ElementAccessExpression => match &parent.data
        {
            NodeData::ElementAccessExpression(d) => Arc::ptr_eq(&d.argument_expression, node),
            _ => false,
        },
        _ => false,
    }
}

pub fn is_array_binding_or_assignment_element(node: &Node) -> bool { ::tsox_core::fntrace::enter("is_array_binding_or_assignment_element"); 
    match node.kind {
        SyntaxKind::BindingElement
        | SyntaxKind::OmittedExpression
        | SyntaxKind::SpreadElement
        | SyntaxKind::ArrayLiteralExpression
        | SyntaxKind::ObjectLiteralExpression
        | SyntaxKind::Identifier
        | SyntaxKind::PropertyAccessExpression
        | SyntaxKind::ElementAccessExpression => true,
        _ => is_assignment_expression(node, true),
    }
}

pub fn is_assignment_pattern(node: &Node) -> bool { ::tsox_core::fntrace::enter("is_assignment_pattern"); 
    node.kind == SyntaxKind::ArrayLiteralExpression
        || node.kind == SyntaxKind::ObjectLiteralExpression
}

pub fn get_assignment_target(node: &Arc<Node>) -> Option<Arc<Node>> { ::tsox_core::fntrace::enter("get_assignment_target"); 
    let mut node = Arc::clone(node);
    loop {
        let parent = node.parent()?;
        match &parent.data {
            NodeData::BinaryExpression(d) => {
                if is_assignment_operator(d.operator_token.kind) && Arc::ptr_eq(&d.left, &node) {
                    return Some(parent);
                }
                return None;
            }
            NodeData::PrefixUnaryExpression(d) => {
                if d.operator == SyntaxKind::PlusPlusToken
                    || d.operator == SyntaxKind::MinusMinusToken
                {
                    return Some(parent);
                }
                return None;
            }
            NodeData::PostfixUnaryExpression(d) => {
                if d.operator == SyntaxKind::PlusPlusToken
                    || d.operator == SyntaxKind::MinusMinusToken
                {
                    return Some(parent);
                }
                return None;
            }
            NodeData::ForInOrOfStatement(d) => {
                if Arc::ptr_eq(&d.initializer, &node) {
                    return Some(parent);
                }
                return None;
            }
            NodeData::ParenthesizedExpression(_)
            | NodeData::ArrayLiteralExpression(_)
            | NodeData::SpreadElement(_)
            | NodeData::NonNullExpression(_) => {
                node = parent;
            }
            NodeData::SpreadAssignment(_) => {
                node = parent.parent()?;
            }
            NodeData::ShorthandPropertyAssignment(d) => {
                if !Arc::ptr_eq(&d.name, &node) {
                    return None;
                }
                node = parent.parent()?;
            }
            NodeData::PropertyAssignment(d) => {
                if Arc::ptr_eq(&d.name, &node) {
                    return None;
                }
                node = parent.parent()?;
            }
            _ => return None,
        }
    }
}

pub fn is_assignment_target(node: &Arc<Node>) -> bool { ::tsox_core::fntrace::enter("is_assignment_target"); 
    get_assignment_target(node).is_some()
}

pub fn is_async_function(node: &Arc<Node>) -> bool { ::tsox_core::fntrace::enter("is_async_function"); 
    match node.kind {
        SyntaxKind::FunctionDeclaration
        | SyntaxKind::FunctionExpression
        | SyntaxKind::ArrowFunction
        | SyntaxKind::MethodDeclaration => {
            let (body, asterisk_token) = match &node.data {
                NodeData::FunctionDeclaration(d) => (d.body.as_ref(), None),
                NodeData::FunctionExpression(d) => (Some(&d.body), d.asterisk_token.as_ref()),
                NodeData::ArrowFunction(d) => (Some(&d.body), None),
                NodeData::MethodDeclaration(d) => (d.body.as_ref(), d.asterisk_token.as_ref()),
                _ => (None, None),
            };
            body.is_some()
                && asterisk_token.is_none()
                && has_syntactic_modifier(node, ModifierFlags::Async)
        }
        _ => false,
    }
}

pub fn is_auto_accessor_property_declaration(node: &Node) -> bool { ::tsox_core::fntrace::enter("is_auto_accessor_property_declaration"); 
    is_property_declaration(node) && has_accessor_modifier(node)
}

fn node_arguments(node: &Arc<Node>) -> Option<Arc<NodeList>> { ::tsox_core::fntrace::enter("node_arguments"); 
    match &node.data {
        NodeData::CallExpression(d) => Some(Arc::clone(&d.arguments)),
        NodeData::NewExpression(d) => d.arguments.clone(),
        _ => None,
    }
}

pub fn is_bindable_object_define_property_call(node: &Arc<Node>) -> bool { ::tsox_core::fntrace::enter("is_bindable_object_define_property_call"); 
    let args = node_arguments(node);
    if args.as_ref().is_some_and(|args| args.nodes.len() == 3) {
        let args = args.unwrap();
        if let Some(expr) = node.expression() {
            if is_property_access_expression(expr)
                && expr.expression().is_some_and(|e| {
                    is_identifier(e) && e.text() == "Object"
                })
                && expr.name().is_some_and(|n| n.text() == "defineProperty")
                && is_string_or_numeric_literal_like(&args.nodes[1])
                && is_bindable_static_name_expression(&args.nodes[0], true)
            {
                return true;
            }
        }
    }
    false
}

pub fn is_bindable_static_access_expression(node: &Arc<Node>, exclude_this_keyword: bool) -> bool { ::tsox_core::fntrace::enter("is_bindable_static_access_expression"); 
    (is_property_access_expression(node)
        && ((!exclude_this_keyword
            && node.expression().is_some_and(|e| e.kind == SyntaxKind::ThisKeyword))
            || (node.name().is_some_and(|n| is_identifier(n))
                && node
                    .expression()
                    .is_some_and(|e| is_bindable_static_name_expression(e, true)))))
        || is_bindable_static_element_access_expression(node, exclude_this_keyword)
}

pub fn is_bindable_static_element_access_expression(
    node: &Arc<Node>,
    exclude_this_keyword: bool,
) -> bool { ::tsox_core::fntrace::enter("is_bindable_static_element_access_expression"); 
    is_literal_like_element_access(node)
        && ((!exclude_this_keyword
            && node.expression().is_some_and(|e| e.kind == SyntaxKind::ThisKeyword))
            || node.expression().is_some_and(|e| is_entity_name_expression(e))
            || node
                .expression()
                .is_some_and(|e| is_bindable_static_access_expression(e, true)))
}

pub fn is_bindable_static_name_expression(node: &Arc<Node>, exclude_this_keyword: bool) -> bool { ::tsox_core::fntrace::enter("is_bindable_static_name_expression"); 
    is_entity_name_expression(node)
        || is_bindable_static_access_expression(node, exclude_this_keyword)
}

pub fn is_block_or_catch_scoped(declaration: &Arc<Node>) -> bool { ::tsox_core::fntrace::enter("is_block_or_catch_scoped"); 
    get_combined_node_flags(declaration).contains(NodeFlags::BlockScoped)
        || is_catch_clause_variable_declaration_or_binding_element(declaration)
}

pub fn is_block_scope(node: &Node, parent_node: &Node) -> bool { ::tsox_core::fntrace::enter("is_block_scope"); 
    match node.kind {
        SyntaxKind::SourceFile
        | SyntaxKind::CaseBlock
        | SyntaxKind::CatchClause
        | SyntaxKind::ModuleDeclaration
        | SyntaxKind::ForStatement
        | SyntaxKind::ForInStatement
        | SyntaxKind::ForOfStatement
        | SyntaxKind::Constructor
        | SyntaxKind::MethodDeclaration
        | SyntaxKind::GetAccessor
        | SyntaxKind::SetAccessor
        | SyntaxKind::FunctionDeclaration
        | SyntaxKind::FunctionExpression
        | SyntaxKind::ArrowFunction
        | SyntaxKind::PropertyDeclaration
        | SyntaxKind::ClassStaticBlockDeclaration => true,
        SyntaxKind::Block => !is_function_like_or_class_static_block_declaration(parent_node),
        _ => false,
    }
}

fn select_expression_of_call_or_new_expression_or_decorator(node: &Arc<Node>) -> Option<Arc<Node>> { ::tsox_core::fntrace::enter("select_expression_of_call_or_new_expression_or_decorator"); 
    if is_call_expression(node) || is_new_expression(node) || is_decorator(node) {
        return node.expression().map(Arc::clone);
    }
    None
}

pub(crate) fn is_callee_worker(
    node: &Arc<Node>,
    pred: fn(&Node) -> bool,
    include_element_access: bool,
    skip_past_outer_expressions: bool,
) -> bool { ::tsox_core::fntrace::enter("is_callee_worker"); 
    let target = if include_element_access {
        climb_past_property_or_element_access(node)
    } else {
        climb_past_property_access(node)
    };
    let mut target = target;
    if skip_past_outer_expressions && is_expression(&target) {
        target = skip_outer_expressions(&target, OuterExpressionKinds::ALL);
    }
    match target.parent() {
        Some(parent) => {
            pred(&parent)
                && select_expression_of_call_or_new_expression_or_decorator(&parent)
                    .is_some_and(|expr| Arc::ptr_eq(&expr, &target))
        }
        None => false,
    }
}

pub fn is_call_expression_target(
    node: &Arc<Node>,
    include_element_access: bool,
    skip_past_outer_expressions: bool,
) -> bool { ::tsox_core::fntrace::enter("is_call_expression_target"); 
    is_callee_worker(
        node,
        is_call_expression,
        include_element_access,
        skip_past_outer_expressions,
    )
}

pub fn is_call_or_new_expression_target(
    node: &Arc<Node>,
    include_element_access: bool,
    skip_past_outer_expressions: bool,
) -> bool { ::tsox_core::fntrace::enter("is_call_or_new_expression_target"); 
    is_callee_worker(
        node,
        is_call_or_new_expression,
        include_element_access,
        skip_past_outer_expressions,
    )
}
