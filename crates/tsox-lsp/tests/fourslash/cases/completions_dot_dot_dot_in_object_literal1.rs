use tsox_lsp::fourslash::{self, Session};


#[ignore = "unimplemented: fourslash.VerifyCompletions"]
#[test]
fn completions_dot_dot_dot_in_object_literal1() {
    let content = r#"// https://github.com/microsoft/TypeScript/issues/57540

const foo = { b: 100 };

const bar: {
  a: number;
  b: number;
} = {
  a: 42,
  .../*1*/
};"#;
    let mut s = Session::new_for_test("completionsDotDotDotInObjectLiteral1", content);
    fourslash::unsupported("VerifyCompletions"); // f.VerifyCompletions(t, "1", &fourslash.CompletionsExpectedList{
}
