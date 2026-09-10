use tsox_lsp::fourslash::{self, Session};


#[ignore = "unimplemented: fourslash.VerifyBaselineHover"]
#[test]
fn quick_info_satisfies_tag() {
    let content = r#"// @noEmit: true
// @allowJS: true
// @checkJs: true
// @filename: /a.js
/** @satisfies {number} comment */
const /*1*/a = 1;"#;
    let mut s = Session::new_for_test("quickInfoSatisfiesTag", content);
    fourslash::unsupported("VerifyBaselineHover"); // f.VerifyBaselineHover(t)
}
