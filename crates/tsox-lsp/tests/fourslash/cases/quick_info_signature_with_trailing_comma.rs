use tsox_lsp::fourslash::{self, Session};

#[ignore = "unimplemented: fourslash.VerifyQuickInfoAt"]
#[test]
fn quick_info_signature_with_trailing_comma() {
    let content = r#"declare function f<T>(a: T): T;
/**/f(2,);"#;
    let mut s = Session::new(content);
    fourslash::unsupported("VerifyQuickInfoAt"); // f.VerifyQuickInfoAt(t, "", "function f<2>(a: 2): 2", "")
}
