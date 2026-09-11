use tsox_lsp::fourslash::{self, Session};


#[test]
fn auto_import_package_json_imports_conditions() {
    let content = r##"// @module: node18
// @Filename: /package.json
{
  "imports": {
    "#thing": {
        "types": { "import": "./types-esm/thing.d.mts", "require": "./types/thing.d.ts" },
        "default": { "import": "./esm/thing.mjs", "require": "./dist/thing.js" }
     }
  }
}
// @Filename: /src/.ts
something/*a*/
// @Filename: /types/thing.d.ts
export function something(name: string): any;"##;
    let mut s = Session::new_for_test("autoImportPackageJsonImportsConditions", content);
    // TODO: f.VerifyImportFixModuleSpecifiers(t, "a", []string{"#thing"}, nil /*preferences*/)
}
