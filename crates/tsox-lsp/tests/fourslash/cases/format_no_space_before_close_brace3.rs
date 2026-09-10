use tsox_lsp::fourslash::{self, Session};


#[ignore = "unimplemented: fourslash.FormatDocument"]
#[test]
fn format_no_space_before_close_brace3() {
    let content = r#"foo( 
 1, /* comment */    );"#;
    let mut s = Session::new_for_test("formatNoSpaceBeforeCloseBrace3", content);
    fourslash::unsupported("FormatDocument"); // f.FormatDocument(t, "")
    fourslash::verify_current_file_content(&mut s, r#"foo(
    1, /* comment */);"#);
}
