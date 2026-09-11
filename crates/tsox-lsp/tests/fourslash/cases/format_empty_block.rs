use tsox_lsp::fourslash::{self, Session};


#[test]
fn format_empty_block() {
    let content = r#"{}"#;
    let mut s = Session::new_for_test("formatEmptyBlock", content);
    // TODO: f.GoToEOF(t)
    fourslash::insert(&mut s, "\n");
    // TODO: f.GoToBOF(t)
    fourslash::verify_current_line_content(&mut s, r#"{ }"#);
}
