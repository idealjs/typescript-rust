use tsox_lsp::fourslash::{self, Session};

#[ignore = "generator: t.Skip('Known failing fourslash test')"]
#[test]
fn auto_import_package_json_imports_case_sensitivity() {
    // TODO: t.Skip("Known failing fourslash test")
    let content = r##"// @module: node18
// @allowImportingTsExtensions: true
// @Filename: /package.json
{
  "type": "module",
  "imports": {
    "#src/*": "./SRC/*"
  }
}
// @Filename: /src/add.ts
export function add(a: number, b: number) {}
// @Filename: /src/index.ts
add/*imports*/;"##;
    let mut s = Session::new(content);
    fourslash::unsupported("VerifyImportFixModuleSpecifiers"); // f.VerifyImportFixModuleSpecifiers(t, "imports", []string{"#src/add.ts"}, &lsutil.UserPreferences{Imp
}
