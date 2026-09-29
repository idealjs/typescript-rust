use tsox_lsp::fourslash::Session;


#[test]
fn smart_selection_js_doc_tags6() {
    let content = r#"/**
 * @template T
 * @param {/**/T} x
 * @return {T}
 */
function foo(x) {
    return x;
}"#;
    let _s = Session::new_for_test("smartSelection_JSDocTags6", content);
    // TODO: f.VerifyBaselineSelectionRanges(t)
}
