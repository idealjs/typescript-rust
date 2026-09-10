use tsox_lsp::fourslash::{self, Session};


#[ignore = "unimplemented: fourslash.VerifyCompletions"]
#[test]
fn auto_import_type_only_preferred1() {
    let content = r#"// @verbatimModuleSyntax: true
// @module: esnext
// @moduleResolution: bundler
// @Filename: /ts.d.ts
declare namespace ts {
  interface SourceFile {
      text: string;
  }
  function createSourceFile(): SourceFile;
}
export = ts;
// @Filename: /types.ts
export interface VFS {
  getSourceFile(path: string): ts/**/
}"#;
    let mut s = Session::new_for_test("autoImportTypeOnlyPreferred1", content);
    fourslash::unsupported("VerifyCompletions"); // f.VerifyCompletions(t, "", &fourslash.CompletionsExpectedList{
}
