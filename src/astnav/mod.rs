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
mod tests {
    use super::*;
    use crate::parser::Parser;

    #[test]
    fn get_token_at_position_jsdoc_type_assertion() {
        let file_text = "function foo(x) {\n    const s = /**@type {string}*/(x)\n}";

        let position: usize = 52;
        let file = Parser::parse_source_file_text("/test.js", file_text.to_string());
        let token = get_touching_property_name(&file.node, position);
        assert!(token.is_some(), "Expected to get a token");
        let token = token.unwrap();
        assert!(
            token.kind == SyntaxKind::Identifier
                || token.kind == SyntaxKind::ParenthesizedExpression,
            "Expected identifier or parenthesized expression, got {:?}",
            token.kind
        );
    }

    #[test]
    fn get_token_at_position_jsdoc_type_assertion_with_comment() {
        let file_text = "function foo(x) {\n    const s = /**@type {string}*/(x)  // comment\n}";
        let x_pos: usize = 52;
        let file = Parser::parse_source_file_text("/test.js", file_text.to_string());
        let token = get_touching_property_name(&file.node, x_pos);
        assert!(token.is_some(), "Expected to get a token");
    }

    #[test]
    fn get_token_at_position_pointer_equality() {
        let file_text = "\n\t\t\tfunction foo() {\n\t\t\t\treturn 0;\n\t\t\t}";
        let file = Parser::parse_source_file_text("/file.ts", file_text.to_string());
        let t1 = get_token_at_position(&file.node, 0);
        let t2 = get_token_at_position(&file.node, 0);
        assert!(t1.is_some() && t2.is_some());
        assert!(
            Arc::ptr_eq(t1.as_ref().unwrap(), t2.as_ref().unwrap()),
            "Expected pointer-equal nodes for repeated calls"
        );
    }

    #[test]
    fn get_token_at_position_baseline() {
        let file_text = "a.b";
        let file = Parser::parse_source_file_text("/f.ts", file_text.to_string());

        let pos: usize = 2;
        let token = get_token_at_position(&file.node, pos).expect("a token at position");
        assert!(
            token.pos() <= pos && pos < token.end(),
            "returned node must contain the position"
        );
        assert_eq!(token.kind, SyntaxKind::Identifier);
    }

    #[test]
    fn get_touching_property_name_baseline() {
        let file_text = "foo.bar";
        let file = Parser::parse_source_file_text("/f.ts", file_text.to_string());

        let pos: usize = 4;
        let token = get_touching_property_name(&file.node, pos).expect("a token at position");
        assert!(
            token.pos() <= pos && pos < token.end(),
            "returned node must contain the position"
        );
        assert_eq!(token.kind, SyntaxKind::Identifier);
    }

    #[test]
    fn find_preceding_token_baseline() {
        let file_text = "a - b";
        let file = Parser::parse_source_file_text("/f.ts", file_text.to_string());

        let token = find_preceding_token(&file.node, 4).expect("a preceding token");
        assert_eq!(token.kind, SyntaxKind::MinusToken, "Expected MinusToken");
    }

    #[test]
    fn find_next_token_baseline() {
        let file_text = "a + b";
        let file = Parser::parse_source_file_text("/f.ts", file_text.to_string());

        let token = find_next_token(&file.node, 0).expect("a following token");
        assert_eq!(token.kind, SyntaxKind::PlusToken, "Expected PlusToken");
    }

    #[test]
    fn find_preceding_token_after_comma_in_parameter_list() {
        let file_content = "takesCb((n, s, ))";
        let position: usize = 15;
        let file = Parser::parse_source_file_text("/file.ts", file_content.to_string());
        let token = find_preceding_token(&file.node, position);
        assert!(token.is_some(), "Expected a preceding token");
        assert_eq!(
            token.unwrap().kind,
            SyntaxKind::CommaToken,
            "Expected CommaToken"
        );
    }

    #[test]
    fn find_preceding_token_after_dot_in_jsdoc() {

        let file_content = "a + b";
        let file = Parser::parse_source_file_text("/file.ts", file_content.to_string());

        let token = find_preceding_token(&file.node, 4);
        assert!(token.is_some(), "Expected a preceding token");
        assert_eq!(
            token.unwrap().kind,
            SyntaxKind::PlusToken,
            "Expected PlusToken"
        );
    }
}
