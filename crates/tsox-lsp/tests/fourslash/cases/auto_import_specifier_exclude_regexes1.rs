use tsox_lsp::fourslash::{self, Session};


#[ignore = "unimplemented: fourslash.VerifyImportFixModuleSpecifiers"]
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
    fourslash::unsupported("VerifyImportFixModuleSpecifiers"); // f.VerifyImportFixModuleSpecifiers(t, "", []string{"ambient", "ambient/utils"}, nil /*preferences*/)
    fourslash::unsupported("VerifyImportFixModuleSpecifiers"); // f.VerifyImportFixModuleSpecifiers(t, "", []string{"ambient"}, &lsutil.UserPreferences{AutoImportSpec
    fourslash::unsupported("VerifyImportFixModuleSpecifiers"); // f.VerifyImportFixModuleSpecifiers(t, "", []string{"ambient", "ambient/utils"}, &lsutil.UserPreferenc
    fourslash::unsupported("VerifyImportFixModuleSpecifiers"); // f.VerifyImportFixModuleSpecifiers(t, "", []string{"ambient"}, &lsutil.UserPreferences{AutoImportSpec
    fourslash::unsupported("VerifyImportFixModuleSpecifiers"); // f.VerifyImportFixModuleSpecifiers(t, "", []string{"ambient", "ambient/utils"}, &lsutil.UserPreferenc
    fourslash::unsupported("VerifyImportFixModuleSpecifiers"); // f.VerifyImportFixModuleSpecifiers(t, "", []string{"ambient"}, &lsutil.UserPreferences{AutoImportSpec
    fourslash::unsupported("VerifyImportFixModuleSpecifiers"); // f.VerifyImportFixModuleSpecifiers(t, "", []string{"ambient"}, &lsutil.UserPreferences{AutoImportSpec
    fourslash::unsupported("VerifyImportFixModuleSpecifiers"); // f.VerifyImportFixModuleSpecifiers(t, "", []string{"ambient"}, &lsutil.UserPreferences{AutoImportSpec
    fourslash::unsupported("VerifyImportFixModuleSpecifiers"); // f.VerifyImportFixModuleSpecifiers(t, "", []string{"ambient/utils"}, &lsutil.UserPreferences{AutoImpo
    fourslash::unsupported("VerifyImportFixModuleSpecifiers"); // f.VerifyImportFixModuleSpecifiers(t, "", []string{"ambient", "ambient/utils"}, &lsutil.UserPreferenc
}
