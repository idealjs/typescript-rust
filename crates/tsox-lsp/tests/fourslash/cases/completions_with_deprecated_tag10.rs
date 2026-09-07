use tsox_lsp::fourslash::{self, Session};

#[ignore = "generator: t.Skip('Known failing fourslash test')"]
#[test]
fn completions_with_deprecated_tag10() {
    // TODO: t.Skip("Known failing fourslash test")
    let content = r#"// @Filename: /foo.ts
/** @deprecated foo */
export const foo = 0;
// @Filename: /index.ts
/**/"#;
    let mut s = Session::new(content);
    fourslash::unsupported("VerifyCompletions"); // f.VerifyCompletions(t, "", &fourslash.CompletionsExpectedList{
}
