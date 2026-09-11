use tsox_lsp::fourslash::{self, Session};


#[test]
fn completions_asserts() {
    let content = r#"declare function assert(argument1: any): asserts a/**/"#;
    let mut s = Session::new_for_test("completionsAsserts", content);
    // TODO: f.VerifyCompletions(t, "", &fourslash.CompletionsExpectedList{
}
