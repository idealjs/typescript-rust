use tsox_lsp::fourslash::{self, Session};

#[ignore = "unimplemented: fourslash.FormatDocument"]
#[test]
fn format_no_space_before_close_brace1() {
    let content = r#"new Foo(1, /* comment */    );"#;
    let mut s = Session::new(content);
    fourslash::unsupported("FormatDocument"); // f.FormatDocument(t, "")
    fourslash::verify_current_file_content(&mut s, r#"new Foo(1, /* comment */);"#);
}
