use tsox_lsp::fourslash::{self, Session};


#[ignore = "unimplemented: fourslash.VerifyBaselineSelectionRanges"]
#[test]
fn smart_selection_js_doc_tags7() {
    let content = r#"/**
 * @constructor
 * @param {/**/number} data
 */
function Foo(data) {
}"#;
    let mut s = Session::new_for_test("smartSelection_JSDocTags7", content);
    fourslash::unsupported("VerifyBaselineSelectionRanges"); // f.VerifyBaselineSelectionRanges(t)
}
