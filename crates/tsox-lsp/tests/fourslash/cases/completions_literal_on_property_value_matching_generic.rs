use tsox_lsp::fourslash::{self, Session};


#[ignore = "unimplemented: fourslash.VerifyCompletions"]
#[test]
fn completions_literal_on_property_value_matching_generic() {
    let content = r#"// @Filename: /a.tsx
declare function bar1<P extends "" | "bar" | "baz">(p: { type: P }): void;

bar1({ type: "/*ts*/" })
"#;
    let mut s = Session::new_for_test("completionsLiteralOnPropertyValueMatchingGeneric", content);
    fourslash::unsupported("VerifyCompletions"); // f.VerifyCompletions(t, []string{"ts"}, &fourslash.CompletionsExpectedList{
}
