use tsox_lsp::fourslash::{self, Session};


#[test]
fn format_object_binding_pattern() {
    let content = r#"const {
x,
y,
} = 0;"#;
    let mut s = Session::new_for_test("formatObjectBindingPattern", content);
    fourslash::format_document(&mut s, "");
    fourslash::verify_current_file_content(&mut s, r#"const {
    x,
    y,
} = 0;"#);
}
