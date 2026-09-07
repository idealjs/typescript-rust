use tsox_lsp::fourslash::{self, Session};

#[ignore = "generator: t.Skip('Known failing fourslash test')"]
#[test]
fn completion_for_string_literal() {
    // TODO: t.Skip("Known failing fourslash test")
    let content = r#"type Options = "Option 1" | "Option 2" | "Option 3";
var x: Options = "[|/*1*/Option 3|]";

function f(a: Options) { };
f("/*2*/"#;
    let mut s = Session::new(content);
    fourslash::unsupported("VerifyCompletions"); // f.VerifyCompletions(t, "1", &fourslash.CompletionsExpectedList{
    fourslash::unsupported("VerifyCompletions"); // f.VerifyCompletions(t, "2", &fourslash.CompletionsExpectedList{
}
