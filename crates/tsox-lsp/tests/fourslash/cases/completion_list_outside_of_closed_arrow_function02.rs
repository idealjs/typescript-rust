use tsox_lsp::fourslash::{self, Session};


#[ignore = "unimplemented: fourslash.VerifyCompletions"]
#[test]
fn completion_list_outside_of_closed_arrow_function02() {
    let content = r#"// no a or b
(a, b) => { }/*1*/"#;
    let mut s = Session::new_for_test("completionListOutsideOfClosedArrowFunction02", content);
    fourslash::unsupported("VerifyCompletions"); // f.VerifyCompletions(t, "1", &fourslash.CompletionsExpectedList{
}
