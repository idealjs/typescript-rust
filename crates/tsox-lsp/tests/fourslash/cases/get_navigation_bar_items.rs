use tsox_lsp::fourslash::{self, Session};


#[test]
fn get_navigation_bar_items() {
    let content = r#"class C {
    foo;
    ["bar"]: string;
}"#;
    let mut s = Session::new_for_test("getNavigationBarItems", content);
    // TODO: f.VerifyBaselineDocumentSymbol(t)
}
