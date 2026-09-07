use tsox_lsp::fourslash::{self, Session};

#[ignore = "unimplemented: fourslash.VerifyQuickInfoAt"]
#[test]
fn comments_union() {
    let content = r#"var a: Array<string> | Array<number>;
a./*1*/length"#;
    let mut s = Session::new(content);
    fourslash::unsupported("VerifyQuickInfoAt"); // f.VerifyQuickInfoAt(t, "1", "(property) Array<T>.length: number", "Gets or sets the length of the ar
}
