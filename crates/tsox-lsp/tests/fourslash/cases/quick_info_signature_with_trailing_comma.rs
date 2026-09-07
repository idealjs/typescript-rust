use tsox_lsp::fourslash::{self, Session};

#[test]
fn quick_info_signature_with_trailing_comma() {
    let content = r#"declare function f<T>(a: T): T;
/**/f(2,);"#;
    let mut s = Session::new(content);
    fourslash::verify_quick_info_at(&mut s, "", "function f<2>(a: 2): 2", "");
}
