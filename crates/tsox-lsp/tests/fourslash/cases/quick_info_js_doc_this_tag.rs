use tsox_lsp::fourslash::Session;


#[test]
fn quick_info_js_doc_this_tag() {
    let content = r#"// @strict: true
// @filename: /a.ts
/** @this {number} */
function f/**/() {
    this
}"#;
    let _s = Session::new_for_test("quickInfoJsDocThisTag", content);
    // TODO: f.VerifyBaselineHover(t)
}
