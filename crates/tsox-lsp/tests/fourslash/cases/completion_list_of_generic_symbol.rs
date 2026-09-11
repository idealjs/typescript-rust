use tsox_lsp::fourslash::{self, Session};


#[test]
fn completion_list_of_generic_symbol() {
    let content = r#"var a = [1,2,3];
a./**/"#;
    let mut s = Session::new_for_test("completionListOfGenericSymbol", content);
    // TODO: f.VerifyCompletions(t, "", &fourslash.CompletionsExpectedList{
}
