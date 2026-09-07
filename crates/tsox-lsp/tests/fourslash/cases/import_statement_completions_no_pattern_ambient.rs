use tsox_lsp::fourslash::{self, Session};

#[ignore = "unimplemented: fourslash.VerifyCompletions"]
#[test]
fn import_statement_completions_no_pattern_ambient() {
    let content = r#"// @Filename: /types.d.ts
declare module "*.css" {
  const styles: any;
  export = styles;
}
// @Filename: /index.ts
import style/**/"#;
    let mut s = Session::new_with_capabilities(content, None);
    fourslash::unsupported("VerifyCompletions"); // f.VerifyCompletions(t, "", &fourslash.CompletionsExpectedList{
}
