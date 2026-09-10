use tsox_lsp::fourslash::{self, Session};


#[ignore = "unimplemented: fourslash.FormatDocument"]
#[test]
fn format_object_binding_pattern_rest_element_with_property_name() {
    let content = r#"const { ...a: b } = {};"#;
    let mut s = Session::new_for_test("formatObjectBindingPattern_restElementWithPropertyName", content);
    fourslash::unsupported("FormatDocument"); // f.FormatDocument(t, "")
    fourslash::verify_current_file_content(&mut s, r#"const { ...a: b } = {};"#);
}
