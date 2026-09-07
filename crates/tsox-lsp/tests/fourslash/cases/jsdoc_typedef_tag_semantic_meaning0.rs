use tsox_lsp::fourslash::{self, Session};

#[ignore = "unimplemented: fourslash.VerifyBaselineFindAllReferences"]
#[test]
fn jsdoc_typedef_tag_semantic_meaning0() {
    let content = r#"// @allowJs: true
// @Filename: a.js
/** /*1*/@typedef {number} /*2*/T */
/*3*/const /*4*/T = 1;
/** @type {/*5*/T} */
const n = /*6*/T;"#;
    let mut s = Session::new(content);
    fourslash::unsupported("VerifyBaselineFindAllReferences"); // f.VerifyBaselineFindAllReferences(t, "1", "2", "3", "4", "5", "6")
}
