use tsox_lsp::fourslash::{self, Session};


#[test]
fn quick_info_js_doc_tags_callback() {
    let content = r#"// @noEmit: true
// @allowJs: true
// @Filename: quickInfoJsDocTagsCallback.js
/**
 * @callback cb/*1*/
 * @param {string} x - x comment
 */

/**
 * @param {/*2*/cb} bar -callback comment
 */
function foo(bar) {
    bar(bar);
}"#;
    let mut s = Session::new_for_test("quickInfoJsDocTagsCallback", content);
    // TODO: f.VerifyBaselineHover(t)
}
