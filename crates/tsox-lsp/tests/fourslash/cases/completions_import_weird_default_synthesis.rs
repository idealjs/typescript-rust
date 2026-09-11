use tsox_lsp::fourslash::{self, Session};


#[test]
fn completions_import_weird_default_synthesis() {
    let content = r#"// @module: commonjs
// @esModuleInterop: false
// @allowSyntheticDefaultImports: false
// @Filename: /collection.ts
class Collection {
  public static readonly default: typeof Collection = Collection;
}
export = Collection as typeof Collection & { default: typeof Collection };
// @Filename: /index.ts
Colle/**/"#;
    let mut s = Session::new_for_test("completionsImport_weirdDefaultSynthesis", content);
    // TODO: f.VerifyApplyCodeActionFromCompletion(t, new(""), &fourslash.ApplyCodeActionFromCompletionOptions{
}
