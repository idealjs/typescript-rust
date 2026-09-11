use tsox_lsp::fourslash::{self, Session};


#[ignore = "go: t.Skip()"]
#[test]
fn auto_import_package_root_path_type_module() {
    let content = r#"// @allowJs: true
// @Filename: /node_modules/pkg/package.json
{
    "name": "pkg",
    "version": "1.0.0",
    "main": "lib",
    "type": "module"
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
    let mut s = Session::new_for_test("autoImportPackageRootPathTypeModule", content);
    // TODO: f.VerifyImportFixModuleSpecifiers(t, "", []string{"pkg"}, nil /*preferences*/)
}
