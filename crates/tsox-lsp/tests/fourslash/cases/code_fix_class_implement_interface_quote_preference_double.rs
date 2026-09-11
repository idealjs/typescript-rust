use tsox_lsp::fourslash::{self, Session};


#[test]
fn code_fix_class_implement_interface_quote_preference_double() {
    let content = r#"interface I {
    a(): void;
    b(x: "x", y: "a" | "b"): "b";

    c: "c";
    d: { e: "e"; };
}
class Foo implements I {}"#;
    let mut s = Session::new_for_test("codeFixClassImplementInterface_quotePreferenceDouble", content);
    // TODO: f.VerifyCodeFix(t, fourslash.VerifyCodeFixOptions{
}
