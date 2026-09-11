use tsox_lsp::fourslash::{self, Session};


#[ignore = "go: t.Skip('Known failing fourslash test')"]
#[test]
fn auto_import_package_root_path() {
    let content = r#"// @allowJs: true
// @Filename: /node_modules/pkg/package.json
{
    "name": "pkg",
    "version": "1.0.0",
    "main": "lib",
    "module": "lib"
 }
// @Filename: /node_modules/pkg/lib/index.js
export function foo() {};
// @Filename: /package.json
{
    "dependencies": {
       "pkg": "*"
    }
 }
// @Filename: /index.ts
foo/**/"#;
    let mut s = Session::new_for_test("autoImportPackageRootPath", content);
    // TODO: f.VerifyImportFixModuleSpecifiers(t, "", []string{"pkg"}, nil /*preferences*/)
}
