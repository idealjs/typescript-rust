use tsox_lsp::fourslash::{self, Session};


#[ignore = "generator: t.Skip('Known failing fourslash test')"]
#[test]
fn auto_import_file_exclude_patterns3() {
    // TODO: t.Skip("Known failing fourslash test")
    let content = r#"// @lib: es5
// @module: commonjs
// @Filename: /ambient1.d.ts
declare module "foo" {
   export const x = 1;
}
// @Filename: /ambient2.d.ts
declare module "foo" {
   export const y = 2;
}
// @Filename: /index.ts
/**/"#;
    let mut s = Session::new_for_test("autoImportFileExcludePatterns3", content);
    fourslash::unsupported("VerifyCompletions"); // f.VerifyCompletions(t, "", &fourslash.CompletionsExpectedList{
    fourslash::unsupported("VerifyCompletions"); // f.VerifyCompletions(t, "", &fourslash.CompletionsExpectedList{
}
