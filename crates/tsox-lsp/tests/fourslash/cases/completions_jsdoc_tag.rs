use tsox_lsp::fourslash::{self, Session};


#[test]
fn completions_jsdoc_tag() {
    let content = r#"/**
 * @typedef {object} T
 * /**/
 */"#;
    let mut s = Session::new_for_test("completionsJsdocTag", content);
    // TODO: f.VerifyCompletions(t, "", &fourslash.CompletionsExpectedList{
}
