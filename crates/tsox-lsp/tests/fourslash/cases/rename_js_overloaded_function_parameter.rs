use tsox_lsp::fourslash::{self, Session};

#[ignore = "unimplemented: fourslash.VerifyBaselineRename"]
#[test]
fn rename_js_overloaded_function_parameter() {
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
function foo(x/**/) {
  return x;
}"#;
    let mut s = Session::new(content);
    fourslash::unsupported("VerifyBaselineRename"); // f.VerifyBaselineRename(t, nil /*preferences*/, "")
}
