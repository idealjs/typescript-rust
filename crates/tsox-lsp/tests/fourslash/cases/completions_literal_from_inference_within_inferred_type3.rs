use tsox_lsp::fourslash::{self, Session};


#[ignore = "generator: t.Skip('Known failing fourslash test')"]
#[test]
fn completions_literal_from_inference_within_inferred_type3() {
    // TODO: t.Skip("Known failing fourslash test")
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
    let mut s = Session::new_for_test("completionsLiteralFromInferenceWithinInferredType3", content);
    fourslash::unsupported("VerifyCompletions"); // f.VerifyCompletions(t, []string{"ts"}, &fourslash.CompletionsExpectedList{
    fourslash::unsupported("VerifyCompletions"); // f.VerifyCompletions(t, []string{"ts2"}, &fourslash.CompletionsExpectedList{
}
