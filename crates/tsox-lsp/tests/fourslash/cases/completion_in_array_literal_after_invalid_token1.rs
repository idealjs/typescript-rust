use tsox_lsp::fourslash::{self, Session};


#[test]
fn completion_in_array_literal_after_invalid_token1() {
    let content = r#"const pairs: Record<string, [string, string]> = {
  a: ["x",:/*m1*/]
};
"#;
    let mut s = Session::new_for_test("completionInArrayLiteralAfterInvalidToken1", content);
    // TODO: f.VerifyCompletions(t, "m1", &fourslash.CompletionsExpectedList{
}
