use tsox_lsp::fourslash::{self, Session};


#[test]
fn import_statement_completions_no_snippet() {
    let content = r#"// @Filename: /mod.ts
export const foo = 0;
// @Filename: /index0.ts
[|import f/**/|]"#;
    let mut s = Session::new_for_test("importStatementCompletions_noSnippet", content);
    fourslash::go_to_marker(&mut s, "");
    // TODO: f.VerifyCompletions(t, "", &fourslash.CompletionsExpectedList{
}
