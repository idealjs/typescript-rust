use tsox_lsp::fourslash::Session;


#[test]
fn smart_selection_js_doc_tags5() {
    let content = r#"/**
 * @callback Foo
 * @param {string} data
 * @param {/**/number} [index] - comment
 * @return {boolean}
 */

/** @type {Foo} */
const foo = s => !(s.length % 2);"#;
    let _s = Session::new_for_test("smartSelection_JSDocTags5", content);
    // TODO: f.VerifyBaselineSelectionRanges(t)
}
