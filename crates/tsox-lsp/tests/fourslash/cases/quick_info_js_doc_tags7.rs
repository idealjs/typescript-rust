use tsox_lsp::fourslash::Session;


#[test]
fn quick_info_js_doc_tags7() {
    let content = r#"// @noEmit: true
// @allowJs: true
// @Filename: quickInfoJsDocTags7.js
/**
 * @typedef {{ [x: string]: any, y: number }} Foo
 */

/**
 * @type {(t: T) => number}
 * @template T
 */
const /**/foo = t => t.y;"#;
    let _s = Session::new_for_test("quickInfoJsDocTags7", content);
    // TODO: f.VerifyBaselineHover(t)
}
