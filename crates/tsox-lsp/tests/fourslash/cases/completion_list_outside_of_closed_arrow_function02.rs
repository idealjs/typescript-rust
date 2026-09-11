use tsox_lsp::fourslash::{self, Session};


#[test]
fn completion_list_outside_of_closed_arrow_function02() {
    let content = r#"// no a or b
(a, b) => { }/*1*/"#;
    let mut s = Session::new_for_test("completionListOutsideOfClosedArrowFunction02", content);
    // TODO: f.VerifyCompletions(t, "1", &fourslash.CompletionsExpectedList{
}
