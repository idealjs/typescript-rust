use tsox_lsp::fourslash::{self, Session};


#[test]
fn completion_for_string_literal2() {
    let content = r#"var o = {
    foo() { },
    bar: 0,
    "some other name": 1
};
declare const p: { [s: string]: any, a: number };

o["[|/*1*/bar|]"];
o["/*2*/ ;
p["[|/*3*/|]"];"#;
    let mut s = Session::new_for_test("completionForStringLiteral2", content);
    fourslash::go_to_marker(&mut s, "1");
    // TODO: f.VerifyCompletions(t, "1", &fourslash.CompletionsExpectedList{
    fourslash::verify_completions_exact_at(&mut s, Some("2"), &["bar", "foo", "some other name"]);
    fourslash::go_to_marker(&mut s, "3");
    // TODO: f.VerifyCompletions(t, "3", &fourslash.CompletionsExpectedList{
}
