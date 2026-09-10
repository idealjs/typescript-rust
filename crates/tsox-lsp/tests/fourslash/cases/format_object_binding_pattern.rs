use tsox_lsp::fourslash::{self, Session};


#[ignore = "unimplemented: fourslash.FormatDocument"]
#[test]
fn format_object_binding_pattern() {
    let content = r#"const {
x,
y,
} = 0;"#;
    let mut s = Session::new_for_test("formatObjectBindingPattern", content);
    fourslash::unsupported("FormatDocument"); // f.FormatDocument(t, "")
    fourslash::verify_current_file_content(&mut s, r#"const {
    x,
    y,
} = 0;"#);
}
