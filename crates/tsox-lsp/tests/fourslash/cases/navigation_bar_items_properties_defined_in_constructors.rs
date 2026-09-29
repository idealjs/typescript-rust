use tsox_lsp::fourslash::Session;


#[test]
fn navigation_bar_items_properties_defined_in_constructors() {
    let content = r#"class List<T> {
    constructor(public a: boolean, private b: T, readonly c: string, d: number) {
        var local = 0;
    }
}"#;
    let _s = Session::new_for_test("navigationBarItemsPropertiesDefinedInConstructors", content);
    // TODO: f.VerifyBaselineDocumentSymbol(t)
}
