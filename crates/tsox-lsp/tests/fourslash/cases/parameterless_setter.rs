use tsox_lsp::fourslash::{self, Session};

#[ignore = "unimplemented: fourslash.VerifyQuickInfoExists"]
#[test]
fn parameterless_setter() {
    let content = r#"class foo {
    get getterOnly() {
        return undefined;
    }
    set setterOnly() { }
}
var obj = new foo();
obj.setterOnly = obj./**/getterOnly;"#;
    let mut s = Session::new(content);
    fourslash::go_to_marker(&mut s, "");
    fourslash::unsupported("VerifyQuickInfoExists"); // f.VerifyQuickInfoExists(t)
}
