use tsox_lsp::fourslash::{self, Session};


#[test]
fn auto_import_package_json_imports_ts() {
    let content = r##"// @module: node18
// @Filename: /package.json
{
  "imports": {
    "#thing": "./src/something.ts"
  }
}
// @Filename: /src/something.ts
export function something(name: string): any;
// @Filename: /a.ts
something/**/"##;
    let mut s = Session::new_for_test("autoImportPackageJsonImports_ts", content);
    // TODO: f.VerifyImportFixModuleSpecifiers(t, "", []string{"#thing"}, nil /*preferences*/)
}
