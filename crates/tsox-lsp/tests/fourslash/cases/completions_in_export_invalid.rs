use tsox_lsp::fourslash::{self, Session};


#[test]
fn completions_in_export_invalid() {
    let content = r#"function topLevel() {}
if (!!true) {
  const blockScoped = 0;
  export { /**/ };
}"#;
    let mut s = Session::new_for_test("completionsInExport_invalid", content);
    // TODO: f.VerifyCompletions(t, "", &fourslash.CompletionsExpectedList{
}
