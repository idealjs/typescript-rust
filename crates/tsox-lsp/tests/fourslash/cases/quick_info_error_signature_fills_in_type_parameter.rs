use tsox_lsp::fourslash::{self, Session};

#[test]
fn quick_info_error_signature_fills_in_type_parameter() {
    let content = r#"declare function f<T>(x: number): T;
const x/**/ = f();"#;
    let mut s = Session::new(content);
    fourslash::verify_quick_info_at(&mut s, "", "const x: unknown", "");
}
