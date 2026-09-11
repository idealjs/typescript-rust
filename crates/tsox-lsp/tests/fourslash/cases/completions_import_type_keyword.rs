use tsox_lsp::fourslash::{self, Session};


#[test]
fn completions_import_type_keyword() {
    let content = r#"// @module: node18
// @Filename: /os.d.ts
declare module "os" {
  export function type(): string;
}
// @Filename: /index.ts
type/**/"#;
    let mut s = Session::new_for_test("completionsImportTypeKeyword", content);
    // TODO: f.VerifyCompletions(t, "", &fourslash.CompletionsExpectedList{
}
