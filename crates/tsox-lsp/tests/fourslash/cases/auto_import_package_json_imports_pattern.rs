use tsox_lsp::fourslash::{self, Session};


#[test]
fn auto_import_package_json_imports_pattern() {
    let content = r##"// @module: node18
// @Filename: /package.json
{
  "imports": {
    "#*": "./src/*"
  }
}
// @Filename: /src/something.ts
export function something(name: string): any;
// @Filename: /a.ts
something/**/"##;
    let mut s = Session::new_for_test("autoImportPackageJsonImportsPattern", content);
    // TODO: f.VerifyImportFixModuleSpecifiers(t, "", []string{"#something.js"}, nil /*preferences*/)
}
