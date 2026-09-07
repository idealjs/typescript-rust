use tsox_lsp::fourslash::{self, Session};

#[ignore = "unimplemented: fourslash.VerifyCompletions"]
#[test]
fn completions_conditional_member() {
    let content = r#"declare function f<T extends string>(
  p: { a: T extends 'foo' ? { x: string } : { y: string } }
): void;

f<'foo'>({ a: { /*1*/ } });
f<string>({ a: { /*2*/ } });"#;
    let mut s = Session::new(content);
    fourslash::unsupported("VerifyCompletions"); // f.VerifyCompletions(t, "1", &fourslash.CompletionsExpectedList{
    fourslash::unsupported("VerifyCompletions"); // f.VerifyCompletions(t, "2", &fourslash.CompletionsExpectedList{
}
