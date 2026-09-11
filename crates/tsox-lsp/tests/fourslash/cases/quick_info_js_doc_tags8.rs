use tsox_lsp::fourslash::{self, Session};


#[test]
fn quick_info_js_doc_tags8() {
    let content = r#"// @noEmit: true
// @allowJs: true
// @Filename: quickInfoJsDocTags8.js
/**
 * @typedef {{ [x: string]: any, y: number }} Foo
 */

/**
 * @type {(t: T) => number}
 * @template {Foo} T
 */
const /**/foo = t => t.y;"#;
    let mut s = Session::new_for_test("quickInfoJsDocTags8", content);
    // TODO: f.VerifyBaselineHover(t)
}
