use tsox_lsp::fourslash::{self, Session};


#[test]
fn jsdoc_prop_tag_completion() {
    let content = r#"/**
 * @typedef Foo
 * @pr/**/
 */"#;
    let mut s = Session::new_for_test("jsdocPropTagCompletion", content);
    fourslash::verify_completions_include_exclude_at(&mut s, Some(""), &["prop"], &[]);
}
