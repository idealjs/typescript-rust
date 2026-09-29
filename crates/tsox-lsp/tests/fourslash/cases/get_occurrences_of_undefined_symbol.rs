use tsox_lsp::fourslash::Session;


#[test]
fn get_occurrences_of_undefined_symbol() {
    let content = r#"var obj1: {
    (bar: any): any;
    new (bar: any): any;
    [bar: any]: any;
    bar: any;
    foob(bar: any): any;
};

class cls3 {
    property zeFunc() {
    super.ceFun/**/c();
}
}"#;
    let _s = Session::new_for_test("getOccurrencesOfUndefinedSymbol", content);
    // TODO: f.VerifyBaselineDocumentHighlights(t, nil /*preferences*/, "")
}
