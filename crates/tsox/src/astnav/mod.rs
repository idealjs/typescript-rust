use crate::ast::{Node, SourceFile, SyntaxKind, for_each_child, is_token_kind};
use std::sync::Arc;

fn collect_children(node: &Node) -> Vec<Arc<Node>> {
    let mut children = Vec::new();
    for_each_child(node, |child| {
        children.push(Arc::clone(child));
        false
    });
    children
}

pub fn get_token_at_position(source_file: &Arc<Node>, position: usize) -> Option<Arc<Node>> {
    let mut current = Arc::clone(source_file);
    loop {
        let children = collect_children(&current);
        let next = children
            .into_iter()
            .find(|child| child.pos() <= position && position < child.end());
        match next {
            Some(child) => current = child,
            None => return Some(current),
        }
    }
}

pub fn find_preceding_token(source_file: &Arc<Node>, position: usize) -> Option<Arc<Node>> {
    find_last_token_ending_at_or_before(source_file, position)
}

fn find_last_token_ending_at_or_before(node: &Arc<Node>, position: usize) -> Option<Arc<Node>> {
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

fn find_first_token_starting_after(node: &Arc<Node>, position: usize) -> Option<Arc<Node>> {
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

pub fn get_touching_property_name(source_file: &Arc<Node>, position: usize) -> Option<Arc<Node>> {
    get_token_at_position(source_file, position)
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
    get_token_at_position(source_file, position)
}

pub fn get_touching_token(source_file: &Arc<Node>, position: usize) -> Option<Arc<Node>> {
    get_token_at_position(source_file, position)
}

#[cfg(test)]
mod tests;
