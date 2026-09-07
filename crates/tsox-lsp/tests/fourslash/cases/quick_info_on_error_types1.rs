use tsox_lsp::fourslash::{self, Session};

#[ignore = "unimplemented: fourslash.VerifyQuickInfoAt"]
#[test]
fn quick_info_on_error_types1() {
    let content = r#"var /*A*/f: {
    x: number;
    <
};"#;
    let mut s = Session::new(content);
    fourslash::unsupported("VerifyQuickInfoAt"); // f.VerifyQuickInfoAt(t, "A", "var f: {\n    (): any;\n    x: number;\n}", "")
}
