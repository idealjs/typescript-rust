use tsox_lsp::fourslash::{self, Session};

#[ignore = "unimplemented: fourslash.VerifyQuickInfoAt"]
#[test]
fn numeric_property_names() {
    let content = r#"var /**/t2 = { 0: 1, 1: "" };"#;
    let mut s = Session::new(content);
    fourslash::unsupported("VerifyQuickInfoAt"); // f.VerifyQuickInfoAt(t, "", "var t2: {\n    0: number;\n    1: string;\n}", "")
}
