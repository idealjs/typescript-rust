use tsox_lsp::fourslash::{self, Session};


#[test]
fn completion_list_at_end_of_word_in_arrow_function03() {
    let content = r#"(d, defaultIsAnInvalidParameterName) => default/*1*/"#;
    let mut s = Session::new_for_test("completionListAtEndOfWordInArrowFunction03", content);
    // TODO: f.VerifyCompletions(t, "1", &fourslash.CompletionsExpectedList{
}
