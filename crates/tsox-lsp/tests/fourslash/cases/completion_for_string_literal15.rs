use tsox_lsp::fourslash::{self, Session};


#[test]
fn completion_for_string_literal15() {
    let content = r#"let x: { [_ in "foo"]: string } = {
    "[|/**/|]"
}"#;
    let mut s = Session::new_for_test("completionForStringLiteral15", content);
    // TODO: f.VerifyCompletions(t, "", &fourslash.CompletionsExpectedList{
}
