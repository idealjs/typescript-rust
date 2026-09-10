use tsox_lsp::fourslash::{self, Session};


#[ignore = "unimplemented: fourslash.VerifyBaselineDocumentSymbol"]
#[test]
fn get_navigation_bar_items() {
    let content = r#"class C {
    foo;
    ["bar"]: string;
}"#;
    let mut s = Session::new_for_test("getNavigationBarItems", content);
    fourslash::unsupported("VerifyBaselineDocumentSymbol"); // f.VerifyBaselineDocumentSymbol(t)
}
