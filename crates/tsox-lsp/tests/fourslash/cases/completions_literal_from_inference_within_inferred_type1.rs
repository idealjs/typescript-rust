use tsox_lsp::fourslash::Session;


#[test]
fn completions_literal_from_inference_within_inferred_type1() {
    let content = r#"// @stableTypeOrdering: true
// @Filename: /a.tsx
declare function test<T>(a: {
  [K in keyof T]: {
    b?: keyof T;
  };
}): void;

test({
  foo: {},
  bar: {
    b: "/*ts*/",
  },
});

test({
  foo: {},
  bar: {
    b: /*ts2*/,
  },
});"#;
    let _s = Session::new_for_test("completionsLiteralFromInferenceWithinInferredType1", content);
    // TODO: f.VerifyCompletions(t, []string{"ts"}, &fourslash.CompletionsExpectedList{
    // TODO: f.VerifyCompletions(t, []string{"ts2"}, &fourslash.CompletionsExpectedList{
}
