use tsox_lsp::fourslash::{self, Session};


#[ignore = "unimplemented: fourslash.VerifyCompletions"]
#[test]
fn completion_after_newline2() {
    let content = r#"// @lib: es5
let foo = 5 as const /*1*/
/*2*/"#;
    let mut s = Session::new_for_test("completionAfterNewline2", content);
    fourslash::verify_completions_empty_at(&mut s, Some("1"));
    fourslash::unsupported("VerifyCompletions"); // f.VerifyCompletions(t, "2", &fourslash.CompletionsExpectedList{
}
