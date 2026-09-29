use tsox_lsp::fourslash::Session;


#[test]
fn smart_selection_js_doc_tags9() {
    let content = r#"/** @enum {/**/number} */
const Foo = {
    x: 0,
    y: 1,
};"#;
    let _s = Session::new_for_test("smartSelection_JSDocTags9", content);
    // TODO: f.VerifyBaselineSelectionRanges(t)
}
