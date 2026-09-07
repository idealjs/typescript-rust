use tsox_lsp::fourslash::{self, Session};

#[ignore = "unimplemented: fourslash.VerifyQuickInfoAt"]
#[test]
fn intellisense_in_object_literal() {
    let content = r#"var x = 3;

class Foo {
    static something() {
        return { "prop": /**/x };
    }
}"#;
    let mut s = Session::new(content);
    fourslash::unsupported("VerifyQuickInfoAt"); // f.VerifyQuickInfoAt(t, "", "var x: number", "")
}
