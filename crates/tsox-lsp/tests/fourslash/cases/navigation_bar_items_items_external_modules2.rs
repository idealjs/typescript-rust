use tsox_lsp::fourslash::{self, Session};


#[ignore = "unimplemented: fourslash.VerifyBaselineDocumentSymbol"]
#[test]
fn navigation_bar_items_items_external_modules2() {
    let content = r#"// @Filename: test/file.ts
export class Bar {
    public s: string;
}
export var x: number;"#;
    let mut s = Session::new_for_test("navigationBarItemsItemsExternalModules2", content);
    fourslash::unsupported("VerifyBaselineDocumentSymbol"); // f.VerifyBaselineDocumentSymbol(t)
}
