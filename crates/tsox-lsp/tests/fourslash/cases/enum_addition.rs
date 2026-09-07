use tsox_lsp::fourslash::{self, Session};

#[ignore = "unimplemented: fourslash.VerifyQuickInfoAt"]
#[test]
fn enum_addition() {
    let content = r#"namespace m { export enum Color { Red } }
var /**/t = m.Color.Red + 1;"#;
    let mut s = Session::new(content);
    fourslash::unsupported("VerifyQuickInfoAt"); // f.VerifyQuickInfoAt(t, "", "var t: number", "")
}
