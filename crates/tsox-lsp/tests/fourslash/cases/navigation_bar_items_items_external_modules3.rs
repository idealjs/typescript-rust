use tsox_lsp::fourslash::Session;


#[test]
fn navigation_bar_items_items_external_modules3() {
    let content = r#"// @Filename: test/my fil	e.ts
export class Bar {
    public s: string;
}
export var x: number;"#;
    let _s = Session::new_for_test("navigationBarItemsItemsExternalModules3", content);
    // TODO: f.VerifyBaselineDocumentSymbol(t)
}
