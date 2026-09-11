use tsox_lsp::fourslash::{self, Session};


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
    let mut s = Session::new_for_test("quickInfoJsDocTags13", content);
    // TODO: f.VerifyBaselineSignatureHelp(t)
}
