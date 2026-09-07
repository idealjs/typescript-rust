use tsox_lsp::fourslash::{self, Session};

#[ignore = "unimplemented: fourslash.VerifyBaselineDocumentHighlights"]
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
    let mut s = Session::new(content);
    fourslash::unsupported("VerifyBaselineDocumentHighlights"); // f.VerifyBaselineDocumentHighlights(t, nil /*preferences*/, "")
}
