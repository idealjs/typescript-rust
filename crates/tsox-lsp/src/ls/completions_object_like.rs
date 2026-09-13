use std::sync::Arc;

use tsox_checker::checker::Checker;
use tsox_checker::checker::types::{
    ContextFlags, TYPE_FLAGS_ANY_OR_UNKNOWN, TYPE_FLAGS_PRIMITIVE, Type, TypeData, TypeFlags,
};
use tsox_frontend::ast::{Node, NodeData, SourceFile, Symbol, SyntaxKind};
use tsox_frontend::scanner::skip_trivia;

use super::completions_accessibility::is_non_public_member;
use super::completions_context::{ScanToken, line_of_position};
use super::completions_definition_location::token_parent;
use super::completions_object_like_types::binding_pattern_type_members;

/// Go tryGetObjectLikeCompletionContainer：contextToken.Parent 语义定位容器
pub(super) fn try_get_object_like_container(
    context_token: Option<&ScanToken>,
    text: &str,
    position: usize,
    root: &Arc<Node>,
) -> Option<Arc<Node>> {
    let context = context_token?;
    let parent = token_parent(root, context)?;
    match context.kind {
        SyntaxKind::OpenBraceToken | SyntaxKind::CommaToken => {
            if matches!(
                parent.kind,
                SyntaxKind::ObjectLiteralExpression | SyntaxKind::ObjectBindingPattern
            ) {
                Some(parent)
            } else {
                None
            }
        }
        SyntaxKind::AsteriskToken => {
            if parent.kind == SyntaxKind::MethodDeclaration
                && parent
                    .parent()
                    .as_ref()
                    .is_some_and(|p| p.kind == SyntaxKind::ObjectLiteralExpression)
            {
                parent.parent()
            } else {
                None
            }
        }
        SyntaxKind::AsyncKeyword => parent
            .parent()
            .filter(|p| p.kind == SyntaxKind::ObjectLiteralExpression),
        SyntaxKind::Identifier => {
            if &text[context.pos..context.end] == "async"
                && parent.kind == SyntaxKind::ShorthandPropertyAssignment
            {
                return parent.parent();
            }
            if let Some(grand) = parent.parent()
                && grand.kind == SyntaxKind::ObjectLiteralExpression
                && (parent.kind == SyntaxKind::SpreadAssignment
                    || (parent.kind == SyntaxKind::ShorthandPropertyAssignment
                        && line_of_position(text, context.end)
                            != line_of_position(text, position)))
            {
                return Some(grand);
            }
            property_assignment_ancestor_container(&parent, context)
        }
        _ => {
            if let Some(method) = parent.parent()
                && let Some(container) = method.parent()
                && matches!(
                    method.kind,
                    SyntaxKind::MethodDeclaration
                        | SyntaxKind::GetAccessor
                        | SyntaxKind::SetAccessor
                )
                && container.kind == SyntaxKind::ObjectLiteralExpression
            {
                return Some(container);
            }
            if parent.kind == SyntaxKind::SpreadAssignment
                && parent
                    .parent()
                    .as_ref()
                    .is_some_and(|p| p.kind == SyntaxKind::ObjectLiteralExpression)
            {
                return parent.parent();
            }
            if context.kind != SyntaxKind::ColonToken {
                return property_assignment_ancestor_container(&parent, context);
            }
            None
        }
    }
}

/// PropertyAssignment 祖先的末 token 恰为 contextToken → 容器为其父 OLE
fn property_assignment_ancestor_container(
    parent: &Arc<Node>,
    context: &ScanToken,
) -> Option<Arc<Node>> {
    let ancestor = find_ancestor_property_assignment(parent)?;
    if ancestor.loc.end() != context.end {
        return None;
    }
    ancestor
        .parent()
        .filter(|p| p.kind == SyntaxKind::ObjectLiteralExpression)
}

fn find_ancestor_property_assignment(node: &Arc<Node>) -> Option<Arc<Node>> {
    let mut current = Some(Arc::clone(node));
    while let Some(n) = current {
        if n.kind == SyntaxKind::PropertyAssignment {
            return Some(n);
        }
        current = n.parent();
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
    let members = properties_for_object_expression(
        checker,
        &instantiated,
        completions_type.as_ref(),
        container,
    );
    if members.is_empty() && number_index.is_none() {
        return None;
    }
    Some(members)
}

fn try_get_object_literal_contextual_type(
    checker: &mut Checker,
    container: &Arc<Node>,
) -> Option<Arc<Type>> {
    let parent = walk_up_parenthesized_expressions(container.parent().as_ref()?);
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
        && let Some(parent) = &current.parent()
    {
        current = Arc::clone(parent);
    }
    current
}

pub(super) fn properties_for_object_expression(
    checker: &mut Checker,
    instantiated: &Arc<Type>,
    completions_type: Option<&Arc<Type>>,
    obj: &Arc<Node>,
) -> Vec<Arc<Symbol>> {
    // Go getPropertiesForObjectExpression：t = 实例化上下文 ∪ completionsType
    //（非 any）；completionsType 存在时剔除「唯一声明在字面量自身」的成员
    //（f({abc}) 的 abc 因自声明进入 T，防自证补全）
    let has_completions_type = completions_type
        .is_some_and(|c| !Arc::ptr_eq(c, instantiated));
    let use_completions_union = has_completions_type
        && completions_type
            .is_some_and(|c| !c.flags.intersects(TYPE_FLAGS_ANY_OR_UNKNOWN));
    let mut union_types: Vec<Arc<Type>> = if instantiated.flags.contains(TypeFlags::Union) {
        union_member_types(instantiated)
    } else {
        vec![Arc::clone(instantiated)]
    };
    if use_completions_union {
        union_types.push(Arc::clone(completions_type.unwrap()));
    }
    // Go hasDeclarationOtherThanSelf 按成员声明过滤后再并入联合；本仓推断
    // 代入生成无声明合成符号，等价视作字面量自声明（先过滤后按名去重，
    // 否则首胜去重会吞掉 completionsType 一侧的具声明成员）
    let declared_elsewhere = |m: &Arc<Symbol>| {
        !m.declarations.is_empty()
            && m.declarations
                .iter()
                .any(|d| d.parent().as_ref().is_some_and(|p| !Arc::ptr_eq(p, obj)))
    };
    let mut seen: std::collections::HashSet<String> = std::collections::HashSet::new();
    let mut properties: Vec<Arc<Symbol>> = Vec::new();
    let constituents: Vec<Arc<Type>> = if union_types.len() > 1 {
        union_types
            .into_iter()
            .filter(|t| !is_union_member_excluded(checker, t, obj))
            .collect()
    } else {
        union_types
    };
    if constituents.is_empty() {
        return Vec::new();
    }
    for t in &constituents {
        let t_props = checker.get_augmented_properties_of_type(t);
        for p in t_props {            if has_completions_type && !declared_elsewhere(&p) {
                continue;
            }
            if seen.insert(p.name.clone()) {
                properties.push(p);
            }
        }
    }
    properties
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
        current = n.parent();
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
        cur = n.parent();
    }
    false
}
