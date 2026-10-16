use tsox_lsp::fourslash::Session;


#[test]
fn auto_import_package_json_imports_preference2() {
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
    let _s = Session::new_for_test("autoImportPackageJsonImportsPreference2", content);
    // TODO: f.VerifyImportFixModuleSpecifiers(t, "", []string{"./src/a/b/c/something"}, &lsutil.UserPreferences{
}
