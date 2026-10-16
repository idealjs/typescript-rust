use tsox_lsp::fourslash::Session;


#[test]
fn smart_selection_js_doc_tags1() {
    let content = r#"/**
 * @returns {Array<{ value: /**/string }>}
 */
function foo() { return [] }"#;
    let _s = Session::new_for_test("smartSelection_JSDocTags1", content);
    // TODO: f.VerifyBaselineSelectionRanges(t)
}
