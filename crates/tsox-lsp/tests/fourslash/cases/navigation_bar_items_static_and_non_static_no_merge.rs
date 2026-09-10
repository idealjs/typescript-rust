use tsox_lsp::fourslash::{self, Session};


#[ignore = "unimplemented: fourslash.VerifyBaselineDocumentSymbol"]
#[test]
fn navigation_bar_items_static_and_non_static_no_merge() {
    let content = r#"class C {
    static x;
    x;
}"#;
    let mut s = Session::new_for_test("navigationBarItemsStaticAndNonStaticNoMerge", content);
    fourslash::unsupported("VerifyBaselineDocumentSymbol"); // f.VerifyBaselineDocumentSymbol(t)
}
