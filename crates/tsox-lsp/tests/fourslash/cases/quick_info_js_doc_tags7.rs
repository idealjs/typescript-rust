use tsox_lsp::fourslash::{self, Session};


#[ignore = "unimplemented: fourslash.VerifyBaselineHover"]
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
    let mut s = Session::new_for_test("quickInfoJsDocTags7", content);
    fourslash::unsupported("VerifyBaselineHover"); // f.VerifyBaselineHover(t)
}
