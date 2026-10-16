use tsox_lsp::fourslash::{self, Session};


#[test]
fn format_no_space_before_close_brace4() {
    let content = r#"new Foo(1
, /* comment */    );"#;
    let mut s = Session::new_for_test("formatNoSpaceBeforeCloseBrace4", content);
    fourslash::format_document(&mut s, "");
    fourslash::verify_current_file_content(&mut s, r#"new Foo(1
    , /* comment */);"#);
}
