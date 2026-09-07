use tsox_lsp::fourslash::{self, Session};

#[ignore = "unimplemented: fourslash.VerifyQuickInfoAt"]
#[test]
fn contextual_typing_from_type_assertion1() {
    let content =
        r#"var f3 = <(x: string) => string> function (/**/x) { return x.toLowerCase(); };"#;
    let mut s = Session::new(content);
    fourslash::unsupported("VerifyQuickInfoAt"); // f.VerifyQuickInfoAt(t, "", "(parameter) x: string", "")
}
