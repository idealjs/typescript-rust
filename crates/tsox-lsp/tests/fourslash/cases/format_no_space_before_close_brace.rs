use tsox_lsp::fourslash::{self, Session};


#[test]
fn format_no_space_before_close_brace() {
    let content = r#"foo(1, /* comment */    );"#;
    let mut s = Session::new_for_test("formatNoSpaceBeforeCloseBrace", content);
    fourslash::format_document(&mut s, "");
    fourslash::verify_current_file_content(&mut s, r#"foo(1, /* comment */);"#);
}
