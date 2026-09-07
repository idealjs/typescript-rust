use tsox_lsp::fourslash::{self, Session};

#[ignore = "unimplemented: fourslash.VerifyCompletions"]
#[test]
fn completions_import_type_keyword() {
    let content = r#"// @module: node18
// @Filename: /os.d.ts
declare module "os" {
  export function type(): string;
}
// @Filename: /index.ts
type/**/"#;
    let mut s = Session::new(content);
    fourslash::unsupported("VerifyCompletions"); // f.VerifyCompletions(t, "", &fourslash.CompletionsExpectedList{
}
