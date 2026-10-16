use tsox_lsp::fourslash::Session;


#[test]
fn auto_import_package_json_imports_js() {
    let content = r##"// @module: node18
// @Filename: /package.json
{
  "imports": {
    "#thing": "./src/something.js"
  }
}
// @Filename: /src/something.ts
export function something(name: string): any;
// @Filename: /a.ts
something/**/"##;
    let _s = Session::new_for_test("autoImportPackageJsonImports_js", content);
    // TODO: f.VerifyImportFixModuleSpecifiers(t, "", []string{"#thing"}, nil /*preferences*/)
}
