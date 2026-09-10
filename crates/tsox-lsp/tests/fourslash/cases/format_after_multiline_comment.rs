use tsox_lsp::fourslash::{self, Session};


#[ignore = "unimplemented: fourslash.FormatDocument"]
#[test]
fn format_after_multiline_comment() {
    let content = r#"/*foo
*/"123123";"#;
    let mut s = Session::new_for_test("formatAfterMultilineComment", content);
    fourslash::unsupported("FormatDocument"); // f.FormatDocument(t, "")
    fourslash::verify_current_file_content(&mut s, r#"/*foo
*/"123123";"#);
}
