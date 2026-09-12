//! Go completions.go getJsxClosingTagCompletion：JSX 变体文件补全管线最前
//! 短路，位置处于闭合标签（沿标签名组成节点上行可达 JsxClosingElement）时，
//! 返回关联开标签名；闭合标签缺 `>` 时标签附 ">"

use std::sync::Arc;

use tsox_frontend::ast::Node;
use tsox_frontend::ast::NodeData;
use tsox_frontend::ast::SyntaxKind;

use super::completions_context;
use super::types::CompletionItem;

/// Some(item) 表示命中闭合标签语境，单项列表短路返回
pub(super) fn jsx_closing_tag_completion(
    text: &str,
    node_at_position: &Arc<Node>,
    position: usize,
) -> Option<CompletionItem> {
    let closing = find_jsx_closing_element(text, node_at_position, position)?;
    let element = closing.parent()?;
    let opening = match &element.data {
        NodeData::JsxElement(d) => Arc::clone(&d.opening_element),
        _ => return None,
    };
    let tag_name = match &opening.data {
        NodeData::JsxOpeningElement(d) => jsx_tag_name_text(&d.tag_name)?,
        _ => return None,
    };
    let has_closing_angle = closing_angle_end(text, &closing).is_some();
    let label = if has_closing_angle {
        tag_name
    } else {
        format!("{tag_name}>")
    };
    Some(CompletionItem {
        label,
        ..Default::default()
    })
}

/// Go FindAncestorOrQuit：从位置节点上行，遇 JsxClosingElement 命中；
/// 标签名组成节点（标识符/属性访问/JSX 命名空间名）续行；其它节点终止。
/// `>` token 不在 AST 上：上行落在 JsxElement 时按闭合标签的有效区间
/// （含 `>`）兜底判定
fn find_jsx_closing_element(
    text: &str,
    node_at_position: &Arc<Node>,
    position: usize,
) -> Option<Arc<Node>> {
    let mut current = Some(Arc::clone(node_at_position));
    while let Some(n) = current {
        match n.kind {
            SyntaxKind::JsxClosingElement => return Some(Arc::clone(&n)),
            SyntaxKind::JsxElement => {
                let closing = match &n.data {
                    NodeData::JsxElement(d) => Arc::clone(&d.closing_element),
                    _ => return None,
                };
                let effective_end =
                    closing_angle_end(text, &closing).unwrap_or_else(|| closing.end());
                if closing.pos() <= position && position <= effective_end {
                    return Some(closing);
                }
                return None;
            }
            SyntaxKind::Identifier
            | SyntaxKind::PropertyAccessExpression
            | SyntaxKind::JsxNamespacedName => {}
            _ => return None,
        }
        current = n.parent();
    }
    None
}

/// 闭合标签区间内 `>` 的结尾位置（token 不在 AST 上，扫描判定）
fn closing_angle_end(text: &str, closing: &Arc<Node>) -> Option<usize> {
    completions_context::scan_tokens(text, true, closing.pos(), closing.end())
        .iter()
        .filter(|t| t.kind == SyntaxKind::GreaterThanToken)
        .map(|t| t.end)
        .max()
}

/// 标签完整文本（node_text 对 PAE/命名空间名返回空串，按结构重建）
pub(super) fn jsx_tag_name_text(tag: &Arc<Node>) -> Option<String> {
    match &tag.data {
        NodeData::Identifier(d) => Some(d.text.clone()),
        NodeData::JsxNamespacedName(d) => Some(format!("{}:{}", d.namespace.text(), d.name.text())),
        NodeData::PropertyAccessExpression(d) => {
            Some(format!("{}.{}", jsx_tag_name_text(&d.expression)?, d.name.text()))
        }
        _ => None,
    }
}
