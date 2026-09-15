use tsox_lsp::fourslash::Session;


#[test]
fn navigation_bar_items_static_and_non_static_no_merge() {
    let content = r#"class C {
    static x;
    x;
}"#;
    let _s = Session::new_for_test("navigationBarItemsStaticAndNonStaticNoMerge", content);
    // TODO: f.VerifyBaselineDocumentSymbol(t)
}
