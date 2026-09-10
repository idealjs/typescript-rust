use tsox_lsp::fourslash::{self, Session};


#[ignore = "unimplemented: fourslash.VerifyBaselineFindAllReferences"]
#[test]
fn find_all_references_js_doc_function_this() {
    let content = r#"// @allowJs: true
// @Filename: Foo.js
/** @type {function (this: string, string): string} */
var f = function (s) { return /*0*/this + s; }"#;
    let mut s = Session::new_for_test("findAllReferencesJSDocFunctionThis", content);
    fourslash::unsupported("VerifyBaselineFindAllReferences"); // f.VerifyBaselineFindAllReferences(t, "0")
}
