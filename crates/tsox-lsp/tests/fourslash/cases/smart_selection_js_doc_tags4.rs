use tsox_lsp::fourslash::{self, Session};


#[ignore = "unimplemented: fourslash.VerifyBaselineSelectionRanges"]
#[test]
fn smart_selection_js_doc_tags4() {
    let content = r#"/**
 * @typedef {object} Foo
 * @property {string} a
 * @property {number} b
 * @property {/**/number} c
 */

/** @type {Foo} */
const foo;"#;
    let mut s = Session::new_for_test("smartSelection_JSDocTags4", content);
    fourslash::unsupported("VerifyBaselineSelectionRanges"); // f.VerifyBaselineSelectionRanges(t)
}
