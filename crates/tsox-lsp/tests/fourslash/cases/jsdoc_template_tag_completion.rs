use tsox_lsp::fourslash::{self, Session};


#[ignore = "unimplemented: fourslash.VerifyCompletions"]
#[test]
fn jsdoc_template_tag_completion() {
    let content = r#"// @lib: es5
/**
 * @template {/**/} T
 * @typedef {Object} Foo
 * @property {T} foo
 */"#;
    let mut s = Session::new_for_test("jsdocTemplateTagCompletion", content);
    fourslash::unsupported("VerifyCompletions"); // f.VerifyCompletions(t, "", &fourslash.CompletionsExpectedList{
}
