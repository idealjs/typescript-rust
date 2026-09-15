use tsox_lsp::fourslash::Session;


#[ignore = "go: t.Skip('Known failing fourslash test')"]
#[test]
fn completions_literal_from_inference_within_inferred_type3() {
    let content = r#"// @stableTypeOrdering: true
declare function test<T>(a: {
  [K in keyof T]: {
    b?: (keyof T)[];
  };
}): void;

test({
  foo: {},
  bar: {
    b: ["/*ts*/"],
  },
});

test({
  foo: {},
  bar: {
    b: [/*ts2*/],
  },
});"#;
    let _s = Session::new_for_test("completionsLiteralFromInferenceWithinInferredType3", content);
    // TODO: f.VerifyCompletions(t, []string{"ts"}, &fourslash.CompletionsExpectedList{
    // TODO: f.VerifyCompletions(t, []string{"ts2"}, &fourslash.CompletionsExpectedList{
}
