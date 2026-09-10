use tsox_lsp::fourslash::{self, Session};


#[ignore = "unimplemented: fourslash.VerifyCompletions"]
#[test]
fn completions_literal_directly_in_rest_constrained_to_tuple_type() {
    let content = r#"// @strict: true

interface Func {
  <Key extends "a" | "b">(
    ...args:
      | [key: Key, options?: any]
      | [key: Key, defaultValue: string, options?: any]
  ): string;
}

declare const func: Func;

func("/*1*/");"#;
    let mut s = Session::new_for_test("completionsLiteralDirectlyInRestConstrainedToTupleType", content);
    fourslash::unsupported("VerifyCompletions"); // f.VerifyCompletions(t, []string{"1"}, &fourslash.CompletionsExpectedList{
}
