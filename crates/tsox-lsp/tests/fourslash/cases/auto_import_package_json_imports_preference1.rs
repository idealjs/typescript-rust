use tsox_lsp::fourslash::{self, Session};


#[ignore = "unimplemented: fourslash.VerifyImportFixModuleSpecifiers"]
#[test]
fn auto_import_package_json_imports_preference1() {
    let content = r##"// @module: node18
// @Filename: /package.json
{
  "imports": {
    "#*": "./src/*.ts"
  }
}
// @Filename: /src/a/b/c/something.ts
export function something(name: string): any;
// @Filename: /a.ts
something/**/"##;
    let mut s = Session::new_for_test("autoImportPackageJsonImportsPreference1", content);
    fourslash::unsupported("VerifyImportFixModuleSpecifiers"); // f.VerifyImportFixModuleSpecifiers(t, "", []string{"./src/a/b/c/something"}, &lsutil.UserPreferences{
}
