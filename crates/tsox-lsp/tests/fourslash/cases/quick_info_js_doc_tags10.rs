use tsox_lsp::fourslash::Session;


#[test]
fn quick_info_js_doc_tags10() {
    let content = r#"// @noEmit: true
// @allowJs: true
// @Filename: quickInfoJsDocTags10.js
/**
 * @param {T1} a
 * @param {T2} a
 * @template T1,T2 Comment Text
 */
const /**/foo = (a, b) => {};"#;
    let _s = Session::new_for_test("quickInfoJsDocTags10", content);
    // TODO: f.VerifyBaselineHover(t)
}
