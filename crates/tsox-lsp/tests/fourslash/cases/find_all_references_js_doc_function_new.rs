use tsox_lsp::fourslash::{self, Session};


#[ignore = "unimplemented: fourslash.VerifyBaselineFindAllReferences"]
#[test]
fn find_all_references_js_doc_function_new() {
    let content = r#"// @allowJs: true
// @Filename: Foo.js
/** @type {function (/*1*/new: string, string): string} */
var f;"#;
    let mut s = Session::new_for_test("findAllReferencesJSDocFunctionNew", content);
    fourslash::unsupported("VerifyBaselineFindAllReferences"); // f.VerifyBaselineFindAllReferences(t, "1")
}
