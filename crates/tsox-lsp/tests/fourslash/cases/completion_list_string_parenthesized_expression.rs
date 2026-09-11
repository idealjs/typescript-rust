use tsox_lsp::fourslash::{self, Session};


#[ignore = "go: t.Skip('Known failing fourslash test')"]
#[test]
fn completion_list_string_parenthesized_expression() {
    let content = r#"const foo = {
    a: 1,
    b: 1,
    c: 1
}
const a = foo["[|/*1*/|]"];
const b = foo[("[|/*2*/|]")];
const c = foo[(("[|/*3*/|]"))];"#;
    let mut s = Session::new_for_test("completionListStringParenthesizedExpression", content);
    fourslash::go_to_marker(&mut s, "1");
    // TODO: f.VerifyCompletions(t, "1", &fourslash.CompletionsExpectedList{
    fourslash::go_to_marker(&mut s, "2");
    // TODO: f.VerifyCompletions(t, "2", &fourslash.CompletionsExpectedList{
    fourslash::go_to_marker(&mut s, "3");
    // TODO: f.VerifyCompletions(t, "3", &fourslash.CompletionsExpectedList{
}
