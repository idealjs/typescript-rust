use tsox_lsp::fourslash::{self, Session};


#[ignore = "unimplemented: fourslash.Configure"]
#[test]
fn auto_import_package_root_path_extension() {
    let content = r#"// @allowJs: true
// @Filename: /node_modules/pkg/package.json
{
    "name": "pkg",
    "version": "1.0.0",
    "main": "lib"
 }
// @Filename: /node_modules/pkg/lib/index.d.mts
export declare function foo(): any;
// @Filename: /package.json
{
    "dependencies": {
       "pkg": "*"
    }
 }
// @Filename: /index.ts
foo/**/"#;
    let mut s = Session::new_for_test("autoImportPackageRootPathExtension", content);
    fourslash::unsupported("Configure"); // f.Configure(t, lsutil.UserPreferences{AutoImportEntrypointDirectorySearch: core.TSTrue})
    fourslash::unsupported("VerifyImportFixModuleSpecifiers"); // f.VerifyImportFixModuleSpecifiers(t, "", []string{"pkg/lib/index.mjs"}, nil /*preferences*/)
}
