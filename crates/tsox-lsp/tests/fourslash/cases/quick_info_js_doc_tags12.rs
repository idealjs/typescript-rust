use tsox_lsp::fourslash::{self, Session};


#[test]
fn quick_info_js_doc_tags12() {
    let content = r#"/**
 * @param {Object} options the args object
 * @param {number} options.a first number
 * @param {number} options.b second number
 * @param {Function} callback the callback function
 * @returns {number}
 */
function /**/f(options, callback = null) {
}"#;
    let mut s = Session::new_for_test("quickInfoJsDocTags12", content);
    // TODO: f.VerifyBaselineHover(t)
}
