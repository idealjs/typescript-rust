use std::sync::Arc;

use tsox_checker::checker::Checker;
use tsox_checker::checker::types::{Type, TypeFlags};
use tsox_frontend::ast::{Node, NodeData, SourceFile, Symbol, SyntaxKind};

use super::completions_accessibility::{enclosing_class_of, is_property_accessible};
use super::completions_context::relevant_tokens;
use super::completions_object_like::union_member_types;

/// Go tryGetObjectTypeLiteralInTypeArgumentCompletionSymbols：类型实参位置的
/// 类型字面量 `f<{ /**/ }>()`，按形参约束补全成员
pub(super) fn type_literal_in_type_argument_completion(
    checker: &mut Checker,
    node_at_position: &Arc<Node>,
    text: &str,
    jsx: bool,
    position: usize,
) -> Option<Vec<Arc<Symbol>>> {
    let type_literal = find_ancestor_type_literal(node_at_position)?;
    let (context, _) = relevant_tokens(text, jsx, type_literal.pos(), position);
    let context = context?;
    if !matches!(
        context.kind,
        SyntaxKind::OpenBraceToken
            | SyntaxKind::SemicolonToken
            | SyntaxKind::CommaToken
            | SyntaxKind::Identifier
    ) {
        return None;
    }
    let parent = type_literal.parent()?;
    let container = if parent.kind == SyntaxKind::IntersectionType {
        parent
    } else {
        type_literal
    };
    let expected = constraint_of_type_argument_property(checker, &container)?;
    let actual = checker.get_type_from_type_node(&container);
    let actual_names: std::collections::HashSet<String> = checker
        .get_apparent_properties(&actual)
        .into_iter()
        .map(|p| p.name.clone())
        .collect();
    let expected_members = if expected.is_union() {
        checker.get_all_possible_properties_of_types(&expected.types().unwrap_or_default())
    } else {
        checker.get_apparent_properties(&expected)
    };
    Some(
        expected_members
            .into_iter()
            .filter(|m| !actual_names.contains(&m.name))
            .collect(),
    )
}

fn find_ancestor_type_literal(node: &Arc<Node>) -> Option<Arc<Node>> {
    let mut current = Some(Arc::clone(node));
    while let Some(n) = current {
        if n.kind == SyntaxKind::TypeLiteral {
            return Some(n);
        }
        if matches!(
            n.kind,
            SyntaxKind::SourceFile | SyntaxKind::FunctionDeclaration | SyntaxKind::Block
        ) {
            return None;
        }
        current = n.parent();
    }
    None
}

pub(super) fn constraint_of_type_argument_property(
    checker: &mut Checker,
    node: &Arc<Node>,
) -> Option<Arc<Type>> {
    if tsox_frontend::ast::is_type_node(node)
        && let Some(constraint) = checker.get_type_argument_constraint(node)
    {
        return Some(constraint);
    }
    let parent = node.parent()?;
    let t = constraint_of_type_argument_property(checker, &parent)?;
    match node.kind {
        SyntaxKind::PropertySignature => {
            let name = node.name()?.text().to_string();
            checker.get_type_of_property_of_contextual_type(&t, &name)
        }
        SyntaxKind::ColonToken => {
            if node
                .parent()
                .as_ref()
                .is_some_and(|p| p.kind == SyntaxKind::PropertySignature)
            {
                Some(t)
            } else {
                None
            }
        }
        SyntaxKind::IntersectionType | SyntaxKind::TypeLiteral | SyntaxKind::UnionType => Some(t),
        SyntaxKind::OpenBracketToken => checker.get_element_type_of_array_type(&t),
        _ => None,
    }
}


/// JS 文件中形参的类型来自 JSDoc @param 标签（Go getEffectiveTypeAnnotationNode）
fn parameter_jsdoc_type(
    checker: &mut Checker,
    file: &Arc<SourceFile>,
    param: &Arc<Node>,
) -> Option<Arc<Type>> {
    let func = param.parent()?;
    let index = function_parameter_index(&func, param)?;
    let mut position = 0usize;
    for jsdoc in file.resolve_jsdoc(&func) {
        if jsdoc.kind != SyntaxKind::JSDoc {
            continue;
        }
        let NodeData::JSDoc(d) = &jsdoc.data else {
            continue;
        };
        let Some(tags) = &d.tags else {
            continue;
        };
        for tag in tags.nodes.iter() {
            if tag.kind != SyntaxKind::JSDocParameterTag {
                continue;
            }
            let NodeData::JSDocParameterOrPropertyTag(t) = &tag.data else {
                continue;
            };
            let matched = t.name.kind == SyntaxKind::Identifier
                && param
                    .name()
                    .is_some_and(|n| n.kind == SyntaxKind::Identifier && n.text() == t.name.text());
            if !matched && position != index {
                position += 1;
                continue;
            }
            let te = t.type_expression.clone()?;
            return match &te.data {
                NodeData::JSDocTypeExpression(ted) => {
                    Some(checker.get_type_from_type_node(&ted.type_node))
                }
                _ => Some(checker.get_type_from_type_node(&te)),
            };
        }
    }
    None
}

fn function_parameter_index(func: &Arc<Node>, param: &Arc<Node>) -> Option<usize> {
    let params = match &func.data {
        NodeData::FunctionDeclaration(d) => &d.parameters,
        NodeData::FunctionExpression(d) => &d.parameters,
        NodeData::ArrowFunction(d) => &d.parameters,
        NodeData::MethodDeclaration(d) => &d.parameters,
        NodeData::ConstructorDeclaration(d) => &d.parameters,
        _ => return None,
    };
    params.iter().position(|p| Arc::ptr_eq(p, param))
}

pub(super) fn binding_pattern_type_members(
    checker: &mut Checker,
    file: &Arc<SourceFile>,
    container: &Arc<Node>,
) -> Option<Vec<Arc<Symbol>>> {
    let t = binding_pattern_type(checker, file, container)?;
    let props = if t.flags.contains(TypeFlags::Union) {
        common_properties_of_union(checker, &t)
    } else {
        checker.get_properties_of_type(&t)
    };
    Some(
        props
            .into_iter()
            .filter(|p| is_property_accessible(container, &t, p))
            .collect(),
    )
}

/// 解构模式绑定路径上的类型：逐层从外层模式类型取属性/下标（Go
/// getTypeForVariableLikeDeclaration 对 BindingElement 的处理）
fn binding_pattern_type(
    checker: &mut Checker,
    file: &Arc<SourceFile>,
    pattern: &Arc<Node>,
) -> Option<Arc<Type>> {
    let parent = pattern.parent()?;
    match parent.kind {
        SyntaxKind::BindingElement => {
            let outer_pattern = parent.parent()?;
            let outer_type = binding_pattern_type(checker, file, &outer_pattern)?;
            if outer_pattern.kind == SyntaxKind::ArrayBindingPattern {
                let index = element_index(&outer_pattern, &parent)?;
                let _ = index;
                return checker.get_element_type_of_array_type(&outer_type);
            }
            let NodeData::BindingElement(d) = &parent.data else {
                return None;
            };
            let name_node = d.property_name.clone().or_else(|| d.name.clone())?;
            checker.get_type_of_property_of_type(&outer_type, name_node.text())
        }
        SyntaxKind::VariableDeclaration => {
            let NodeData::VariableDeclaration(d) = &parent.data else {
                return None;
            };
            if let Some(type_node) = &d.type_node {
                return Some(checker.get_type_from_type_node(type_node));
            }
            if let Some(init) = &d.initializer {
                if init.kind == SyntaxKind::ThisKeyword
                    && let Some(class) = enclosing_class_of(pattern)
                {
                    return Some(checker.build_class_instance_type_with_base(&class));
                }
                return Some(checker.get_type_of_node(init));
            }
            None
        }
        SyntaxKind::Parameter => parameter_jsdoc_type(checker, file, &parent)
            .or_else(|| Some(checker.get_type_of_node(&parent))),
        _ => None,
    }
}


fn element_index(pattern: &Arc<Node>, element: &Arc<Node>) -> Option<usize> {
    let NodeData::BindingPattern(d) = &pattern.data else {
        return None;
    };
    d.elements.iter().position(|e| Arc::ptr_eq(e, element))
}

fn common_properties_of_union(checker: &mut Checker, t: &Arc<Type>) -> Vec<Arc<Symbol>> {
    let mut members = union_member_types(t);
    let Some(first) = members.first().cloned() else {
        return Vec::new();
    };
    members.remove(0);
    let mut props = checker.get_properties_of_type(&first);
    for other in &members {
        let names: std::collections::HashSet<String> = checker
            .get_properties_of_type(other)
            .into_iter()
            .map(|p| p.name.clone())
            .collect();
        props.retain(|p| names.contains(&p.name));
    }
    props
}

