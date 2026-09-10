use tsox_lsp::fourslash::{self, Session};


#[ignore = "generator: t.Skip('Known failing fourslash test')"]
#[test]
fn auto_import_allow_importing_ts_extensions_package_json_imports1() {
    // TODO: t.Skip("Known failing fourslash test")
    let content = r##"// @lib: es5
// @module: node18
// @allowImportingTsExtensions: true
// @Filename: /node_modules/pkg/package.json
{
  "name": "pkg",
  "type": "module",
  "exports": {
    "./*": {
      "types": "./types/*",
      "default": "./dist/*"
    }
  }
}
// @Filename: /node_modules/pkg/types/external.d.ts
export declare function external(name: string): any;
// @Filename: /package.json
{
  "name": "self",
  "type": "module",
  "imports": {
    "#*": "./src/*"
  },
  "dependencies": {
    "pkg": "*"
  }
}
// @Filename: /src/add.ts
export function add(a: number, b: number) {}
// @Filename: /src/index.ts
add/*imports*/;
external/*exports*/;"##;
    let mut s = Session::new_for_test("autoImportAllowImportingTsExtensionsPackageJsonImports1", content);
    fourslash::unsupported("VerifyImportFixModuleSpecifiers"); // f.VerifyImportFixModuleSpecifiers(t, "imports", []string{"#add.ts"}, nil /*preferences*/)
    fourslash::unsupported("VerifyImportFixModuleSpecifiers"); // f.VerifyImportFixModuleSpecifiers(t, "exports", []string{"pkg/external.js"}, nil /*preferences*/)
}
