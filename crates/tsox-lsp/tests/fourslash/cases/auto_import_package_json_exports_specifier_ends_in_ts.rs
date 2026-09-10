use tsox_lsp::fourslash::{self, Session};


#[ignore = "unimplemented: fourslash.VerifyImportFixModuleSpecifiers"]
#[test]
fn auto_import_package_json_exports_specifier_ends_in_ts() {
    let content = r#"// @module: node18
// @Filename: /node_modules/pkg/package.json
{
    "name": "pkg",
    "version": "1.0.0",
    "exports": {
      "./something.ts": "./a.js"
    }
 }
// @Filename: /node_modules/pkg/a.d.ts
export function foo(): void;
// @Filename: /package.json
{
    "dependencies": {
       "pkg": "*"
    }
 }
// @Filename: /index.ts
foo/**/"#;
    let mut s = Session::new_for_test("autoImportPackageJsonExportsSpecifierEndsInTs", content);
    fourslash::unsupported("VerifyImportFixModuleSpecifiers"); // f.VerifyImportFixModuleSpecifiers(t, "", []string{"pkg/something.ts"}, nil /*preferences*/)
}
