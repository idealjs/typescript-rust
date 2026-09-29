use tsox_lsp::fourslash::Session;


#[test]
fn smart_selection_js_doc_tags2() {
    let content = r#"/**
 * @type {/**/string}
 */
const foo;"#;
    let _s = Session::new_for_test("smartSelection_JSDocTags2", content);
    // TODO: f.VerifyBaselineSelectionRanges(t)
}
