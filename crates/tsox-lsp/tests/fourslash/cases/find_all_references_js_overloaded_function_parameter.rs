use tsox_lsp::fourslash::{self, Session};


#[ignore = "unimplemented: fourslash.VerifyBaselineFindAllReferences"]
#[test]
fn find_all_references_js_overloaded_function_parameter() {
    let content = r#"// @allowJs: true
// @checkJs: true
// @Filename: foo.js
/**
 * @overload
 * @param {number} x
 * @returns {number}
 *
 * @overload
 * @param {string} x
 * @returns {string} 
 *
 * @param {unknown} x
 * @returns {unknown} 
 */
function foo(x/*1*/) {
  return x;
}"#;
    let mut s = Session::new_for_test("findAllReferencesJsOverloadedFunctionParameter", content);
    fourslash::unsupported("VerifyBaselineFindAllReferences"); // f.VerifyBaselineFindAllReferences(t, "1")
}
