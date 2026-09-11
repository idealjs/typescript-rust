use tsox_lsp::fourslash::{self, Session};


#[test]
fn jsdoc_typedef_tag_semantic_meaning1() {
    let content = r#"// @allowJs: true
// @Filename: a.js
/** @typedef {number} */
/*1*/const /*2*/T = 1;
/** @type {/*3*/T} */
const n = /*4*/T;"#;
    let mut s = Session::new_for_test("jsdocTypedefTagSemanticMeaning1", content);
    // TODO: f.VerifyBaselineFindAllReferences(t, "1", "2", "3", "4")
}
