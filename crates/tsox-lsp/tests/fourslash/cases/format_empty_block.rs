use tsox_lsp::fourslash::{self, Session};


#[ignore = "unimplemented: fourslash.GoToBOF"]
#[test]
fn format_empty_block() {
    let content = r#"{}"#;
    let mut s = Session::new_for_test("formatEmptyBlock", content);
    fourslash::unsupported("GoToEOF"); // f.GoToEOF(t)
    fourslash::insert(&mut s, "\n");
    fourslash::unsupported("GoToBOF"); // f.GoToBOF(t)
    fourslash::verify_current_line_content(&mut s, r#"{ }"#);
}
