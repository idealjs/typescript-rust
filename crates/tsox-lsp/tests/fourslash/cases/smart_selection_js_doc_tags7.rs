use tsox_lsp::fourslash::Session;


#[test]
fn smart_selection_js_doc_tags7() {
    let content = r#"/**
 * @constructor
 * @param {/**/number} data
 */
function Foo(data) {
}"#;
    let _s = Session::new_for_test("smartSelection_JSDocTags7", content);
    // TODO: f.VerifyBaselineSelectionRanges(t)
}
