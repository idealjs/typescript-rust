use tsox_lsp::fourslash::{self, Session};


#[test]
fn jsdoc_template_tag_completion() {
    let content = r#"// @lib: es5
/**
 * @template {/**/} T
 * @typedef {Object} Foo
 * @property {T} foo
 */"#;
    let mut s = Session::new_for_test("jsdocTemplateTagCompletion", content);
    fourslash::go_to_marker(&mut s, "");
    // TODO: f.VerifyCompletions(t, "", &fourslash.CompletionsExpectedList{
}
