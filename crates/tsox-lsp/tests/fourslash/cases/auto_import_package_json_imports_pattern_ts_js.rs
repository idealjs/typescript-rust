use tsox_lsp::fourslash::{self, Session};


#[ignore = "unimplemented: fourslash.VerifyImportFixModuleSpecifiers"]
#[test]
fn auto_import_package_json_imports_pattern_ts_js() {
    let content = r##"// @module: node18
// @Filename: /package.json
{
  "imports": {
    "#*.ts": "./src/*.js"
  }
}
// @Filename: /src/something.ts
export function something(name: string): any;
// @Filename: /a.ts
something/**/"##;
    let mut s = Session::new_for_test("autoImportPackageJsonImportsPattern_ts_js", content);
    fourslash::unsupported("VerifyImportFixModuleSpecifiers"); // f.VerifyImportFixModuleSpecifiers(t, "", []string{"#something.ts"}, nil /*preferences*/)
}
