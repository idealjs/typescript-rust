use tsox_lsp::fourslash::Session;


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
    let _s = Session::new_for_test("completionsLiteralDirectlyInRestConstrainedToTupleType", content);
    // TODO: f.VerifyCompletions(t, []string{"1"}, &fourslash.CompletionsExpectedList{
}
