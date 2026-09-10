use tsox_lsp::fourslash::{self, Session};


#[ignore = "unimplemented: fourslash.VerifyImportFixModuleSpecifiers"]
#[test]
fn auto_import_package_json_imports_caps_in_path1() {
    let content = r##"// @module: node18
// @Filename: /Dev/package.json
{
  "imports": {
    "#thing": "./src/something.js"
  }
}
// @Filename: /Dev/src/something.ts
export function something(name: string): any;
// @Filename: /Dev/a.ts
something/**/"##;
    let mut s = Session::new_for_test("autoImportPackageJsonImports_capsInPath1", content);
    fourslash::unsupported("VerifyImportFixModuleSpecifiers"); // f.VerifyImportFixModuleSpecifiers(t, "", []string{"#thing"}, nil /*preferences*/)
}
