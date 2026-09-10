use tsox_lsp::fourslash::{self, Session};


#[ignore = "unimplemented: fourslash.VerifyCompletions"]
#[test]
fn completion_list_of_generic_symbol() {
    let content = r#"var a = [1,2,3];
a./**/"#;
    let mut s = Session::new_for_test("completionListOfGenericSymbol", content);
    fourslash::unsupported("VerifyCompletions"); // f.VerifyCompletions(t, "", &fourslash.CompletionsExpectedList{
}
