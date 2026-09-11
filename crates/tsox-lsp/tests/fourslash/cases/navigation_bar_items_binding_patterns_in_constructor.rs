use tsox_lsp::fourslash::{self, Session};


#[test]
fn navigation_bar_items_binding_patterns_in_constructor() {
    let content = r#"class A {
    x: any
    constructor([a]: any) {
    }
}
class B {
    x: any;
    constructor( {a} = { a: 1 }) {
    }
}"#;
    let mut s = Session::new_for_test("navigationBarItemsBindingPatternsInConstructor", content);
    // TODO: f.VerifyBaselineDocumentSymbol(t)
}
