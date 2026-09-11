use tsox_lsp::fourslash::{self, Session};


#[test]
fn quick_info_js_doc_tags14() {
    let content = r#"/**
 * @param {Object} options the args object
 * @param {number} options.a first number
 * @param {number} options.b second number
 * @param {Object} options.c sub-object
 * @param {number} options.c.d third number
 * @param {Function} callback the callback function
 * @returns {number}
 */
function /**/fn(options, callback = null) { }"#;
    let mut s = Session::new_for_test("quickInfoJsDocTags14", content);
    // TODO: f.VerifyBaselineHover(t)
}
