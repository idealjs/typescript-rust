use super::super::SyntaxKind;
use super::super::node_data_generated::{IdentifierData, NodeData};
use super::line_map::{LineMap, utf16_len};
use super::node::Node;

#[test]
fn identifier_node() {
    let node = Node::new(
        SyntaxKind::Identifier,
        NodeData::Identifier(IdentifierData {
            text: "foo".to_string(),
        }),
    );
    assert_eq!(node.kind, SyntaxKind::Identifier);
    assert_eq!(node.text(), "foo");
}

#[test]
fn node_ids_are_unique() {
    let n1 = Node::new(SyntaxKind::Unknown, NodeData::Token);
    let n2 = Node::new(SyntaxKind::Unknown, NodeData::Token);
    assert_ne!(n1.id(), n2.id());
}

#[test]
fn line_map_basic() {
    let lm = LineMap::from_text("abc\ndef\nghi");
    assert_eq!(lm.line_starts, vec![0, 4, 8]);
    assert_eq!(lm.line_at(0), 0);
    assert_eq!(lm.line_at(3), 0);
    assert_eq!(lm.line_at(4), 1);
    assert_eq!(lm.line_at(7), 1);
    assert_eq!(lm.line_at(8), 2);
}

#[test]
fn line_map_crlf() {
    let lm = LineMap::from_text("abc\r\ndef\r\nghi");
    assert_eq!(lm.line_starts, vec![0, 5, 10]);
    assert_eq!(lm.line_at(0), 0);
    assert_eq!(lm.line_at(3), 0);
    assert_eq!(lm.line_at(5), 1);
    assert_eq!(lm.line_at(10), 2);
}

#[test]
fn line_map_cr_only() {
    let lm = LineMap::from_text("abc\rdef");
    assert_eq!(lm.line_starts, vec![0, 4]);
    assert_eq!(lm.line_at(0), 0);
    assert_eq!(lm.line_at(4), 1);
}

#[test]
fn line_map_unicode_line_separators() {
    let lm = LineMap::from_text("ab\u{2028}cd\u{2029}ef");
    assert_eq!(lm.line_starts.len(), 3);
    assert_eq!(lm.line_at(0), 0);

    assert_eq!(lm.line_at(5), 1);

    assert_eq!(lm.line_at(10), 2);
}

#[test]
fn line_map_utf16_column_ascii() {
    let text = "abc\ndef";
    let lm = LineMap::from_text(text);

    assert_eq!(lm.utf16_column_at(text, 5), 1);

    assert_eq!(lm.utf16_column_at(text, 6), 2);
}

#[test]
fn line_map_utf16_column_non_ascii() {
    let text = "café\ndef";
    let lm = LineMap::from_text(text);

    assert_eq!(lm.utf16_column_at(text, 3), 3);

    assert_eq!(lm.utf16_column_at(text, 5), 4);
}

#[test]
fn line_map_utf16_column_emoji() {
    let text = "x🦀y";
    let lm = LineMap::from_text(text);

    assert_eq!(lm.utf16_column_at(text, 5), 3);
}

#[test]
fn utf16_len_basic() {
    assert_eq!(utf16_len("abc"), 3);
    assert_eq!(utf16_len("café"), 4);
    assert_eq!(utf16_len("🦀"), 2);
    assert_eq!(utf16_len("x🦀y"), 4);
}
