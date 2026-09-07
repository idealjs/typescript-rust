use tsox_lsp::fourslash::{self, Session};

#[ignore = "unimplemented: fourslash.VerifyCompletions"]
#[test]
fn completions_properties_priorities() {
    let content = r#"// @strict: true
interface I {
  B?: number;
  a: number;
  c?: string;
  d: string
}
const foo = {
  a: 1,
  B: 2
}
const i: I = {
  ...foo,
  /*a*/
}"#;
    let mut s = Session::new(content);
    fourslash::unsupported("VerifyCompletions"); // f.VerifyCompletions(t, []string{"a"}, &fourslash.CompletionsExpectedList{
}
