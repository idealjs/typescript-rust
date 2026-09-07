use tsox_lsp::fourslash::{self, Session};

#[ignore = "generator: }"]
#[test]
fn quick_info_recursive_object_literal() {
    let content = r#"var a = { f: /**/a"#;
    let mut s = Session::new(content);
    fourslash::unsupported("VerifyQuickInfoAt"); // f.VerifyQuickInfoAt(t, "", "var a: any", "")
    // TODO: }
}
