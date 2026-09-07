use tsox_lsp::fourslash::{self, Session};

#[ignore = "unimplemented: fourslash.VerifyCompletions"]
#[test]
fn auto_import_completions_for_arbitrary_non_identifier_exports() {
    let content = r#"
// @module: esnext
// @Filename: /a.ts
const foo = 0;
export { foo as "foo-bar" };
export const fooBar = 1;

// @Filename: /b.ts
foo/**/
"#;
    let mut s = Session::new(content);
    fourslash::unsupported("VerifyCompletions"); // f.VerifyCompletions(t, "", &fourslash.CompletionsExpectedList{
}
