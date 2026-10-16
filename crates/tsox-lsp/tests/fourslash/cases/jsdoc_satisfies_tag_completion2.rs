use tsox_lsp::fourslash::{self, Session};


#[test]
fn jsdoc_satisfies_tag_completion2() {
    let content = r#"// @noEmit: true
// @allowJS: true
// @checkJs: true
// @filename: /a.js
/**
 * @/**/
 */
const t = { a: 1 };"#;
    let mut s = Session::new_for_test("jsdocSatisfiesTagCompletion2", content);
    fourslash::verify_completions_include_exclude_at(&mut s, Some(""), &["satisfies"], &[]);
}
