use tsox_lsp::fourslash::{self, Session};

#[ignore = "unimplemented: fourslash.VerifyQuickInfoAt"]
#[test]
fn quick_info_call_property() {
    let content = r#"interface I {
    /** Doc */
    m: () => void;
}
function f(x: I): void {
    x./**/m();
}"#;
    let mut s = Session::new(content);
    fourslash::unsupported("VerifyQuickInfoAt"); // f.VerifyQuickInfoAt(t, "", "(property) I.m: () => void", "Doc")
}
