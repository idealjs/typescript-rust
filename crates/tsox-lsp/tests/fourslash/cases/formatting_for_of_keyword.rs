use tsox_lsp::fourslash::{self, Session};


#[test]
fn formatting_for_of_keyword() {
    let content = r#"/**/for ([]of[]) { }"#;
    let mut s = Session::new_for_test("formattingForOfKeyword", content);
    fourslash::format_document(&mut s, "");
    fourslash::go_to_marker(&mut s, "");
    fourslash::verify_current_line_content(&mut s, r#"for ([] of []) { }"#);
}
