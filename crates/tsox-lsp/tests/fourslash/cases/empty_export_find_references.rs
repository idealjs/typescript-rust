use tsox_lsp::fourslash::{self, Session};


#[test]
fn empty_export_find_references() {
    let content = r#"// @allowNonTsExtensions: true
// @Filename: Foo.js
/**/module.exports = {

}"#;
    let mut s = Session::new_for_test("emptyExportFindReferences", content);
    // TODO: f.VerifyBaselineDocumentHighlights(t, nil /*preferences*/, "")
}
