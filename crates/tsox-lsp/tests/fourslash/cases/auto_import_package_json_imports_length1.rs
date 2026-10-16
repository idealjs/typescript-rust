use tsox_lsp::fourslash::Session;


#[test]
fn auto_import_package_json_imports_length1() {
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
    let _s = Session::new_for_test("autoImportPackageJsonImportsLength1", content);
    // TODO: f.VerifyImportFixModuleSpecifiers(t, "", []string{"./something"}, nil /*preferences*/)
}
