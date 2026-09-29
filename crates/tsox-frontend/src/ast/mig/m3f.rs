use crate::ast::*;
use crate::ast::utilities_navigation::get_source_file_of_node;
use std::sync::Arc;

use super::m3f_2::node_initializer;
use super::m3f_4::is_declaration_binding_element;
use super::m3g::is_module_augmentation_external;
use super::m3g_2::is_signed_numeric_literal;
use crate::parser::mig::wp1_2::Pragma;

fn is_external_module_augmentation(node: &Arc<Node>) -> bool {
    is_ambient_module(node) && is_module_augmentation_external(node)
}

pub fn get_node_id(node: &Node) -> u64 {
    node.id()
}

pub fn get_symbol_id(symbol: &Arc<Symbol>) -> u64 {
    symbol.id()
}

pub fn get_symbol_table(data: &mut SymbolTable) -> &SymbolTable {
    if data.entries.is_empty() {
        *data = SymbolTable::new();
    }
    data
}

pub fn get_right_most_assigned_expression(node: &Arc<Node>) -> Arc<Node> {
    let mut current = Arc::clone(node);
    while is_assignment_expression(&current, false) {
        if let NodeData::BinaryExpression(d) = &current.data {
            current = Arc::clone(&d.right);
        } else {
            break;
        }
    }
    current
}

pub fn get_next_jsdoc_comment_location(node: &Arc<Node>) -> Option<Arc<Node>> {
    let parent = node.parent()?;
    match parent.kind {
        SyntaxKind::PropertyAssignment
        | SyntaxKind::ExportAssignment
        | SyntaxKind::PropertyDeclaration
        | SyntaxKind::VariableDeclaration
        | SyntaxKind::SatisfiesExpression
        | SyntaxKind::ReturnStatement
        | SyntaxKind::VariableStatement
        | SyntaxKind::ExpressionStatement => Some(parent),
        SyntaxKind::VariableDeclarationList => {
            if let NodeData::VariableDeclarationList(d) = &parent.data {
                if Arc::ptr_eq(&d.declarations.nodes[0], node) {
                    return Some(parent);
                }
            }
            None
        }
        _ => None,
    }
}

pub fn get_non_augmentation_declaration(symbol: &Arc<Symbol>) -> Option<Arc<Node>> {
    symbol
        .declarations
        .iter()
        .find(|d| !is_external_module_augmentation(d) && !is_global_scope_augmentation(d))
        .map(Arc::clone)
}

pub fn get_pragma_argument(pragma: Option<&Pragma>, name: &str) -> String {
    if let Some(pragma) = pragma {
        if let Some(arg) = pragma.args.iter().find(|a| a.name == name) {
            return arg.value.clone();
        }
    }
    String::new()
}

pub fn get_pragma_from_source_file<'a>(
    file: Option<&'a SourceFile>,
    name: &str,
) -> Option<&'a Pragma> {
    let _ = (file, name);
    None
}

pub fn get_property_name_for_property_name_node(name: &Arc<Node>) -> String {
    match name.kind {
        SyntaxKind::Identifier
        | SyntaxKind::PrivateIdentifier
        | SyntaxKind::StringLiteral
        | SyntaxKind::NoSubstitutionTemplateLiteral
        | SyntaxKind::NumericLiteral
        | SyntaxKind::BigIntLiteral
        | SyntaxKind::JsxNamespacedName => name.text().to_string(),
        SyntaxKind::ComputedPropertyName => {
            let name_expression = name.expression();
            if let Some(expr) = name_expression {
                if is_string_or_numeric_literal_like(expr) {
                    return expr.text().to_string();
                }
                if is_signed_numeric_literal(expr) {
                    if let NodeData::PrefixUnaryExpression(d) = &expr.data {
                        let text = d.operand.text().to_string();
                        if d.operator == SyntaxKind::MinusToken {
                            return format!("-{text}");
                        }
                        return text;
                    }
                }
            }
            INTERNAL_SYMBOL_NAME_MISSING.to_string()
        }
        _ => panic!("Unhandled case in getPropertyNameForPropertyNameNode"),
    }
}

pub fn get_reparsed_node_for_node(node: &Arc<Node>) -> Arc<Node> {
    if node.flags.contains(NodeFlags::HasJSDoc)
        && !node.flags.contains(NodeFlags::Reparsed)
        && get_source_file_of_node(node).is_some()
    {
        return Arc::clone(node);
    }
    Arc::clone(node)
}

pub fn compare_node_positions(n1: &Node, n2: &Node) -> std::cmp::Ordering {
    n1.loc.pos().cmp(&n2.loc.pos()).then(n1.loc.end().cmp(&n2.loc.end()))
}

pub fn get_rest_indicator_of_binding_or_assignment_element(
    binding_element: &Arc<Node>,
) -> Option<Arc<Node>> {
    match &binding_element.data {
        NodeData::ParameterDeclaration(d) => d.dot_dot_dot_token.as_ref().map(Arc::clone),
        NodeData::BindingElement(d) => d.dot_dot_dot_token.as_ref().map(Arc::clone),
        NodeData::SpreadElement(_) | NodeData::SpreadAssignment(_) => {
            Some(Arc::clone(binding_element))
        }
        _ => None,
    }
}

pub fn get_rest_parameter_element_type(node: Option<&Arc<Node>>) -> Option<Arc<Node>> {
    let node = node?;
    match &node.data {
        NodeData::ArrayTypeNode(d) => Some(Arc::clone(&d.element_type)),
        NodeData::TypeReferenceNode(d) => d
            .type_arguments
            .as_ref()
            .and_then(|args| args.nodes.first())
            .map(Arc::clone),
        _ => None,
    }
}

pub fn get_semantic_jsx_children(children: &[Arc<Node>]) -> Vec<Arc<Node>> {
    children
        .iter()
        .filter(|child| match child.kind {
            SyntaxKind::JsxExpression => child.expression().is_some(),
            SyntaxKind::JsxText => match &child.data {
                NodeData::JsxText(d) => !d.contains_only_trivia_white_spaces,
                _ => false,
            },
            _ => true,
        })
        .map(Arc::clone)
        .collect()
}

pub fn get_source_file_of_module(module: &Arc<Symbol>) -> Option<Arc<Node>> {
    let declaration = module.value_declaration.clone().or_else(|| {
        get_non_augmentation_declaration(module)
    })?;
    get_source_file_of_node(&declaration)
}

pub fn get_target_of_binding_or_assignment_element(binding_element: &Arc<Node>) -> Option<Arc<Node>> {
    if is_declaration_binding_element(binding_element) {
        return binding_element.name().map(Arc::clone);
    }

    if is_object_literal_element(binding_element) {
        return match binding_element.kind {
            SyntaxKind::PropertyAssignment => node_initializer(binding_element).and_then(
                |init| get_target_of_binding_or_assignment_element(init),
            ),
            SyntaxKind::ShorthandPropertyAssignment => binding_element.name().map(Arc::clone),
            SyntaxKind::SpreadAssignment => binding_element
                .expression()
                .and_then(|expr| get_target_of_binding_or_assignment_element(expr)),
            _ => None,
        };
    }

    if is_assignment_expression(binding_element, true) {
        if let NodeData::BinaryExpression(d) = &binding_element.data {
            return get_target_of_binding_or_assignment_element(&d.left);
        }
    }

    if is_spread_element(binding_element) {
        return binding_element
            .expression()
            .and_then(|expr| get_target_of_binding_or_assignment_element(expr));
    }

    Some(Arc::clone(binding_element))
}
