use tsox_lsp::fourslash::Session;


#[test]
fn completion_satisfies_keyword() {
    let content = r#"const x = { a: 1 } /*1*/
function foo() {
    const x = { a: 1 } /*2*/
}"#;
    let _s = Session::new_for_test("completionSatisfiesKeyword", content);
    // TODO: f.VerifyCompletions(t, []string{"1", "2"}, &fourslash.CompletionsExpectedList{
}
