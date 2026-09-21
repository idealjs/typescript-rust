use std::sync::Arc;

use tsox_frontend::ast::node_data_generated::NodeData;
use tsox_frontend::ast::{Node, SyntaxKind};

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub(crate) enum AccessKind {
    Read,
    Write,
    ReadWrite,
}

pub(crate) fn access_kind(node: &Arc<Node>) -> AccessKind {
    let Some(parent) = node.parent() else {
        return AccessKind::Read;
    };
    match parent.kind {
        SyntaxKind::ParenthesizedExpression => access_kind(&parent),
        SyntaxKind::PrefixUnaryExpression => {
            let NodeData::PrefixUnaryExpression(d) = &parent.data else {
                return AccessKind::Read;
            };
            match d.operator {
                SyntaxKind::PlusPlusToken | SyntaxKind::MinusMinusToken => AccessKind::ReadWrite,
                _ => AccessKind::Read,
            }
        }
        SyntaxKind::PostfixUnaryExpression => {
            let NodeData::PostfixUnaryExpression(d) = &parent.data else {
                return AccessKind::Read;
            };
            match d.operator {
                SyntaxKind::PlusPlusToken | SyntaxKind::MinusMinusToken => AccessKind::ReadWrite,
                _ => AccessKind::Read,
            }
        }
        SyntaxKind::BinaryExpression => {
            let NodeData::BinaryExpression(d) = &parent.data else {
                return AccessKind::Read;
            };
            if !Arc::ptr_eq(&d.left, node) {
                return AccessKind::Read;
            }
            let op = d.operator_token.kind;
            if crate::binder::helpers::is_assignment_operator(op) {
                if op == SyntaxKind::EqualsToken {
                    AccessKind::Write
                } else {
                    AccessKind::ReadWrite
                }
            } else {
                AccessKind::Read
            }
        }
        SyntaxKind::PropertyAccessExpression => {
            let NodeData::PropertyAccessExpression(d) = &parent.data else {
                return AccessKind::Read;
            };
            if !Arc::ptr_eq(&d.name, node) {
                return AccessKind::Read;
            }
            access_kind(&parent)
        }
        SyntaxKind::PropertyAssignment => {
            let NodeData::PropertyAssignment(d) = &parent.data else {
                return AccessKind::Read;
            };
            let parent_access = parent
                .parent()
                .as_ref()
                .map(access_kind)
                .unwrap_or(AccessKind::Read);
            if Arc::ptr_eq(&d.name, node) {
                reverse_access_kind(parent_access)
            } else {
                parent_access
            }
        }
        SyntaxKind::ShorthandPropertyAssignment => {
            let NodeData::ShorthandPropertyAssignment(d) = &parent.data else {
                return AccessKind::Read;
            };
            if d
                .object_assignment_initializer
                .as_ref()
                .is_some_and(|n| Arc::ptr_eq(n, node))
            {
                return AccessKind::Read;
            }
            let grandparent = parent.parent();
            grandparent
                .as_ref()
                .map(access_kind)
                .unwrap_or(AccessKind::Read)
        }
        SyntaxKind::ArrayLiteralExpression => access_kind(&parent),
        SyntaxKind::ForInStatement | SyntaxKind::ForOfStatement => {
            let NodeData::ForInOrOfStatement(d) = &parent.data else {
                return AccessKind::Read;
            };
            if Arc::ptr_eq(&d.initializer, node) {
                AccessKind::Write
            } else {
                AccessKind::Read
            }
        }
        _ => AccessKind::Read,
    }
}

pub(crate) fn reverse_access_kind(kind: AccessKind) -> AccessKind {
    match kind {
        AccessKind::Read => AccessKind::Write,
        AccessKind::Write => AccessKind::Read,
        AccessKind::ReadWrite => AccessKind::ReadWrite,
    }
}

pub(crate) fn is_write_only_access(node: &Arc<Node>) -> bool {
    access_kind(node) == AccessKind::Write
}
