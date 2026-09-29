//! Go checker getContextualTypeForChildJsxExpression：JSX children 位置
//! 表达式的上下文类型取自组件 props 上 JSX.ElementChildrenAttribute
//! 指名的属性；children 回调（`{ user => ... }`）的形参类型由此推导

use std::sync::Arc;

use tsox_checker::checker::types::{AccessFlags, SignatureKind, Type};
use tsox_checker::checker::Checker;
use tsox_frontend::ast::Node;
use tsox_frontend::ast::NodeData;
use tsox_frontend::ast::SyntaxKind;

use super::completions_jsx_attributes::jsx_namespace_container_member_name;

/// children 回调形参的上下文类型：identifier 解析到某函数表达式（JsxExpression
/// 挂在 JsxElement children 上）的无注解形参符号时，取上下文签名对应位置的
/// 形参类型
pub(super) fn jsx_children_param_type(
    checker: &mut Checker,
    identifier: &Arc<Node>,
) -> Option<Arc<Type>> {
    let sym = checker.resolve_identifier(identifier)?;
    let param = sym
        .declarations
        .iter()
        .find(|d| d.kind == SyntaxKind::Parameter)
        .cloned()?;
    if matches!(&param.data, NodeData::ParameterDeclaration(d) if d.type_node.is_some()) {
        return None;
    }
    let fn_node = param.parent()?;
    if !matches!(
        fn_node.kind,
        SyntaxKind::ArrowFunction | SyntaxKind::FunctionExpression
    ) {
        return None;
    }
    let param_index = fn_parameters(&fn_node)
        .into_iter()
        .position(|p| Arc::ptr_eq(&p, &param))?;
    let jsx_expression = enclosing_jsx_child_expression(&fn_node)?;
    let contextual = jsx_children_contextual_type(checker, &jsx_expression)?;
    let sig = checker
        .get_signatures_of_type(&contextual, SignatureKind::Call)
        .into_iter()
        .next()?;
    let sig_param = sig.parameters.get(param_index)?;
    Some(checker.get_type_of_symbol(sig_param))
}

/// Go getContextualTypeForChildJsxExpression：attributes 的 apparent 上下文
/// 类型上 children 属性的类型；多语义 child 时数组型按序号取元素
fn jsx_children_contextual_type(
    checker: &mut Checker,
    jsx_expression: &Arc<Node>,
) -> Option<Arc<Type>> {
    let element = jsx_expression.parent()?;
    let opening = match &element.data {
        NodeData::JsxElement(d) => Arc::clone(&d.opening_element),
        _ => return None,
    };
    let attrs_type = component_attributes_type(checker, &element, &opening)?;
    let children_name = jsx_namespace_container_member_name(checker, "ElementChildrenAttribute")?;
    if children_name.is_empty() {
        return None;
    }
    let prop = checker.get_property_of_type(&attrs_type, &children_name)?;
    let child_field_type = checker.get_type_of_symbol(&prop);

    let children: Vec<Arc<Node>> = match &element.data {
        NodeData::JsxElement(d) => d.children.iter().cloned().collect(),
        _ => Vec::new(),
    };
    // Go GetSemanticJsxChildren：全空白 JsxText 不算语义 child
    let semantic: Vec<Arc<Node>> = children
        .into_iter()
        .filter(|c| match &c.data {
            NodeData::JsxText(t) => !t.contains_only_trivia_white_spaces,
            _ => true,
        })
        .collect();
    let index = semantic
        .iter()
        .position(|c| Arc::ptr_eq(c, jsx_expression))?;
    if semantic.len() == 1 {
        return Some(child_field_type);
    }
    // Go mapTypeEx：union 成员逐一，数组型取第 index 个元素
    let members = match &child_field_type.data {
        tsox_checker::checker::types::TypeData::Union(u) => {
            u.union_or_intersection.types.clone()
        }
        _ => vec![Arc::clone(&child_field_type)],
    };
    let mut mapped = Vec::with_capacity(members.len());
    for t in members {
        if checker.is_array_like_type(&t) {
            let index_type = checker.get_number_literal_type(tsox_core::jsnum::Number(index as f64));
            mapped.push(checker.try_get_indexed_access_type(
                &t,
                &index_type,
                AccessFlags::Contextual,
            )?);
        } else {
            mapped.push(t);
        }
    }
    Some(checker.get_union_type_ex(mapped, Default::default()))
}

/// 组件 attributes 类型：class 构造首参（含泛型推断）优先，函数组件取
/// 调用签名首参（Go getJsxPropsTypeFromCallSignature 的主体）
fn component_attributes_type(
    checker: &mut Checker,
    element: &Arc<Node>,
    opening: &Arc<Node>,
) -> Option<Arc<Type>> {
    if let Some(t) = checker.jsx_element_attributes_contextual_type(element) {
        return Some(t);
    }
    let tag = match &opening.data {
        NodeData::JsxOpeningElement(d) => Arc::clone(&d.tag_name),
        _ => return None,
    };
    if tag.kind != SyntaxKind::Identifier {
        return None;
    }
    let sym = checker.resolve_identifier(&tag)?;
    let ty = checker.get_type_of_symbol(&sym);
    let sig = checker
        .get_signatures_of_type(&ty, SignatureKind::Call)
        .into_iter()
        .next()?;
    if sig.parameters.is_empty() {
        return None;
    }
    Some(checker.get_type_of_symbol(&sig.parameters[0]))
}

fn fn_parameters(fn_node: &Arc<Node>) -> Vec<Arc<Node>> {
    match &fn_node.data {
        NodeData::ArrowFunction(d) => d.parameters.iter().cloned().collect(),
        NodeData::FunctionExpression(d) => d.parameters.iter().cloned().collect(),
        _ => Vec::new(),
    }
}

/// 函数体宿主向上（跳过括号）是否为 JsxElement children 里的 JsxExpression
fn enclosing_jsx_child_expression(fn_node: &Arc<Node>) -> Option<Arc<Node>> {
    let mut current = fn_node.parent()?;
    while current.kind == SyntaxKind::ParenthesizedExpression {
        current = current.parent()?;
    }
    if current.kind == SyntaxKind::JsxExpression
        && current
            .parent()
            .as_ref()
            .is_some_and(|p| p.kind == SyntaxKind::JsxElement)
    {
        Some(current)
    } else {
        None
    }
}
