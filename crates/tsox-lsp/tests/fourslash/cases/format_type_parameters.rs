use tsox_lsp::fourslash::{self, Session};


#[ignore = "unimplemented: fourslash.FormatDocument"]
#[test]
fn format_type_parameters() {
    let content = r#"/**/type Bar<T extends any[]= any[]> = T"#;
    let mut s = Session::new_for_test("formatTypeParameters", content);
    fourslash::unsupported("FormatDocument"); // f.FormatDocument(t, "")
    fourslash::go_to_marker(&mut s, "");
    fourslash::verify_current_line_content(&mut s, r#"type Bar<T extends any[] = any[]> = T"#);
}
