use tsox_lsp::fourslash::{self, Session};


#[test]
fn jsdoc_overload_tag_completion() {
    let content = r#"// @allowJS: true
// @checkJs: true
// @filename: /a.js
/**
 * @/**/
 */"#;
    let mut s = Session::new_for_test("jsdocOverloadTagCompletion", content);
    fourslash::verify_completions_include_exclude_at(&mut s, Some(""), &["overload"], &[]);
}
