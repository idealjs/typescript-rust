use tsox_lsp::fourslash::Session;


#[test]
fn jsdoc_typedef_tag_semantic_meaning0() {
    let content = r#"// @allowJs: true
// @Filename: a.js
/** /*1*/@typedef {number} /*2*/T */
/*3*/const /*4*/T = 1;
/** @type {/*5*/T} */
const n = /*6*/T;"#;
    let _s = Session::new_for_test("jsdocTypedefTagSemanticMeaning0", content);
    // TODO: f.VerifyBaselineFindAllReferences(t, "1", "2", "3", "4", "5", "6")
}
