use tsox_lsp::fourslash::{self, Session};


#[test]
fn auto_import_package_json_imports_preference3() {
    let content = r##"// @module: node18
// @Filename: /package.json
{
  "imports": {
    "#*": "./src/*.ts"
  }
}
// @Filename: /src/a/b/c/something.ts
export function something(name: string): any;
// @Filename: /src/a/b/c/d.ts
something/**/"##;
    let mut s = Session::new_for_test("autoImportPackageJsonImportsPreference3", content);
    // TODO: f.VerifyImportFixModuleSpecifiers(t, "", []string{"#a/b/c/something"}, &lsutil.UserPreferences{Impor
}
