use tsox_lsp::fourslash::{self, Session};


#[test]
fn jsdoc_import_tag_completion1() {
    let content = r#"// @allowJS: true
// @checkJs: true
// @filename: /a.js
/**
 * @/**/
 */"#;
    let mut s = Session::new_for_test("jsdocImportTagCompletion1", content);
    fourslash::verify_completions_include_exclude_at(&mut s, Some(""), &["import"], &[]);
}
