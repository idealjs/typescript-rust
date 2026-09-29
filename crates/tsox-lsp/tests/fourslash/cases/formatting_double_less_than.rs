use tsox_lsp::fourslash::{self, Session};


#[test]
fn formatting_double_less_than() {
    let content = r#"/*1*/if (<number>foo < <number>bar) {}"#;
    let mut s = Session::new_for_test("formattingDoubleLessThan", content);
    fourslash::format_document(&mut s, "");
    fourslash::go_to_marker(&mut s, "1");
    fourslash::verify_current_line_content(&mut s, r#"if (<number>foo < <number>bar) { }"#);
}
