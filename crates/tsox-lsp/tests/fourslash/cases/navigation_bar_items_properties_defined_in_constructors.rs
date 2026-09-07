use tsox_lsp::fourslash::{self, Session};

#[ignore = "unimplemented: fourslash.VerifyBaselineDocumentSymbol"]
#[test]
fn navigation_bar_items_properties_defined_in_constructors() {
    let content = r#"class List<T> {
    constructor(public a: boolean, private b: T, readonly c: string, d: number) {
        var local = 0;
    }
}"#;
    let mut s = Session::new(content);
    fourslash::unsupported("VerifyBaselineDocumentSymbol"); // f.VerifyBaselineDocumentSymbol(t)
}
