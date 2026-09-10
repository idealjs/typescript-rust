use tsox_lsp::fourslash::{self, Session};


#[ignore = "unimplemented: fourslash.VerifyImportFixModuleSpecifiers"]
#[test]
fn auto_import_allow_importing_ts_extensions_package_json_imports2() {
    let content = r##"// @Filename: /tsconfig.json
{
  "compilerOptions": {
    "module": "nodenext",
    "allowImportingTsExtensions": true,
    "rootDir": "src",
    "outDir": "dist",
    "declarationDir": "types",
    "declaration": true
  }
}
// @Filename: /package.json
{
  "name": "self",
  "type": "module",
  "imports": {
    "#*": {
      "types": "./types/*",
      "default": "./dist/*"
    }
  }
}
// @Filename: /src/add.ts
export function add(a: number, b: number) {}
// @Filename: /src/index.ts
add/*imports*/;
external/*exports*/;"##;
    let mut s = Session::new_for_test("autoImportAllowImportingTsExtensionsPackageJsonImports2", content);
    fourslash::unsupported("VerifyImportFixModuleSpecifiers"); // f.VerifyImportFixModuleSpecifiers(t, "imports", []string{"#add.js"}, nil /*preferences*/)
}
