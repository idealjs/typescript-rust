use tsox_lsp::fourslash::{self, Session};


#[ignore = "unimplemented: fourslash.Configure"]
#[test]
fn auto_import_no_package_json_nodenext() {
    let content = r#"// @lib: es5
// @module: node18
// @Filename: /node_modules/lit/index.d.cts
export declare function customElement(name: string): any;
// @Filename: /a.ts
customElement/**/"#;
    let mut s = Session::new_for_test("autoImportNoPackageJson_nodenext", content);
    fourslash::unsupported("Configure"); // f.Configure(t, lsutil.UserPreferences{AutoImportEntrypointDirectorySearch: core.TSTrue})
    fourslash::unsupported("VerifyImportFixModuleSpecifiers"); // f.VerifyImportFixModuleSpecifiers(t, "", []string{"lit/index.cjs"}, nil /*preferences*/)
}
