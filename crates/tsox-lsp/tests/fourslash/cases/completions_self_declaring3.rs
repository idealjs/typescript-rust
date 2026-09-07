use tsox_lsp::fourslash::{self, Session};

#[ignore = "unimplemented: fourslash.VerifyCompletions"]
#[test]
fn completions_self_declaring3() {
    let content = r#"function f<T extends { x: number }>(p: T & (T extends { hello: string } ? { goodbye: number } : {})) {}
f({ x/*x*/: 0, hello/*hello*/: "", goodbye/*goodbye*/: 0, abc/*abc*/: "" })"#;
    let mut s = Session::new(content);
    fourslash::unsupported("VerifyCompletions"); // f.VerifyCompletions(t, "x", &fourslash.CompletionsExpectedList{
    fourslash::unsupported("VerifyCompletions"); // f.VerifyCompletions(t, "hello", &fourslash.CompletionsExpectedList{
    fourslash::unsupported("VerifyCompletions"); // f.VerifyCompletions(t, "goodbye", &fourslash.CompletionsExpectedList{
    fourslash::unsupported("VerifyCompletions"); // f.VerifyCompletions(t, "abc", &fourslash.CompletionsExpectedList{
}
