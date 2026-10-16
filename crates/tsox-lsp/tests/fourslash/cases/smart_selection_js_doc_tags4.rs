use tsox_lsp::fourslash::Session;


#[test]
fn smart_selection_js_doc_tags4() {
    let content = r#"/**
 * @typedef {object} Foo
 * @property {string} a
 * @property {number} b
 * @property {/**/number} c
 */

/** @type {Foo} */
const foo;"#;
    let _s = Session::new_for_test("smartSelection_JSDocTags4", content);
    // TODO: f.VerifyBaselineSelectionRanges(t)
}
