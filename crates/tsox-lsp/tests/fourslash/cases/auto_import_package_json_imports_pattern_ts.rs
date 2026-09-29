use tsox_lsp::fourslash::Session;


#[test]
fn auto_import_package_json_imports_pattern_ts() {
    let content = r##"// @module: node18
// @Filename: /package.json
{
  "imports": {
    "#*": "./src/*.ts"
  }
}
// @Filename: /src/something.ts
export function something(name: string): any;
// @Filename: /a.ts
something/**/"##;
    let _s = Session::new_for_test("autoImportPackageJsonImportsPattern_ts", content);
    // TODO: f.VerifyImportFixModuleSpecifiers(t, "", []string{"#something"}, nil /*preferences*/)
}
