use tsox_lsp::fourslash::Session;


#[test]
fn empty_export_find_references() {
    let content = r#"// @allowNonTsExtensions: true
// @Filename: Foo.js
/**/module.exports = {

}"#;
    let _s = Session::new_for_test("emptyExportFindReferences", content);
    // TODO: f.VerifyBaselineDocumentHighlights(t, nil /*preferences*/, "")
}
