use tsox_lsp::fourslash::{self, Session};


#[ignore = "unimplemented: fourslash.VerifyCompletions"]
#[test]
fn completion_list_outside_of_closed_arrow_function01() {
    let content = r#"// no a or b
/*1*/(a, b) => { }"#;
    let mut s = Session::new_for_test("completionListOutsideOfClosedArrowFunction01", content);
    fourslash::unsupported("VerifyCompletions"); // f.VerifyCompletions(t, "1", &fourslash.CompletionsExpectedList{
}
