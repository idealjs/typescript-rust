use tsox_lsp::fourslash::Session;


#[test]
fn quick_info_js_doc_tags11() {
    let content = r#"// @noEmit: true
// @allowJs: true
// @Filename: quickInfoJsDocTags11.js
/**
 * @param {T1} a
 * @param {T2} b
 * @template {number} T1 Comment T1
 * @template {number} T2 Comment T2
 */
const /**/foo = (a, b) => {};"#;
    let _s = Session::new_for_test("quickInfoJsDocTags11", content);
    // TODO: f.VerifyBaselineHover(t)
}
