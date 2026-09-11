use tsox_lsp::fourslash::{self, Session};


#[test]
fn smart_selection_js_doc_tags8() {
    let content = r#"/**
 * @this {/*1*/Foo}
 * @param {/*2*/*} e
 */
function callback(e) {
}"#;
    let mut s = Session::new_for_test("smartSelection_JSDocTags8", content);
    // TODO: f.VerifyBaselineSelectionRanges(t)
}
