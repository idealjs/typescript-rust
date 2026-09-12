//! hover 解构赋值目标的位置判定（parser 重建丢失字段，需 span 推断）。

use std::sync::Arc;

use tsox_frontend::ast::{Node, SyntaxKind};

/// 对象字面量处于赋值表达式左侧（解构赋值目标）的位置判定（旧管线同款）
pub(crate) fn is_assignment_target_literal(obj: &Arc<Node>) -> bool {
    let target = match assignment_target_expr(obj) {
        Some(t) => t,
        None => return false,
    };
    let mut cur = obj.parent();
    while let Some(n) = cur {
        match n.kind {
            SyntaxKind::ObjectLiteralExpression | SyntaxKind::ParenthesizedExpression => {
                cur = n.parent()
            }
            SyntaxKind::BinaryExpression => {
                if let tsox_frontend::ast::NodeData::BinaryExpression(be) = &n.data {
                    return be.operator_token.kind == SyntaxKind::EqualsToken
                        && be.left.pos() <= obj.pos()
                        && obj.end() <= be.left.end();
                }
                return false;
            }
            _ => return false,
        }
    }
    false
}

fn assignment_target_expr(obj: &Arc<Node>) -> Option<Arc<Node>> {
    let mut cur = obj.parent();
    while let Some(n) = cur.clone() {
        match n.kind {
            SyntaxKind::ObjectLiteralExpression | SyntaxKind::ParenthesizedExpression => {
                cur = n.parent()
            }
            SyntaxKind::BinaryExpression => {
                if let tsox_frontend::ast::NodeData::BinaryExpression(be) = &n.data {
                    if be.operator_token.kind == SyntaxKind::EqualsToken {
                        return Some(Arc::clone(&be.left));
                    }
                }
                return None;
            }
            _ => return None,
        }
    }
    None
}
