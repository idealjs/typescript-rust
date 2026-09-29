use std::sync::Arc;

use tsox_frontend::ast::mig::m3b::{members, parameters};
use tsox_frontend::ast::mig::m3g_3::OuterExpressionKinds;
use tsox_frontend::ast::mig::m3g_2::is_this_parameter;
use tsox_frontend::ast::mig::m3g_3::{node_is_decorated, node_or_child_is_decorated};
use tsox_frontend::ast::mig::w3::get_all_accessor_declarations;
use tsox_frontend::ast::node::Node;
use tsox_frontend::ast::node_data_generated::{is_class_static_block_declaration, NodeData};
use tsox_frontend::ast::syntax_kind_generated::SyntaxKind;
use tsox_frontend::ast::utilities::{is_accessor, is_class_element, has_static_modifier};

use crate::mig::m4g::r33k7_defs::has_decorators;

pub const OUTER_EXPRESSION_KINDS_ALL_EXCEPT_ASSERTIONS_OR_EXPRESSIONS_WITH_TYPE_ARGUMENTS:
    OuterExpressionKinds = OuterExpressionKinds::ALL
    .difference(
        OuterExpressionKinds::TYPE_ASSERTIONS
            .union(OuterExpressionKinds::NON_NULL_ASSERTIONS)
            .union(OuterExpressionKinds::SATISFIES),
    )
    .difference(OuterExpressionKinds::EXPRESSIONS_WITH_TYPE_ARGUMENTS);

pub fn is_static(node: &Node) -> bool {
    is_class_element(node) && has_static_modifier(node) || is_class_static_block_declaration(node)
}

pub fn child_is_decorated(
    use_legacy_decorators: bool,
    node: &Arc<Node>,
    parent: Option<&Arc<Node>>,
) -> bool {
    match node.kind {
        SyntaxKind::ClassDeclaration | SyntaxKind::ClassExpression => members(node)
            .iter()
            .any(|m| node_or_child_is_decorated(use_legacy_decorators, m, Some(node), parent)),
        SyntaxKind::MethodDeclaration | SyntaxKind::SetAccessor | SyntaxKind::Constructor => {
            parameters(node)
                .iter()
                .any(|p| node_is_decorated(use_legacy_decorators, p, Some(node), parent))
        }
        _ => false,
    }
}

pub fn class_element_or_class_element_parameter_is_decorated(
    use_legacy_decorators: bool,
    node: &Arc<Node>,
    parent: &Arc<Node>,
) -> bool {
    let mut parameter_list: Option<&[Arc<Node>]> = None;
    let decls;
    if is_accessor(node) {
        decls = get_all_accessor_declarations(members(parent), node);
        let first_accessor_with_decorators =
            if decls.first_accessor.as_ref().is_some_and(|a| has_decorators(a)) {
                decls.first_accessor.clone()
            } else if decls.second_accessor.as_ref().is_some_and(|a| has_decorators(a)) {
                decls.second_accessor.clone()
            } else {
                None
            };
        let Some(first_accessor_with_decorators) = first_accessor_with_decorators else {
            return false;
        };
        if !Arc::ptr_eq(&first_accessor_with_decorators, node) {
            return false;
        }
        if let Some(set_accessor) = &decls.set_accessor {
            parameter_list = Some(parameters(set_accessor));
        }
    } else if node.kind == SyntaxKind::MethodDeclaration {
        parameter_list = Some(parameters(node));
    }
    if node_is_decorated(use_legacy_decorators, node, Some(parent), None) {
        return true;
    }
    if let Some(parameter_list) = parameter_list {
        for parameter in parameter_list {
            if is_this_parameter(parameter) {
                continue;
            }
            if node_is_decorated(use_legacy_decorators, parameter, Some(node), Some(parent)) {
                return true;
            }
        }
    }
    false
}

pub fn get_innermost_module_declaration_from_dotted_module(
    module_declaration: &Arc<Node>,
) -> Arc<Node> {
    let mut current = Arc::clone(module_declaration);
    loop {
        let next = match &current.data {
            NodeData::ModuleDeclaration(d) => match &d.body {
                Some(body) if body.kind == SyntaxKind::ModuleDeclaration => {
                    Some(Arc::clone(body))
                }
                _ => None,
            },
            _ => None,
        };
        match next {
            Some(next) => current = next,
            None => return current,
        }
    }
}
