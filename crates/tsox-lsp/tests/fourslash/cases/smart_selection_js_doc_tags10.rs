use tsox_lsp::fourslash::{self, Session};


#[test]
fn smart_selection_js_doc_tags10() {
    let content = r#"/**
 * @template T
 * @extends {/**/Set<T>}
 */
class A extends B {
}"#;
    let mut s = Session::new_for_test("smartSelection_JSDocTags10", content);
    // TODO: f.VerifyBaselineSelectionRanges(t)
}
