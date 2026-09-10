use tsox_lsp::fourslash::{self, Session};


#[ignore = "unimplemented: fourslash.VerifyBaselineFindAllReferences"]
#[test]
fn find_all_references_js_doc_type_literal() {
    let content = r#"// @allowJs: true
// @checkJs: true
// @Filename: foo.js
/**
 * @param {object} o - very important!
 * @param {string} o.x - a thing, its ok
 * @param {number} o.y - another thing
 * @param {Object} o.nested - very nested
 * @param {boolean} o.nested./*1*/great - much greatness
 * @param {number} o.nested.times - twice? probably!??
 */
 function f(o) { return o.nested./*2*/great; }"#;
    let mut s = Session::new_for_test("findAllReferencesJsDocTypeLiteral", content);
    fourslash::unsupported("VerifyBaselineFindAllReferences"); // f.VerifyBaselineFindAllReferences(t, "1", "2")
}
