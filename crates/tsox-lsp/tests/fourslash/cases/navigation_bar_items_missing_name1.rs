use tsox_lsp::fourslash::Session;


#[test]
fn navigation_bar_items_missing_name1() {
    let content = r#"export function
class C {
    foo() {}
}"#;
    let _s = Session::new_for_test("navigationBarItemsMissingName1", content);
    // TODO: f.VerifyBaselineDocumentSymbol(t)
}
