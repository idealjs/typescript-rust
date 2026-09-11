use tsox_lsp::fourslash::{self, Session};


#[test]
fn navigation_bar_getter_and_setter() {
    let content = r#"class X {
    get x() {}
    set x(value) {
        // Inner declaration should make the setter top-level.
        function f() {}
    }
}"#;
    let mut s = Session::new_for_test("navigationBarGetterAndSetter", content);
    // TODO: f.VerifyBaselineDocumentSymbol(t)
}
