use tsox_lsp::fourslash::{self, Session};


#[test]
fn format_empty_block() {
    let content = r#"{}"#;
    let mut s = Session::new_for_test("formatEmptyBlock", content);
    fourslash::go_to_eof(&mut s, );
    fourslash::insert(&mut s, "\n");
    fourslash::go_to_bof(&mut s, );
    fourslash::verify_current_line_content(&mut s, r#"{ }"#);
}
