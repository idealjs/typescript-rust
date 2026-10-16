use tsox_lsp::fourslash::Session;


#[test]
fn find_all_references_js_doc_function_new() {
    let content = r#"// @allowJs: true
// @Filename: Foo.js
/** @type {function (/*1*/new: string, string): string} */
var f;"#;
    let _s = Session::new_for_test("findAllReferencesJSDocFunctionNew", content);
    // TODO: f.VerifyBaselineFindAllReferences(t, "1")
}
