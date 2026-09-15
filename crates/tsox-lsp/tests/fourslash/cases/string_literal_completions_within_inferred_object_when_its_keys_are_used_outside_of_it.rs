use tsox_lsp::fourslash::Session;


#[test]
fn string_literal_completions_within_inferred_object_when_its_keys_are_used_outside_of_it() {
    let content = r#"// @strict: true
declare function createMachine<T>(config: {
  initial: keyof T;
  states: {
    [K in keyof T]: {
      on?: Record<string, keyof T>;
    };
  };
}): void;

createMachine({
  initial: "a",
  states: {
    a: {
      on: {
        NEXT: "/*1*/",
      },
    },
    b: {
      on: {
        NEXT: "/*2*/",
      },
    },
  },
});"#;
    let _s = Session::new_for_test("stringLiteralCompletionsWithinInferredObjectWhenItsKeysAreUsedOutsideOfIt", content);
    // TODO: f.VerifyCompletions(t, []string{"1", "2"}, &fourslash.CompletionsExpectedList{
}
