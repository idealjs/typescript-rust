#![allow(unused_imports)]

use super::*;

pub(crate) fn get_assignment_target(node: &Node) -> Option<&Node> {
    pub(crate) fn is_assignment_operator(kind: SyntaxKind) -> bool {
        use SyntaxKind::*;
        matches!(
            kind,
            EqualsToken
                | PlusEqualsToken
                | MinusEqualsToken
                | AsteriskEqualsToken
                | SlashEqualsToken
                | PercentEqualsToken
                | AsteriskAsteriskEqualsToken
                | LessThanLessThanEqualsToken
                | GreaterThanGreaterThanEqualsToken
                | GreaterThanGreaterThanGreaterThanEqualsToken
                | AmpersandEqualsToken
                | BarEqualsToken
                | CaretEqualsToken
                | AmpersandAmpersandEqualsToken
                | BarBarEqualsToken
                | QuestionQuestionEqualsToken
        )
    }
    let mut current: &Node = node;
    loop {
        let parent = current.parent.as_ref()?;
        match &parent.data {
            crate::ast::NodeData::BinaryExpression(bin) => {
                let on_path = Arc::as_ref(&bin.left) as *const Node == current;
                return if on_path && is_assignment_operator(bin.operator_token.kind) {
                    Some(parent)
                } else {
                    None
                };
            }
            crate::ast::NodeData::PrefixUnaryExpression(pre) => {
                let incdec = matches!(
                    pre.operator,
                    SyntaxKind::PlusPlusToken | SyntaxKind::MinusMinusToken
                );
                let on_path = Arc::as_ref(&pre.operand) as *const Node == current;
                return if incdec && on_path {
                    Some(parent)
                } else {
                    None
                };
            }
            crate::ast::NodeData::PostfixUnaryExpression(post) => {
                let incdec = matches!(
                    post.operator,
                    SyntaxKind::PlusPlusToken | SyntaxKind::MinusMinusToken
                );
                let on_path = Arc::as_ref(&post.operand) as *const Node == current;
                return if incdec && on_path {
                    Some(parent)
                } else {
                    None
                };
            }
            crate::ast::NodeData::ForInOrOfStatement(for_stmt) => {
                let on_path = Arc::as_ref(&for_stmt.initializer) as *const Node == current;
                return if on_path { Some(parent) } else { None };
            }
            crate::ast::NodeData::ParenthesizedExpression(_) => {
                current = parent;
            }
            _ => return None,
        }
    }
}

pub fn is_compound_like_assignment(assignment: &Node) -> bool {
    let crate::ast::NodeData::BinaryExpression(bin) = &assignment.data else {
        return false;
    };
    if bin.operator_token.kind != SyntaxKind::EqualsToken {
        return false;
    }
    let mut right = &bin.right;
    while let crate::ast::NodeData::ParenthesizedExpression(p) = &right.data {
        right = &p.expression;
    }
    matches!(&right.data, crate::ast::NodeData::BinaryExpression(rhs)
        if is_shift_operator_or_higher(rhs.operator_token.kind))
}

pub fn is_in_compound_like_assignment(node: &Node) -> bool {
    let Some(target) = get_assignment_target(node) else {
        return false;
    };

    let crate::ast::NodeData::BinaryExpression(bin) = &target.data else {
        return false;
    };
    bin.operator_token.kind == SyntaxKind::EqualsToken && is_compound_like_assignment(target)
}

pub fn is_delete_target(node: &Node) -> bool {
    if !crate::ast::is_access_expression(node) {
        return false;
    }
    node.parent
        .as_ref()
        .map(|p| p.kind == SyntaxKind::DeleteExpression)
        .unwrap_or(false)
}

pub fn is_right_side_of_access_expression(node: &Node) -> bool {
    if let Some(parent) = &node.parent {
        if is_property_access_expression(parent) {
            return parent
                .name()
                .map(|n| std::ptr::eq(n.as_ref(), node))
                .unwrap_or(false);
        }
        if is_element_access_expression(parent) {
            return parent
                .expression()
                .map(|e| std::ptr::eq(e.as_ref(), node))
                .unwrap_or(false);
        }
    }
    false
}

pub fn is_top_level_in_external_module_augmentation(node: &Node) -> bool {
    let _ = node;
    false
}

pub fn is_syntactic_default(node: &Node) -> bool {
    matches!(
        node.kind,
        SyntaxKind::ExportSpecifier | SyntaxKind::NamespaceExportDeclaration
    ) || node.has_syntactic_modifier(ModifierFlags::Default)
}

pub fn is_type_reference_identifier(node: &Node) -> bool {
    node.parent
        .as_ref()
        .map(|p| crate::ast::is_type_reference_node(p))
        .unwrap_or(false)
}

pub fn is_in_type_query(node: &Node) -> bool {
    let _ = node;
    false
}

pub fn is_side_effect_import(node: &Node) -> bool {
    let _ = node;
    false
}

pub fn get_external_module_require_argument(node: &Node) -> Option<Arc<Node>> {
    let _ = node;
    None
}

pub fn is_shorthand_ambient_module(node: &Node) -> bool {
    node.kind == SyntaxKind::ModuleDeclaration
}

pub fn is_shorthand_ambient_module_symbol(module_symbol: &Symbol) -> bool {
    module_symbol
        .value_declaration
        .as_ref()
        .map(|d| is_shorthand_ambient_module(d))
        .unwrap_or(false)
}

pub fn entity_name_to_string(name: &Node) -> String {
    name.text().to_string()
}

pub fn get_containing_qualified_name_node(node: &Arc<Node>) -> Arc<Node> {
    let mut result = Arc::clone(node);
    let mut current = node.parent.clone();
    while let Some(ref parent) = current {
        if is_qualified_name(parent) {
            result = Arc::clone(parent);
            current = parent.parent.clone();
        } else {
            break;
        }
    }
    result
}

pub fn is_const_type_reference(node: &Node) -> bool {
    crate::ast::is_type_reference_node(node) && node.text() == "const"
}

pub fn get_single_variable_of_variable_statement(node: &Node) -> Option<Arc<Node>> {
    let _ = node;
    None
}

pub fn is_jsx_intrinsic_tag_name(tag_name: &Node) -> bool {
    crate::ast::is_identifier(tag_name) || crate::ast::is_jsx_namespaced_name(tag_name)
}

pub fn walk_up_outer_expressions(node: &Node) -> Option<Arc<Node>> {
    node.parent.clone()
}

pub fn get_containing_function_or_class_static_block(node: &Node) -> Option<Arc<Node>> {
    node.parent.as_ref().and_then(|parent| {
        crate::ast::find_ancestor(parent, |n| {
            crate::ast::is_function_like_or_class_static_block_declaration(n)
        })
    })
}

pub fn get_enclosing_container(node: &Node) -> Option<Arc<Node>> {
    node.parent.as_ref().and_then(|parent| {
        crate::ast::find_ancestor(parent, |n| {
            matches!(
                n.kind,
                SyntaxKind::FunctionDeclaration
                    | SyntaxKind::FunctionExpression
                    | SyntaxKind::ArrowFunction
                    | SyntaxKind::MethodDeclaration
                    | SyntaxKind::GetAccessor
                    | SyntaxKind::SetAccessor
                    | SyntaxKind::Constructor
                    | SyntaxKind::ClassDeclaration
                    | SyntaxKind::ClassExpression
                    | SyntaxKind::ModuleDeclaration
                    | SyntaxKind::SourceFile
            )
        })
    })
}

pub fn is_this_initialized_declaration(node: &Node) -> bool {
    crate::ast::is_variable_declaration(node)
        && node
            .expression()
            .map(|e| e.kind == SyntaxKind::ThisKeyword)
            .unwrap_or(false)
}

pub fn is_declaration_readonly(declaration: &Arc<Node>) -> bool {
    get_combined_modifier_flags(declaration).contains(ModifierFlags::Readonly)
}

pub fn get_binding_element_property_name(node: &Node) -> Option<Arc<Node>> {
    node.name().cloned()
}

pub fn is_valid_number_string(s: &str, round_trip_only: bool) -> bool {
    if s.is_empty() {
        return false;
    }
    let n = crate::jsnum::Number::from_string(s);
    !n.is_nan() && !n.is_inf() && (!round_trip_only || n.to_string() == s)
}

pub fn is_valid_big_int_string(_s: &str, _round_trip_only: bool) -> bool {
    false
}

pub fn is_valid_es_symbol_declaration(node: &Node) -> bool {
    let _ = node;
    false
}

pub fn is_variable_declaration_in_variable_statement(node: &Node) -> bool {
    node.parent
        .as_ref()
        .map(|p| is_variable_declaration_list(p))
        .unwrap_or(false)
        && node
            .parent
            .as_ref()
            .and_then(|p| p.parent.as_ref())
            .map(|gp| is_variable_statement(gp))
            .unwrap_or(false)
}

pub fn is_in_ambient_or_type_node(node: &Node) -> bool {
    if node.flags.contains(NodeFlags::Ambient) {
        return true;
    }

    false
}
