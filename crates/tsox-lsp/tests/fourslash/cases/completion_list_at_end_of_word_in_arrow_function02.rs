use tsox_lsp::fourslash::{self, Session};


#[test]
fn completion_list_at_end_of_word_in_arrow_function02() {
    let content = r#"(d, defaultIsAnInvalidParameterName) => d/*1*/"#;
    let mut s = Session::new_for_test("completionListAtEndOfWordInArrowFunction02", content);
    // TODO: f.VerifyCompletions(t, "1", &fourslash.CompletionsExpectedList{
}
