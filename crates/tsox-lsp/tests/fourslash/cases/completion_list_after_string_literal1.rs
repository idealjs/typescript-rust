use tsox_lsp::fourslash::{self, Session};


#[ignore = "unimplemented: fourslash.VerifyCompletions"]
#[test]
fn completion_list_after_string_literal1() {
    let content = r#"// @lib: es5
"a"./**/"#;
    let mut s = Session::new_for_test("completionListAfterStringLiteral1", content);
    fourslash::unsupported("VerifyCompletions"); // f.VerifyCompletions(t, "", &fourslash.CompletionsExpectedList{
}
