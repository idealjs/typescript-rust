use tsox_lsp::fourslash::{self, Session};

#[ignore = "unimplemented: fourslash.VerifyBaselineSignatureHelp"]
#[test]
fn quick_info_js_doc_tags13() {
    let content = r#"// @allowJs: true
// @checkJs: true
// @filename: ./a.js
/**
 * First overload
 * @overload
 * @param {number} a
 * @returns {void}
 */

/**
 * Second overload
 * @overload
 * @param {string} a
 * @returns {void}
 */

/**
 * @param {string | number} a
 * @returns {void}
 */
function f(a) {}

f(/*a*/1);
f(/*b*/"");"#;
    let mut s = Session::new(content);
    fourslash::unsupported("VerifyBaselineSignatureHelp"); // f.VerifyBaselineSignatureHelp(t)
}
