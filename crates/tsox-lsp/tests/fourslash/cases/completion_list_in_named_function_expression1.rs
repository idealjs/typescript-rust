use tsox_lsp::fourslash::{self, Session};

#[ignore = "generator: t.Skip('Known failing fourslash test')"]
#[test]
fn completion_list_in_named_function_expression1() {
    // TODO: t.Skip("Known failing fourslash test")
    let content = r#"var x = function foo() {
   /*1*/
}"#;
    let mut s = Session::new(content);
    fourslash::unsupported("VerifyCompletions"); // f.VerifyCompletions(t, "1", &fourslash.CompletionsExpectedList{
}
