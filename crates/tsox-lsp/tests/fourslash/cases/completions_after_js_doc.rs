use tsox_lsp::fourslash::{self, Session};


#[test]
fn completions_after_js_doc() {
    let content = r#"export interface Foo {
  /** JSDoc */
  /**/foo(): void;
}"#;
    let mut s = Session::new_for_test("completionsAfterJSDoc", content);
    // TODO: f.VerifyCompletions(t, "", &fourslash.CompletionsExpectedList{
}
