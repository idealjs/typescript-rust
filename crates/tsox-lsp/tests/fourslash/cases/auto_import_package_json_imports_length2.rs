use tsox_lsp::fourslash::Session;


#[test]
fn auto_import_package_json_imports_length2() {
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
    let _s = Session::new_for_test("autoImportPackageJsonImportsLength2", content);
    // TODO: f.VerifyImportFixModuleSpecifiers(t, "", []string{"#a/b/c/something"}, nil /*preferences*/)
}
