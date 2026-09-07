use tsox_lsp::fourslash::{self, Session};

#[ignore = "unimplemented: fourslash.FormatDocument"]
#[test]
fn formatting_object_literal() {
    let content = r#"var clear = {
"a": 1/**/
}"#;
    let mut s = Session::new(content);
    fourslash::unsupported("FormatDocument"); // f.FormatDocument(t, "")
    fourslash::go_to_marker(&mut s, "");
    fourslash::verify_current_line_content(&mut s, r#"    "a": 1"#);
}
