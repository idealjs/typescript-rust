use tsox_lsp::fourslash::{self, Session};


#[test]
fn completion_list_outside_of_for_loop01() {
    let content = r#"for (let i = 0; i < 10; i++) i;/*1*/"#;
    let mut s = Session::new_for_test("completionListOutsideOfForLoop01", content);
    fourslash::go_to_marker(&mut s, "1");
    // TODO: f.VerifyCompletions(t, "1", &fourslash.CompletionsExpectedList{
}
