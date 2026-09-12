//! Go completions.go tryGetJsxCompletionSymbols：contextToken 位于 JsxAttribute /
//! JsxSpreadAttribute / JsxExpression / JsxOpeningLikeElement 时，列出
//! opening element 名字类型的属性（intrinsic 走 JSX.IntrinsicElements[tag]；
//! class 组件走 JSX.ElementAttributesProperty 容器名 / 构造签名首参），
//! 再按已写属性名过滤（filterJsxAttributes）

use std::collections::HashSet;
use std::sync::Arc;

use tsox_frontend::ast::Node;
use tsox_frontend::ast::NodeData;
use tsox_frontend::ast::Symbol;
use tsox_frontend::ast::SyntaxKind;
use tsox_frontend::scanner::skip_trivia;
use tsox_checker::checker::Checker;

use super::completions_jsx_closing_tag::jsx_tag_name_text;

/// Some(符号列表) 表示命中 JSX 属性语境；None 表示继续后续补全路径
pub(super) fn jsx_attribute_completion(
    checker: &mut Checker,
    file_text: &str,
    node_at_position: &Arc<Node>,
    position: usize,
) -> Option<Vec<Arc<Symbol>>> {
    let (element, attributes) = opening_like_element_of(node_at_position, position)?;
    let tag_node = match &element.data {
        NodeData::JsxSelfClosingElement(d) => Arc::clone(&d.tag_name),
        NodeData::JsxOpeningElement(d) => Arc::clone(&d.tag_name),
        _ => return None,
    };
    let tag_name = jsx_tag_name_text(&tag_node)?;

    let t = if tag_name
        .chars()
        .next()
        .is_some_and(|c| c.is_ascii_lowercase() || tag_name.contains('-'))
    {
        // intrinsic：JSX.IntrinsicElements[tag] 的类型
        let elem_sym = checker.jsx_intrinsic_element_symbol(&tag_name)?;
        checker.get_type_of_symbol(&elem_sym)
    } else if let Some(t) = checker.jsx_element_attributes_contextual_type(&element) {
        // class 组件有构造器：构造签名首参（或 LibraryManagedAttributes 变换）
        t
    } else {
        // 无显式构造器（Go getJsxElementAttributesType 的 class 分支退化路径）：
        // JSX.ElementAttributesProperty 声明的容器名（React 为 "props"）在类
        // 实例类型上的属性即 attributes 类型；容器无属性时用实例类型本身；
        // 容器不存在时 attributes 为 any（None → 调用方继续后续路径）
        let class_sym = checker.resolve_identifier(&tag_node)?;
        let class_decl = class_sym
            .declarations
            .iter()
            .find(|d| d.kind == SyntaxKind::ClassDeclaration)
            .cloned()?;
        let instance = checker.build_class_instance_type_with_base(&class_decl);

        match jsx_element_properties_name(checker) {
            None => return None,
            Some(props_name) if props_name.is_empty() => instance,
            Some(props_name) => checker
                .get_property_of_type(&instance, &props_name)
                .map(|p| checker.get_type_of_symbol(&p))
                .unwrap_or_else(|| Arc::clone(&instance)),
        }
    };
    let props = checker.get_apparent_properties(&t);
    Some(filter_jsx_attributes(props, &attributes, file_text, position))
}

/// Go tryGetContainingJsxElement 的节点近似：从位置节点上行找属性语境
/// （JsxAttribute/JsxSpreadAttribute 挂在 JsxAttributes 上；JsxExpression 仅当
/// 是属性初始化器），返回宿主 opening/self-closing 元素与其属性表
fn opening_like_element_of(
    node_at_position: &Arc<Node>,
    position: usize,
) -> Option<(Arc<Node>, Arc<Node>)> {
    let mut current = Some(Arc::clone(node_at_position));
    while let Some(n) = current {
        if n.pos() > position {
            current = n.parent();
            continue;
        }
        let attrs = match n.kind {
            SyntaxKind::JsxAttribute | SyntaxKind::JsxSpreadAttribute => n.parent(),
            // 仅属性初始化器里的 JsxExpression 算属性语境；children 位置
            // （父为 JsxElement/JsxFragment）不算
            SyntaxKind::JsxExpression => n
                .parent()
                .clone()
                .filter(|p| p.kind == SyntaxKind::JsxAttribute)
                .and_then(|attr| attr.parent()),
            SyntaxKind::JsxSelfClosingElement | SyntaxKind::JsxOpeningElement => {
                let attrs = match &n.data {
                    NodeData::JsxSelfClosingElement(d) => Arc::clone(&d.attributes),
                    NodeData::JsxOpeningElement(d) => Arc::clone(&d.attributes),
                    _ => return None,
                };
                return Some((Arc::clone(&n), attrs));
            }
            SyntaxKind::SourceFile => return None,
            _ => {
                current = n.parent();
                continue;
            }
        };
        let element = attrs
            .as_ref()
            .and_then(|a| {
                (a.kind == SyntaxKind::JsxAttributes).then(|| a.parent())
            })
            .flatten();
        return element.map(|e| (e, attrs.unwrap()));
    }
    None
}

/// Go filterJsxAttributes：已写的属性名不再给出（正在编辑中的属性除外；
/// spread 属性只标记 sortText 不参与过滤）
fn filter_jsx_attributes(
    symbols: Vec<Arc<Symbol>>,
    attributes: &Arc<Node>,
    text: &str,
    position: usize,
) -> Vec<Arc<Symbol>> {
    let properties = match &attributes.data {
        NodeData::JsxAttributes(d) => d.properties.iter().cloned().collect::<Vec<_>>(),
        _ => return symbols,
    };
    let existing: HashSet<String> = properties
        .iter()
        .filter(|a| !is_editing_jsx_attribute(a, text, position))
        .filter_map(|a| match &a.data {
            NodeData::JsxAttribute(d) => Some(d.name.text().to_string()),
            _ => None,
        })
        .collect();
    symbols
        .into_iter()
        .filter(|s| !existing.contains(&s.name))
        .collect()
}

/// Go isCurrentlyEditingNode 的 JSX 适配：本仓 JSX 属性节点 span 会吞掉
/// 尾随空白（end 撑到下一个 token 起点），按去掉尾随空白的实际内容端判定
fn is_editing_jsx_attribute(attr: &Arc<Node>, text: &str, position: usize) -> bool {
    let start = skip_trivia(text, attr.pos());
    let content_end = text[..attr.end().min(text.len())]
        .trim_end()
        .len()
        .max(start);
    start <= position && position <= content_end
}

/// Go getJsxElementPropertiesName：JSX.ElementAttributesProperty 的单属性名
/// （React.d.ts 为 "props"）。容器不存在 → None（非 intrinsic 元素 attributes
/// 为 any）；容器无属性 → Some("")（attributes 为类实例类型本身）
fn jsx_element_properties_name(checker: &mut Checker) -> Option<String> {
    jsx_namespace_container_member_name(checker, "ElementAttributesProperty")
}

/// Go getNameFromJsxElementAttributesContainer：JSX 命名空间内容器
/// （ElementAttributesProperty / ElementChildrenAttribute）的单属性名；
/// 容器不存在 → None；容器无属性 → Some("")
pub(super) fn jsx_namespace_container_member_name(
    checker: &mut Checker,
    container_name: &str,
) -> Option<String> {
    // JSX 命名空间可来自文件内 declare namespace（locals/members）或全局 lib
    let jsx_sym = (|| {
        let symbol_map = checker.program.symbol_map();
        let file = checker.display_enclosing_file.clone()?;
        if let Some(locals) = symbol_map.locals.get(&file.node.id())
            && let Some(sym) = locals.get("JSX")
        {
            return Some(Arc::clone(sym));
        }
        if let Some(sym) = symbol_map.symbol_of(&file.node)
            && let Some(sym) = sym.members.get("JSX")
        {
            return Some(Arc::clone(sym));
        }
        None
    })()
    .or_else(|| checker.globals.get("JSX").cloned())?;
    let container = find_member_on_symbol(checker, &jsx_sym, container_name)?;
    // 容器是接口声明：成员由 checker 按声明构建（locals 在声明节点上）
    let resolved = checker.resolve_interface_type_ex(&container, None);
    let container_type = if checker.is_error_type(&resolved) {
        checker.get_type_of_symbol(&container)
    } else {
        resolved
    };
    let props = checker.get_apparent_properties(&container_type);
    match props.len() {
        0 => Some(String::new()),
        1 => Some(props[0].name.clone()),
        // 多于一个属性是错误（Go 报 The_global_type_JSX_0_may_not_have_more_than_one_property）
        _ => None,
    }
}

fn find_member_on_symbol(
    checker: &mut Checker,
    sym: &Arc<Symbol>,
    member: &str,
) -> Option<Arc<Symbol>> {
    for d in &sym.declarations {
        if let Some(locals) = checker.program.symbol_map().locals.get(&d.id())
            && let Some(m) = locals.get(member)
        {
            return Some(Arc::clone(m));
        }
    }
    sym.members.get(member).cloned()
}
