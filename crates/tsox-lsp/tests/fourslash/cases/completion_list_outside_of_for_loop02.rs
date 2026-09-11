use tsox_lsp::fourslash::{self, Session};


#[test]
fn completion_list_outside_of_for_loop02() {
    let content = r#"for (let i = 0; i < 10; i++);/*1*/"#;
    let mut s = Session::new_for_test("completionListOutsideOfForLoop02", content);
    // TODO: f.VerifyCompletions(t, "1", &fourslash.CompletionsExpectedList{
}
