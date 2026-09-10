use tsox_lsp::fourslash::{self, Session};


#[ignore = "unimplemented: fourslash.VerifyCompletions"]
#[test]
fn completion_after_at_char() {
    let content = r#"// @lib: es5
@a/**/"#;
    let mut s = Session::new_for_test("completionAfterAtChar", content);
    fourslash::unsupported("VerifyCompletions"); // f.VerifyCompletions(t, "", &fourslash.CompletionsExpectedList{
}
