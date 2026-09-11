use std::sync::Arc;

use tsox_checker::checker::Checker;
use tsox_checker::checker::types::{ContextFlags, TYPE_FLAGS_PRIMITIVE, Type, TypeData, TypeFlags};
use tsox_frontend::ast::{Node, NodeData, SourceFile, Symbol, SyntaxKind};
use tsox_frontend::scanner::skip_trivia;

use super::completions_accessibility::is_non_public_member;
use super::completions_context::relevant_tokens;
use super::completions_object_like_types::binding_pattern_type_members;

/// 光标所在的最内层物体字面量/解构模式（不跨越函数边界）
fn find_object_like_container(node: &Arc<Node>) -> Option<Arc<Node>> {
    let mut current = Some(Arc::clone(node));
    while let Some(n) = current {
        match n.kind {
            SyntaxKind::ObjectLiteralExpression | SyntaxKind::ObjectBindingPattern => {
                return Some(n);
            }
            SyntaxKind::ArrowFunction
            | SyntaxKind::FunctionExpression
            | SyntaxKind::FunctionDeclaration
            | SyntaxKind::MethodDeclaration
            | SyntaxKind::ClassDeclaration
            | SyntaxKind::ClassExpression
            | SyntaxKind::SourceFile => return None,
            _ => {}
        }
        current = n.parent.clone();
    }
    None
}

/// 命中物体语境返回容器；context token 为 `{` / `,` / `async` / `*` 才属成员名位
pub(super) fn try_get_object_like_container(
    node_at_position: &Arc<Node>,
    text: &str,
    jsx: bool,
    position: usize,
) -> Option<Arc<Node>> {
    let container = find_object_like_container(node_at_position)?;
    let (context, _previous) = relevant_tokens(text, jsx, container.pos(), position);
    let context = context?;
    if matches!(
        context.kind,
        SyntaxKind::OpenBraceToken | SyntaxKind::CommaToken | SyntaxKind::AsyncKeyword
    ) {
        return Some(container);
    }
    if context.kind == SyntaxKind::AsteriskToken
        && container.kind == SyntaxKind::ObjectLiteralExpression
    {
        return Some(container);
    }
    None
}


/// Go tryGetObjectLikeCompletionSymbols：Some(成员) 表示命中物体语境（可能为空
/// 列表，不回退 scope）；None 表示继续走后续搜索
pub(super) fn object_like_completion(
    checker: &mut Checker,
    file: &Arc<SourceFile>,
    container: &Arc<Node>,
    text: &str,
    position: usize,
) -> Option<Vec<Arc<Symbol>>> {
    let type_members = if container.kind == SyntaxKind::ObjectLiteralExpression {
        object_literal_type_members(checker, container)?
    } else {
        binding_pattern_type_members(checker, file, container)?
    };
    let existing = container_children(container);
    Some(filter_object_members(
        type_members,
        &existing,
        text,
        position,
    ))
}

fn container_children(container: &Arc<Node>) -> Vec<Arc<Node>> {
    match &container.data {
        NodeData::ObjectLiteralExpression(d) => d.properties.nodes.clone(),
        NodeData::BindingPattern(d) => d.elements.nodes.clone(),
        _ => Vec::new(),
    }
}

fn object_literal_type_members(
    checker: &mut Checker,
    container: &Arc<Node>,
) -> Option<Vec<Arc<Symbol>>> {
    let instantiated = match checker.get_contextual_type(container, ContextFlags::None) {
        Some(t) => t,
        None => try_get_object_literal_contextual_type(checker, container)?,
    };
    let completions_type =
        checker.get_contextual_type(container, ContextFlags::IgnoreNodeInferences);
    let effective = match &completions_type {
        Some(t) => Arc::clone(t),
        None => Arc::clone(&instantiated),
    };
    let number_index = checker.get_number_index_type(&effective);
    let members = properties_for_object_expression(checker, &instantiated, container);
    if members.is_empty() && number_index.is_none() {
        return None;
    }
    Some(members)
}

fn try_get_object_literal_contextual_type(
    checker: &mut Checker,
    container: &Arc<Node>,
) -> Option<Arc<Type>> {
    let parent = walk_up_parenthesized_expressions(container.parent.as_ref()?);
    if parent.kind == SyntaxKind::BinaryExpression
        && let NodeData::BinaryExpression(d) = &parent.data
        && d.operator_token.kind == SyntaxKind::EqualsToken
        && Arc::ptr_eq(&d.left, container)
    {
        return Some(checker.get_type_of_node(&parent));
    }
    checker.get_contextual_type(&parent, ContextFlags::None)
}

fn walk_up_parenthesized_expressions(node: &Arc<Node>) -> Arc<Node> {
    let mut current = Arc::clone(node);
    while current.kind == SyntaxKind::ParenthesizedExpression
        && let Some(parent) = &current.parent
    {
        current = Arc::clone(parent);
    }
    current
}

pub(super) fn properties_for_object_expression(
    checker: &mut Checker,
    contextual_type: &Arc<Type>,
    obj: &Arc<Node>,
) -> Vec<Arc<Symbol>> {
    if !contextual_type.flags.contains(TypeFlags::Union) {
        return checker.get_apparent_properties(contextual_type);
    }
    let filtered: Vec<Arc<Type>> = union_member_types(contextual_type)
        .into_iter()
        .filter(|t| !is_union_member_excluded(checker, t, obj))
        .collect();
    if filtered.is_empty() {
        return Vec::new();
    }
    checker.get_all_possible_properties_of_types(&filtered)
}

fn is_union_member_excluded(checker: &mut Checker, t: &Arc<Type>, obj: &Arc<Node>) -> bool {
    // Go getApparentProperties 的联合成员五条件：primitive、array-like、
    // 判别已失效（isTypeInvalidDueToUnionDiscriminant）、调用/构造签名、
    // 含非公开成员的 class
    if t.flags.intersects(TYPE_FLAGS_PRIMITIVE) {
        return true;
    }
    if checker.is_array_type(t)
        || checker.is_array_like_type(t)
        || extends_array_like(checker, t)
    {
        return true;
    }
    if checker.is_type_invalid_due_to_union_discriminant(t, obj) {
        return true;
    }
    if checker.type_has_call_or_construct_signatures(t) {
        return true;
    }
    // 类实例型未带 Class 旗标时按符号声明回判（Go IsClass 语义）
    let is_class_like = t.is_class()
        || t.symbol.as_ref().is_some_and(|s| {
            s.declarations
                .iter()
                .any(|d| matches!(d.kind, SyntaxKind::ClassDeclaration | SyntaxKind::ClassExpression))
        });
    if is_class_like {
        let props = checker.get_apparent_properties(t);
        return props.iter().any(is_non_public_member);
    }
    false
}

// Go isArrayLikeType 的 isTypeAssignableTo(anyReadonlyArrayType) 近似：
// 接口/引用目标的基类链走到全局 Array/ReadonlyArray（Many extends
// ReadonlyArray 形态）
fn extends_array_like(checker: &mut Checker, t: &Arc<Type>) -> bool {
    let Some(target) = t.target() else {
        return false;
    };
    let mut stack: Vec<Arc<Type>> = vec![Arc::clone(&target)];
    let mut guard = 0;
    while let Some(cur) = stack.pop() {
        guard += 1;
        if guard > 16 {
            return false;
        }
        if let Some(sym) = cur.symbol.clone()
            && matches!(sym.name.as_str(), "Array" | "ReadonlyArray")
        {
            return true;
        }
        stack.extend(checker.get_base_types(&cur));
    }
    false
}

fn filter_object_members(
    contextual: Vec<Arc<Symbol>>,
    existing: &[Arc<Node>],
    text: &str,
    position: usize,
) -> Vec<Arc<Symbol>> {
    if existing.is_empty() {
        return contextual;
    }
    let mut existing_names: std::collections::HashSet<String> = std::collections::HashSet::new();
    for member in existing {
        if !matches!(
            member.kind,
            SyntaxKind::PropertyAssignment
                | SyntaxKind::ShorthandPropertyAssignment
                | SyntaxKind::BindingElement
                | SyntaxKind::MethodDeclaration
                | SyntaxKind::GetAccessor
                | SyntaxKind::SetAccessor
                | SyntaxKind::SpreadAssignment
        ) {
            continue;
        }
        if member.kind == SyntaxKind::SpreadAssignment {
            continue;
        }
        if is_currently_editing_node(member, text, position) {
            continue;
        }
        if let Some(name) = existing_member_name(member) {
            existing_names.insert(name);
        }
    }
    contextual
        .into_iter()
        .filter(|m| !existing_names.contains(&m.name))
        .collect()
}

fn existing_member_name(member: &Arc<Node>) -> Option<String> {
    let name = match &member.data {
        NodeData::BindingElement(d) => d.property_name.clone().or_else(|| d.name.clone()),
        _ => member.name().cloned(),
    }?;
    if matches!(
        name.kind,
        SyntaxKind::Identifier | SyntaxKind::StringLiteral | SyntaxKind::NumericLiteral
    ) {
        return Some(name.text().to_string());
    }
    None
}

pub(super) fn is_currently_editing_node(node: &Arc<Node>, text: &str, position: usize) -> bool {
    let start = skip_trivia(text, node.pos());
    start <= position && position <= node.end()
}

pub(super) fn union_member_types(t: &Arc<Type>) -> Vec<Arc<Type>> {
    match &t.data {
        TypeData::Union(u) => u.union_or_intersection.types.clone(),
        _ => vec![Arc::clone(t)],
    }
}

pub(super) fn enclosing_binding_element(node: &Arc<Node>) -> bool {
    let mut current = Some(Arc::clone(node));
    while let Some(n) = current {
        if n.kind == SyntaxKind::BindingElement {
            return true;
        }
        if matches!(
            n.kind,
            SyntaxKind::ObjectBindingPattern
                | SyntaxKind::ArrayBindingPattern
                | SyntaxKind::VariableDeclaration
                | SyntaxKind::SourceFile
        ) {
            return false;
        }
        current = n.parent.clone();
    }
    false
}

/// Go NodeFlagsInWithStatement：对象字面量位于 with 语句内
pub(super) fn in_with_statement(container: &Arc<Node>) -> bool {
    let mut cur = Some(Arc::clone(container));
    while let Some(n) = cur {
        if n.kind == SyntaxKind::WithStatement {
            return true;
        }
        if n.kind == SyntaxKind::SourceFile {
            break;
        }
        cur = n.parent.clone();
    }
    false
}
