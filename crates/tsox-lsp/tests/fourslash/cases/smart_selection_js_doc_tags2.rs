use tsox_lsp::fourslash::{self, Session};


#[ignore = "unimplemented: fourslash.VerifyBaselineSelectionRanges"]
#[test]
fn smart_selection_js_doc_tags2() {
    let content = r#"/**
 * @type {/**/string}
 */
const foo;"#;
    let mut s = Session::new_for_test("smartSelection_JSDocTags2", content);
    fourslash::unsupported("VerifyBaselineSelectionRanges"); // f.VerifyBaselineSelectionRanges(t)
}
