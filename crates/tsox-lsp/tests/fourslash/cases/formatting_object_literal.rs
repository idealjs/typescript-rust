use tsox_lsp::fourslash::{self, Session};


#[test]
fn formatting_object_literal() {
    let content = r#"var clear = {
"a": 1/**/
}"#;
    let mut s = Session::new_for_test("formattingObjectLiteral", content);
    fourslash::format_document(&mut s, "");
    fourslash::go_to_marker(&mut s, "");
    fourslash::verify_current_line_content(&mut s, r#"    "a": 1"#);
}
