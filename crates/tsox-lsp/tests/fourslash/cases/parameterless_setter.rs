use tsox_lsp::fourslash::{self, Session};


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
    let mut s = Session::new_for_test("parameterlessSetter", content);
    fourslash::go_to_marker(&mut s, "");
    // TODO: f.VerifyQuickInfoExists(t)
}
