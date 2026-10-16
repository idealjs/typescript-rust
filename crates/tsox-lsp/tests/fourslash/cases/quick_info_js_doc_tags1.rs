use tsox_lsp::fourslash::Session;


#[test]
fn quick_info_js_doc_tags1() {
    let content = r#"// @Filename: quickInfoJsDocTags1.ts
/**
 * Doc
 * @author Me <me@domain.tld>
 * @augments {C<T>} Augments it
 * @template T A template
 * @type {number | string} A type
 * @typedef {number | string} NumOrStr
 * @property {number} x The prop
 * @param {number} x The param
 * @returns The result
 * @see x (the parameter)
 */
function /**/foo(x) {}"#;
    let _s = Session::new_for_test("quickInfoJsDocTags1", content);
    // TODO: f.VerifyBaselineHover(t)
}
