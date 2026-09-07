use tsox_lsp::fourslash::{self, Session};

#[ignore = "unimplemented: fourslash.VerifyCompletions"]
#[test]
fn completions_literal_directly_in_argument_with_nullable_constraint() {
    let content = r#"// @strict: true

declare function func<
  const T extends 'a' | 'b' | undefined = undefined,
>(arg?: T): string;

func('/*1*/');"#;
    let mut s = Session::new(content);
    fourslash::unsupported("VerifyCompletions"); // f.VerifyCompletions(t, []string{"1"}, &fourslash.CompletionsExpectedList{
}
