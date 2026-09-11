use tsox_lsp::fourslash::{self, Session};


#[test]
fn smart_selection_js_doc_tags3() {
    let content = r#"/**
 * @param {/**/string} x
 */
function foo(x) {}"#;
    let mut s = Session::new_for_test("smartSelection_JSDocTags3", content);
    // TODO: f.VerifyBaselineSelectionRanges(t)
}
