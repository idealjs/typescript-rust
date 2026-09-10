use tsox_lsp::fourslash::{self, Session};


#[ignore = "unimplemented: fourslash.VerifyBaselineSelectionRanges"]
#[test]
fn smart_selection_js_doc_tags9() {
    let content = r#"/** @enum {/**/number} */
const Foo = {
    x: 0,
    y: 1,
};"#;
    let mut s = Session::new_for_test("smartSelection_JSDocTags9", content);
    fourslash::unsupported("VerifyBaselineSelectionRanges"); // f.VerifyBaselineSelectionRanges(t)
}
