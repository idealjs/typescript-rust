pub(crate) use crate::ast::{
    Node, SourceFile, SyntaxKind, for_each_child, is_keyword_kind, is_property_name_literal,
    is_token_kind,
};
pub(crate) use std::sync::Arc;

pub(crate) fn collect_children(node: &Node) -> Vec<Arc<Node>> {
    let mut children = Vec::new();
    for_each_child(node, |child| {
        children.push(Arc::clone(child));
        false
    });
    children
}

pub fn get_token_at_position(source_file: &Arc<Node>, position: usize) -> Option<Arc<Node>> {
    get_token_at_position_inner(source_file, position, None, None)
}

/// 对齐 Go astnav.GetTouchingPropertyName：谓词（属性名字面量/关键字/#私有名）
/// 同时作为 includePrecedingTokenAtEndPosition，位置在 token 结尾时回退前 token
pub fn get_touching_property_name(source_file: &Arc<Node>, position: usize) -> Option<Arc<Node>> {
    let pred = |n: &Arc<Node>| {
        is_property_name_literal(n)
            || is_keyword_kind(n.kind)
            || n.kind == SyntaxKind::PrivateIdentifier
    };
    get_token_at_position_inner(source_file, position, Some(&pred), None)
}

/// 同上，但子节点遍历附带各节点的 JSDoc（对齐 Go VisitEachChildAndJSDoc），
/// 使 JSDoc 注解内的类型引用可被定位
pub fn get_touching_property_name_with_jsdoc(
    file: &Arc<SourceFile>,
    position: usize,
) -> Option<Arc<Node>> {
    let pred = |n: &Arc<Node>| {
        is_property_name_literal(n)
            || is_keyword_kind(n.kind)
            || n.kind == SyntaxKind::PrivateIdentifier
    };
    let jsdoc_of = |n: &Arc<Node>| file.resolve_jsdoc(n);
    get_token_at_position_inner(&file.node, position, Some(&pred), Some(&jsdoc_of))
}

fn get_token_at_position_inner(
    source_file: &Arc<Node>,
    position: usize,
    predicate: Option<&dyn Fn(&Arc<Node>) -> bool>,
    jsdoc_of: Option<&dyn Fn(&Arc<Node>) -> Vec<Arc<Node>>>,
) -> Option<Arc<Node>> {
    let mut current = Arc::clone(source_file);
    loop {
        let mut prev_subtree: Option<Arc<Node>> = None;
        let mut next: Option<Arc<Node>> = None;
        let mut children = collect_children(&current);
        if let Some(jsdoc_of) = jsdoc_of {
            // 对齐 Go VisitEachChildAndJSDoc：JSDoc 挂在子节点 leading 区，
            // 并入候选后注释内的位置可命中 JSDoc 子树
            let jsdocs: Vec<Vec<Arc<Node>>> = children
                .iter()
                .map(|child| jsdoc_of(child))
                .collect();
            children.extend(jsdocs.into_iter().flatten());
        }
        for child in children {
            if next.is_some() {
                break;
            }
            if predicate.is_some() && child.kind != SyntaxKind::EndOfFile && child.end() == position
            {
                prev_subtree = Some(Arc::clone(&child));
            }
            let contains =
                child.end() > position || child.end() == position && child.kind == SyntaxKind::EndOfFile;
            if !contains || child.pos() > position {
                continue;
            }
            next = Some(child);
        }
        if let Some(prev) = prev_subtree {
            if let Some(token) = find_last_token_ending_at_or_before(&prev, position) {
                if token.end() == position && predicate.is_some_and(|p| p(&token)) {
                    return Some(token);
                }
            }
        }
        match next {
            Some(child) => current = child,
            None => break,
        }
    }
    // 空隙回退（Go scanner 分支）：无子节点包含位置（注释/空白/子节点区间外的
    // 前置列表如装饰器），扫描覆盖位置的 token，其次谓词命中的位置收尾 token
    if !is_token_kind(current.kind) {
        let mut covering: Option<Arc<Node>> = None;
        let mut hit: Option<Arc<Node>> = None;
        let jsdocs = jsdoc_of.map(|f| f(&current)).unwrap_or_default();
        let mut visit_token = |tok: &Arc<Node>| {
            if tok.pos() <= position && position < tok.end() && tok.kind != SyntaxKind::EndOfFile
            {
                if covering.as_ref().is_none_or(|c| c.pos() <= tok.pos()) {
                    covering = Some(Arc::clone(tok));
                }
            }
            if tok.end() == position && predicate.is_some_and(|p| p(tok)) {
                hit = Some(Arc::clone(tok));
            }
        };
        collect_tokens_in_order(&current, &mut |tok| {
            visit_token(tok);
            false
        });
        for jsdoc in &jsdocs {
            collect_tokens_in_order(jsdoc, &mut |tok| {
                visit_token(tok);
                false
            });
        }
        if let Some(tok) = covering {
            return Some(tok);
        }
        if let Some(tok) = hit {
            return Some(tok);
        }
    }
    Some(current)
}

fn collect_tokens_in_order(node: &Arc<Node>, visit: &mut impl FnMut(&Arc<Node>) -> bool) {
    if is_token_kind(node.kind) {
        visit(node);
        return;
    }
    let children = collect_children(node);
    for child in children {
        collect_tokens_in_order(&child, visit);
    }
}

pub fn find_preceding_token(source_file: &Arc<Node>, position: usize) -> Option<Arc<Node>> {
    find_last_token_ending_at_or_before(source_file, position)
}

pub(crate) fn find_last_token_ending_at_or_before(
    node: &Arc<Node>,
    position: usize,
) -> Option<Arc<Node>> {
    if node.pos() >= position {
        return None;
    }

    if is_token_kind(node.kind) {
        return if node.end() <= position {
            Some(Arc::clone(node))
        } else {
            None
        };
    }

    let children = collect_children(node);
    for child in children.iter().rev() {
        if let Some(token) = find_last_token_ending_at_or_before(child, position) {
            return Some(token);
        }
    }
    None
}

pub fn find_next_token(source_file: &Arc<Node>, position: usize) -> Option<Arc<Node>> {
    find_first_token_starting_after(source_file, position)
}

pub(crate) fn find_first_token_starting_after(
    node: &Arc<Node>,
    position: usize,
) -> Option<Arc<Node>> {
    if node.end() <= position {
        return None;
    }

    if is_token_kind(node.kind) {
        return if node.pos() > position {
            Some(Arc::clone(node))
        } else {
            None
        };
    }

    let children = collect_children(node);
    for child in children.iter() {
        if let Some(token) = find_first_token_starting_after(child, position) {
            return Some(token);
        }
    }
    None
}

pub fn find_child_of_kind(containing_node: &Arc<Node>, kind: SyntaxKind) -> Option<Arc<Node>> {
    let mut result = None;
    for_each_child(containing_node, |child| {
        if child.kind == kind {
            result = Some(Arc::clone(child));
            return true;
        }
        false
    });
    result
}

pub fn get_start_of_node(
    node: &Arc<Node>,
    _source_file: &SourceFile,
    _include_jsdoc: bool,
) -> usize {
    node.pos()
}

pub fn get_end_of_node(node: &Arc<Node>) -> usize {
    node.end()
}

pub fn is_missing_node(node: &Node) -> bool {
    node.pos() == node.end() && (node.pos() as i32) >= 0 && node.kind != SyntaxKind::EndOfFile
}

pub fn get_position_of_line_and_character(
    source_file: &SourceFile,
    line: usize,
    character: usize,
) -> usize {
    let line_map = &source_file.line_map;
    if line >= line_map.line_starts.len() {
        return source_file.text.len();
    }
    let line_start = line_map.line_starts[line] as usize;
    let text = &source_file.text;
    let bytes = text.as_bytes();
    let text_len = bytes.len();
    let mut col_utf16 = 0usize;
    let mut pos = line_start;
    while pos < text_len && col_utf16 < character {
        let b = bytes[pos];
        if b < 0x80 {
            pos += 1;
            col_utf16 += 1;
        } else {
            let remaining = &text[pos..];
            match remaining.chars().next() {
                Some(ch) => {
                    pos += ch.len_utf8();
                    col_utf16 += ch.len_utf16();
                }
                None => break,
            }
        }
    }
    pos
}

pub fn get_line_and_character_of_position(
    source_file: &SourceFile,
    position: usize,
) -> (usize, usize) {
    let line_map = &source_file.line_map;
    let line = line_map.line_at(position);
    let character = line_map.utf16_column_at(&source_file.text, position);
    (line, character)
}

pub fn get_touching_property_name_astnav(
    source_file: &Arc<Node>,
    position: usize,
) -> Option<Arc<Node>> {
    get_touching_property_name(source_file, position)
}

pub fn get_touching_token(source_file: &Arc<Node>, position: usize) -> Option<Arc<Node>> {
    get_token_at_position(source_file, position)
}

#[cfg(test)]
pub(crate) mod tests;
