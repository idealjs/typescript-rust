use tsox_lsp::fourslash::{self, Session};


#[ignore = "unimplemented: fourslash.VerifyCompletions"]
#[test]
fn completions_literal_matching_generic_signature() {
    let content = r#"// @Filename: /a.tsx
declare function bar1<P extends "" | "bar" | "baz">(p: P): void;

bar1("/*ts*/")
"#;
    let mut s = Session::new_for_test("completionsLiteralMatchingGenericSignature", content);
    fourslash::unsupported("VerifyCompletions"); // f.VerifyCompletions(t, []string{"ts"}, &fourslash.CompletionsExpectedList{
}
