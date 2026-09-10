use tsox_lsp::fourslash::{self, Session};


#[ignore = "unimplemented: fourslash.VerifyBaselineFindAllReferences"]
#[test]
fn jsdoc_satisfies_tag_find_all_references() {
    let content = r#"// @noEmit: true
// @allowJS: true
// @checkJs: true
// @filename: /a.js
/**
 * @typedef {Object} T
 * @property {number} a
 */

/** @satisfies {/**/T} comment */
const foo = { a: 1 };"#;
    let mut s = Session::new_for_test("jsdocSatisfiesTagFindAllReferences", content);
    fourslash::unsupported("VerifyBaselineFindAllReferences"); // f.VerifyBaselineFindAllReferences(t, "")
}
