use tsox_lsp::fourslash::{self, Session};


#[test]
fn basic_global_completions() {
    let content = r#"// @lib: es5
/*1*/"#;
    let mut s = Session::new_for_test("basicGlobalCompletions", content);
    // TODO: f.VerifyCompletions(t, "1", &fourslash.CompletionsExpectedList{
}
