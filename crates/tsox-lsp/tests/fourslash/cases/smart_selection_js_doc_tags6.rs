use tsox_lsp::fourslash::{self, Session};


#[ignore = "unimplemented: fourslash.VerifyBaselineSelectionRanges"]
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
    let mut s = Session::new_for_test("smartSelection_JSDocTags6", content);
    fourslash::unsupported("VerifyBaselineSelectionRanges"); // f.VerifyBaselineSelectionRanges(t)
}
