use tsox_lsp::fourslash::{self, Session};

#[ignore = "unimplemented: fourslash.VerifyCompletions"]
#[test]
fn import_statement_completions_no_snippet() {
    let content = r#"// @Filename: /mod.ts
export const foo = 0;
// @Filename: /index0.ts
[|import f/**/|]"#;
    let mut s = Session::new(content);
    fourslash::unsupported("VerifyCompletions"); // f.VerifyCompletions(t, "", &fourslash.CompletionsExpectedList{
}
