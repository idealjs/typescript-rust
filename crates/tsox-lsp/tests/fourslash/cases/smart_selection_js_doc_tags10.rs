use tsox_lsp::fourslash::Session;


#[test]
fn smart_selection_js_doc_tags10() {
    let content = r#"/**
 * @template T
 * @extends {/**/Set<T>}
 */
class A extends B {
}"#;
    let _s = Session::new_for_test("smartSelection_JSDocTags10", content);
    // TODO: f.VerifyBaselineSelectionRanges(t)
}
