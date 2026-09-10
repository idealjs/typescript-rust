use tsox_lsp::fourslash::{self, Session};


#[ignore = "generator: t.Skip('Known failing fourslash test')"]
#[test]
fn completion_list_string_parenthesized_expression() {
    // TODO: t.Skip("Known failing fourslash test")
    let content = r#"const foo = {
    a: 1,
    b: 1,
    c: 1
}
const a = foo["[|/*1*/|]"];
const b = foo[("[|/*2*/|]")];
const c = foo[(("[|/*3*/|]"))];"#;
    let mut s = Session::new_for_test("completionListStringParenthesizedExpression", content);
    fourslash::unsupported("VerifyCompletions"); // f.VerifyCompletions(t, "1", &fourslash.CompletionsExpectedList{
    fourslash::unsupported("VerifyCompletions"); // f.VerifyCompletions(t, "2", &fourslash.CompletionsExpectedList{
    fourslash::unsupported("VerifyCompletions"); // f.VerifyCompletions(t, "3", &fourslash.CompletionsExpectedList{
}
