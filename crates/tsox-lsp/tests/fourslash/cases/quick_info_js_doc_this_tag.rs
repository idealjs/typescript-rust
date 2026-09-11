use tsox_lsp::fourslash::{self, Session};


#[test]
fn quick_info_js_doc_this_tag() {
    let content = r#"// @strict: true
// @filename: /a.ts
/** @this {number} */
function f/**/() {
    this
}"#;
    let mut s = Session::new_for_test("quickInfoJsDocThisTag", content);
    // TODO: f.VerifyBaselineHover(t)
}
