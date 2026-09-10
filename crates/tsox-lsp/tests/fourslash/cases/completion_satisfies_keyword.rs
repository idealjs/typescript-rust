use tsox_lsp::fourslash::{self, Session};


#[ignore = "unimplemented: fourslash.VerifyCompletions"]
#[test]
fn completion_satisfies_keyword() {
    let content = r#"const x = { a: 1 } /*1*/
function foo() {
    const x = { a: 1 } /*2*/
}"#;
    let mut s = Session::new_for_test("completionSatisfiesKeyword", content);
    fourslash::unsupported("VerifyCompletions"); // f.VerifyCompletions(t, []string{"1", "2"}, &fourslash.CompletionsExpectedList{
}
