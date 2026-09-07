use tsox_lsp::fourslash::{self, Session};

#[ignore = "generator: t.Skip('Known failing fourslash test')"]
#[test]
fn quick_info_on_generic_with_constraints1() {
    // TODO: t.Skip("Known failing fourslash test")
    let content = r#"interface Fo/*1*/o<T/*2*/T extends Date> {}"#;
    let mut s = Session::new(content);
    fourslash::unsupported("VerifyQuickInfoAt"); // f.VerifyQuickInfoAt(t, "1", "interface Foo<TT extends Date>", "")
    fourslash::unsupported("VerifyQuickInfoAt"); // f.VerifyQuickInfoAt(t, "2", "(type parameter) TT in Foo<TT extends Date>", "")
}
