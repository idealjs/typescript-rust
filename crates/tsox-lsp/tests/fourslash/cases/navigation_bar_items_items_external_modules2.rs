use tsox_lsp::fourslash::{self, Session};


#[test]
fn navigation_bar_items_items_external_modules2() {
    let content = r#"// @Filename: test/file.ts
export class Bar {
    public s: string;
}
export var x: number;"#;
    let mut s = Session::new_for_test("navigationBarItemsItemsExternalModules2", content);
    // TODO: f.VerifyBaselineDocumentSymbol(t)
}
