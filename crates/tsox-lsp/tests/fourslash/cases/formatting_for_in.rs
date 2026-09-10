use tsox_lsp::fourslash::{self, Session};


#[ignore = "unimplemented: fourslash.FormatDocument"]
#[test]
fn formatting_for_in() {
    let content = r#"/**/for (var i    in[]   )  {}"#;
    let mut s = Session::new_for_test("formattingForIn", content);
    fourslash::unsupported("FormatDocument"); // f.FormatDocument(t, "")
    fourslash::go_to_marker(&mut s, "");
    fourslash::verify_current_line_content(&mut s, r#"for (var i in []) { }"#);
}
