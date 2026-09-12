use tsox_lsp::fourslash::{self, Session};


#[test]
fn auto_import_specifier_exclude_regexes1() {
    let content = r#"// @module: preserve
// @Filename: /node_modules/lib/index.d.ts
declare module "ambient" {
    export const x: number;
}
declare module "ambient/utils" {
   export const x: number;
}
// @Filename: /index.ts
x/**/"#;
    let mut s = Session::new_for_test("autoImportSpecifierExcludeRegexes1", content);
    // TODO: f.VerifyImportFixModuleSpecifiers(t, "", []string{"ambient", "ambient/utils"}, nil /*preferences*/)
    // TODO: f.VerifyImportFixModuleSpecifiers(t, "", []string{"ambient"}, &lsutil.UserPreferences{AutoImportSpec
    // TODO: f.VerifyImportFixModuleSpecifiers(t, "", []string{"ambient", "ambient/utils"}, &lsutil.UserPreferenc
    // TODO: f.VerifyImportFixModuleSpecifiers(t, "", []string{"ambient"}, &lsutil.UserPreferences{AutoImportSpec
    // TODO: f.VerifyImportFixModuleSpecifiers(t, "", []string{"ambient", "ambient/utils"}, &lsutil.UserPreferenc
    // TODO: f.VerifyImportFixModuleSpecifiers(t, "", []string{"ambient"}, &lsutil.UserPreferences{AutoImportSpec
    // TODO: f.VerifyImportFixModuleSpecifiers(t, "", []string{"ambient"}, &lsutil.UserPreferences{AutoImportSpec
    // TODO: f.VerifyImportFixModuleSpecifiers(t, "", []string{"ambient"}, &lsutil.UserPreferences{AutoImportSpec
    // TODO: f.VerifyImportFixModuleSpecifiers(t, "", []string{"ambient/utils"}, &lsutil.UserPreferences{AutoImpo
    // TODO: f.VerifyImportFixModuleSpecifiers(t, "", []string{"ambient", "ambient/utils"}, &lsutil.UserPreferenc
    fourslash::go_to_marker(&mut s, "");
    // TODO: f.VerifyCompletions(t, "", &fourslash.CompletionsExpectedList{
    fourslash::go_to_marker(&mut s, "");
    // TODO: f.VerifyCompletions(t, "", &fourslash.CompletionsExpectedList{
}
