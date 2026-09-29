use tsox_lsp::fourslash::Session;


#[test]
fn quick_info_js_doc_tags_typedef() {
    let content = r#"// @noEmit: true
// @allowJs: true
// @Filename: quickInfoJsDocTagsTypedef.js
/**
 * Bar comment
 * @typedef {Object} /*1*/Bar
 * @property {string} baz - baz comment
 * @property {string} qux - qux comment
 */

/**
 * foo comment
 * @param {/*2*/Bar} x - x comment
 * @returns {Bar}
 */
function foo(x) {
    return x;
}"#;
    let _s = Session::new_for_test("quickInfoJsDocTagsTypedef", content);
    // TODO: f.VerifyBaselineHover(t)
}
