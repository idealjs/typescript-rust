use crate::ast::*;
use std::sync::Arc;

use super::m3b::get_resolution_mode_override;
use super::m3f::compare_node_positions;
use super::m3f_4::is_dynamic_name;
use super::m3g_2::is_this_parameter;
use super::m3g_3::try_get_import_from_module_specifier;

pub fn node_initializer(node: &Node) -> Option<&Arc<Node>> {
    match &node.data {
        NodeData::VariableDeclaration(d) => d.initializer.as_ref(),
        NodeData::ParameterDeclaration(d) => d.initializer.as_ref(),
        NodeData::BindingElement(d) => d.initializer.as_ref(),
        NodeData::PropertyDeclaration(d) => d.initializer.as_ref(),
        NodeData::PropertyAssignment(d) => Some(&d.initializer),
        NodeData::EnumMember(d) => d.initializer.as_ref(),
        NodeData::ForStatement(d) => d.initializer.as_ref(),
        NodeData::ForInOrOfStatement(d) => Some(&d.initializer),
        NodeData::JsxAttribute(d) => d.initializer.as_ref(),
        _ => None,
    }
}

pub fn node_parameters(node: &Node) -> Option<&Arc<NodeList>> {
    match &node.data {
        NodeData::FunctionDeclaration(d) => Some(&d.parameters),
        NodeData::FunctionExpression(d) => Some(&d.parameters),
        NodeData::ArrowFunction(d) => Some(&d.parameters),
        NodeData::ConstructorDeclaration(d) => Some(&d.parameters),
        NodeData::MethodDeclaration(d) => Some(&d.parameters),
        NodeData::GetAccessorDeclaration(d) => Some(&d.parameters),
        NodeData::SetAccessorDeclaration(d) => Some(&d.parameters),
        NodeData::MethodSignatureDeclaration(d) => Some(&d.parameters),
        NodeData::CallSignatureDeclaration(d) => Some(&d.parameters),
        NodeData::ConstructSignatureDeclaration(d) => Some(&d.parameters),
        NodeData::FunctionTypeNode(d) => Some(&d.parameters),
        NodeData::ConstructorTypeNode(d) => Some(&d.parameters),
        _ => None,
    }
}

pub fn node_type_parameters(node: &Node) -> Option<&Arc<NodeList>> {
    match &node.data {
        NodeData::FunctionDeclaration(d) => d.type_parameters.as_ref(),
        NodeData::FunctionExpression(d) => d.type_parameters.as_ref(),
        NodeData::ArrowFunction(d) => d.type_parameters.as_ref(),
        NodeData::ClassDeclaration(d) => d.type_parameters.as_ref(),
        NodeData::ClassExpression(d) => d.type_parameters.as_ref(),
        NodeData::InterfaceDeclaration(d) => d.type_parameters.as_ref(),
        NodeData::TypeAliasDeclaration(d) => d.type_parameters.as_ref(),
        NodeData::MethodDeclaration(d) => d.type_parameters.as_ref(),
        NodeData::MethodSignatureDeclaration(d) => d.type_parameters.as_ref(),
        NodeData::ConstructorDeclaration(d) => d.type_parameters.as_ref(),
        NodeData::GetAccessorDeclaration(d) => d.type_parameters.as_ref(),
        NodeData::SetAccessorDeclaration(d) => d.type_parameters.as_ref(),
        NodeData::CallSignatureDeclaration(d) => d.type_parameters.as_ref(),
        NodeData::ConstructSignatureDeclaration(d) => d.type_parameters.as_ref(),
        NodeData::FunctionTypeNode(d) => d.type_parameters.as_ref(),
        NodeData::ConstructorTypeNode(d) => d.type_parameters.as_ref(),
        _ => None,
    }
}

pub fn question_token(node: &Node) -> Option<&Arc<Node>> {
    match &node.data {
        NodeData::ParameterDeclaration(d) => d.question_token.as_ref(),
        NodeData::ConditionalExpression(d) => Some(&d.question_token),
        NodeData::MappedTypeNode(d) => d.question_token.as_ref(),
        NodeData::NamedTupleMember(d) => d.question_token.as_ref(),
        NodeData::PropertySignatureDeclaration(d) => d.postfix_token.as_ref(),
        NodeData::MethodSignatureDeclaration(d) => d.postfix_token.as_ref(),
        NodeData::MethodDeclaration(d) => d.postfix_token.as_ref(),
        _ => None,
    }
}

pub fn has_abstract_modifier(node: &Node) -> bool {
    has_syntactic_modifier(node, ModifierFlags::Abstract)
}

pub fn has_ambient_modifier(node: &Node) -> bool {
    has_syntactic_modifier(node, ModifierFlags::Ambient)
}

pub fn has_context_sensitive_parameters(node: &Node) -> bool {
    if node_type_parameters(node).is_none() {
        if let Some(parameters) = node_parameters(node) {
            if parameters.nodes.iter().any(|p| p.type_node().is_none()) {
                return true;
            }
            if !is_arrow_function(node) {
                let parameter = parameters.nodes.first();
                if parameter.is_none()
                    || !parameter.is_some_and(|p| is_this_parameter(p))
                {
                    return node.flags.contains(NodeFlags::ContainsThis);
                }
            }
        }
    }
    false
}

pub fn has_dynamic_name(declaration: &Arc<Node>) -> bool {
    get_name_of_declaration(declaration).is_some_and(|name| is_dynamic_name(&name))
}

pub fn has_import_attributes(node: &Node) -> bool {
    matches!(
        node.kind,
        SyntaxKind::ImportDeclaration
            | SyntaxKind::JSImportDeclaration
            | SyntaxKind::ExportDeclaration
            | SyntaxKind::ImportType
    )
}

pub fn has_inferred_type(node: &Node) -> bool {
    matches!(
        node.kind,
        SyntaxKind::Parameter
            | SyntaxKind::PropertySignature
            | SyntaxKind::PropertyDeclaration
            | SyntaxKind::BindingElement
            | SyntaxKind::PropertyAccessExpression
            | SyntaxKind::ElementAccessExpression
            | SyntaxKind::BinaryExpression
            | SyntaxKind::CallExpression
            | SyntaxKind::VariableDeclaration
            | SyntaxKind::ExportAssignment
            | SyntaxKind::PropertyAssignment
            | SyntaxKind::ShorthandPropertyAssignment
            | SyntaxKind::JSDocParameterTag
            | SyntaxKind::JSDocPropertyTag
    )
}

pub fn has_initializer(node: &Node) -> bool {
    match node.kind {
        SyntaxKind::VariableDeclaration
        | SyntaxKind::Parameter
        | SyntaxKind::BindingElement
        | SyntaxKind::PropertyDeclaration
        | SyntaxKind::PropertyAssignment
        | SyntaxKind::EnumMember
        | SyntaxKind::ForStatement
        | SyntaxKind::ForInStatement
        | SyntaxKind::ForOfStatement
        | SyntaxKind::JsxAttribute => node_initializer(node).is_some(),
        _ => false,
    }
}

pub fn has_modifier(node: &Node, flags: ModifierFlags) -> bool {
    node.syntactic_modifier_flags().intersects(flags)
}

pub fn has_question_token(node: &Node) -> bool {
    is_question_token(question_token(node).map(Arc::as_ref))
}

pub fn has_resolution_mode_override(node: Option<&Arc<Node>>) -> bool {
    let node = match node {
        Some(n) => n,
        None => return false,
    };
    let attributes: Option<&Arc<Node>> = match &node.data {
        NodeData::ImportTypeNode(d) => d.attributes.as_ref(),
        NodeData::ImportDeclaration(d) => d.attributes.as_ref(),
        NodeData::ExportDeclaration(d) => d.attributes.as_ref(),
        _ => None,
    };
    if let Some(attributes) = attributes {
        return get_resolution_mode_override(attributes, None).1;
    }
    false
}

pub fn has_same_property_access_name(node1: &Arc<Node>, node2: &Arc<Node>) -> bool {
    if node1.kind == SyntaxKind::Identifier && node2.kind == SyntaxKind::Identifier {
        node1.text() == node2.text()
    } else if node1.kind == SyntaxKind::PropertyAccessExpression
        && node2.kind == SyntaxKind::PropertyAccessExpression
    {
        node1.name().is_some_and(|n1| {
            node2.name().is_some_and(|n2| {
                n1.text() == n2.text()
                    && node1
                        .expression()
                        .is_some_and(|e1| {
                            node2.expression().is_some_and(|e2| {
                                has_same_property_access_name(e1, e2)
                            })
                        })
            })
        })
    } else {
        false
    }
}

pub fn has_type_arguments(node: &Node) -> bool {
    matches!(
        node.kind,
        SyntaxKind::CallExpression
            | SyntaxKind::NewExpression
            | SyntaxKind::TaggedTemplateExpression
            | SyntaxKind::TypeReference
            | SyntaxKind::ExpressionWithTypeArguments
            | SyntaxKind::ImportType
            | SyntaxKind::TypeQuery
            | SyntaxKind::JsxOpeningElement
            | SyntaxKind::JsxSelfClosingElement
    )
}

pub fn import_from_module_specifier(node: &Arc<Node>) -> Option<Arc<Node>> {
    if let Some(result) = try_get_import_from_module_specifier(node) {
        return Some(result);
    }
    None
}

pub fn index_of_node(nodes: &[Arc<Node>], node: &Arc<Node>) -> Option<usize> {
    let mut lo = 0usize;
    let mut hi = nodes.len();
    while lo < hi {
        let mid = (lo + hi) / 2;
        match compare_node_positions(&nodes[mid], node) {
            std::cmp::Ordering::Less => lo = mid + 1,
            std::cmp::Ordering::Equal => return Some(mid),
            std::cmp::Ordering::Greater => hi = mid,
        }
    }
    None
}
