use tsox_lsp::fourslash::{self, Session};


#[test]
fn format_no_space_before_close_brace5() {
    let content = r#"new Foo(1, 
    /* comment */    );"#;
    let mut s = Session::new_for_test("formatNoSpaceBeforeCloseBrace5", content);
    // TODO: f.FormatDocument(t, "")
    fourslash::verify_current_file_content(&mut s, r#"new Foo(1,
    /* comment */);"#);
}
