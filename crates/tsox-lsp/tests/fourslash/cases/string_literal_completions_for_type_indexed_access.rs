use tsox_lsp::fourslash::{self, Session};

#[ignore = "unimplemented: fourslash.VerifyCompletions"]
#[test]
fn string_literal_completions_for_type_indexed_access() {
    let content = r#"type Foo = { a: string; b: number; c: boolean; };
type A = Foo["/*1*/"];
type AorB = Foo["a" | "/*2*/"];"#;
    let mut s = Session::new(content);
    fourslash::unsupported("VerifyCompletions"); // f.VerifyCompletions(t, []string{"1"}, &fourslash.CompletionsExpectedList{
    fourslash::unsupported("VerifyCompletions"); // f.VerifyCompletions(t, []string{"2"}, &fourslash.CompletionsExpectedList{
}
