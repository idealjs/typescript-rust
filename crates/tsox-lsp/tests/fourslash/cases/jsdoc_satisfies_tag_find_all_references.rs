use tsox_lsp::fourslash::Session;


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
    let _s = Session::new_for_test("jsdocSatisfiesTagFindAllReferences", content);
    // TODO: f.VerifyBaselineFindAllReferences(t, "")
}
