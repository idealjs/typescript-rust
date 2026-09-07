use tsox_lsp::fourslash::{self, Session};

#[ignore = "unimplemented: fourslash.VerifyQuickInfoAt"]
#[test]
fn quick_info_error_signature_fills_in_type_parameter() {
    let content = r#"declare function f<T>(x: number): T;
const x/**/ = f();"#;
    let mut s = Session::new(content);
    fourslash::unsupported("VerifyQuickInfoAt"); // f.VerifyQuickInfoAt(t, "", "const x: unknown", "")
}
