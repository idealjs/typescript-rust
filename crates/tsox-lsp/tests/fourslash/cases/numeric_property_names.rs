use tsox_lsp::fourslash::{self, Session};


#[test]
fn numeric_property_names() {
    let content = r#"var /**/t2 = { 0: 1, 1: "" };"#;
    let mut s = Session::new_for_test("numericPropertyNames", content);
    fourslash::verify_quick_info_at(&mut s, "", "var t2: {\n    0: number;\n    1: string;\n}", "");
}
