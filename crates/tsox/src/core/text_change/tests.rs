use super::*;

#[test]
fn apply_single_edit() {
    let text = "hello world";
    let edit = TextChange::new(TextRange::new(0, 5), "HELLO");
    assert_eq!(edit.apply_to(text), "HELLO world");
}

#[test]
fn apply_bulk_edits_works() {
    let text = "abcdef";
    let edits = vec![
        TextChange::new(TextRange::new(0, 1), "A"),
        TextChange::new(TextRange::new(2, 3), "C"),
        TextChange::new(TextRange::new(5, 6), "F"),
    ];

    assert_eq!(apply_bulk_edits(text, &edits), "AbCdeF");
}
