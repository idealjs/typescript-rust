use std::sync::Arc;

use tsox_checker::checker::Checker;
use tsox_frontend::ast::Node;

pub(super) fn primitive_interface_of(t: &tsox_checker::checker::types::Type) -> Option<&'static str> {
    use tsox_checker::checker::types::TypeFlags;
    if t.flags.intersects(TypeFlags::String | TypeFlags::StringLiteral) {
        Some("String")
    } else if t.flags.intersects(TypeFlags::Number | TypeFlags::NumberLiteral) {
        Some("Number")
    } else if t.flags.intersects(TypeFlags::Boolean | TypeFlags::BooleanLiteral) {
        Some("Boolean")
    } else if t.flags.intersects(TypeFlags::ESSymbol | TypeFlags::UniqueESSymbol) {
        Some("Symbol")
    } else {
        None
    }
}

pub(super) fn source_text_of(checker: &Checker, node: &Arc<Node>) -> Option<(String, Arc<Node>)> {
    let sf = checker.get_source_file_of_node(node)?;
    let mut cur = Arc::clone(node);
    while cur.kind != tsox_frontend::ast::SyntaxKind::SourceFile {
        cur = cur.parent.clone()?;
    }
    Some((sf.text.clone(), cur))
}

pub(super) fn deepest_node_ending_at(root: &Arc<Node>, end: usize) -> Option<Arc<Node>> {
    use tsox_frontend::ast::node_data_generated::for_each_child;
    // 沿包含 end 的孩子下降（尾随 '.' 会被外层语句吞掉），途经 end==dot 的
    // 最深节点即接收者表达式
    let mut best: Option<Arc<Node>> = None;
    if root.pos() < end && root.end() == end {
        best = Some(Arc::clone(root));
    }
    let mut children = Vec::new();
    for_each_child(root, |c| {
        children.push(Arc::clone(c));
        false
    });
    for c in children {
        if c.pos() < end && c.end() == end {
            // 最浅命中即接收者（继续下降会落进尾部实参列表/子表达式）
            return Some(c);
        }
        if c.pos() <= end && c.end() > end {
            if let Some(deeper) = deepest_node_ending_at(&c, end) {
                return Some(deeper);
            }
        }
    }
    best
}

pub(super) fn base_identifier(node: &Arc<Node>) -> Option<Arc<Node>> {
    use tsox_frontend::ast::NodeData;
    match &node.data {
        NodeData::Identifier(_) => Some(Arc::clone(node)),
        NodeData::QualifiedName(d) => base_identifier(&d.left),
        NodeData::PropertyAccessExpression(d) => base_identifier(&d.expression),
        _ => None,
    }
}
