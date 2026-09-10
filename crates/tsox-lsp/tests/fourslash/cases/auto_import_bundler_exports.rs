use tsox_lsp::fourslash::{self, Session};


#[ignore = "unimplemented: fourslash.VerifyImportFixModuleSpecifiers"]
#[test]
fn auto_import_bundler_exports() {
    let content = r#"// @module: esnext
// @moduleResolution: bundler
// @Filename: /node_modules/dep/package.json
{
  "name": "dep",
  "version": "1.0.0",
  "exports": {
    ".": "./dist/index.js"
  }
}
// @Filename: /node_modules/dep/dist/index.d.ts
export const dep: number;
// @Filename: /index.ts
dep/**/"#;
    let mut s = Session::new_for_test("autoImportBundlerExports", content);
    fourslash::unsupported("VerifyImportFixModuleSpecifiers"); // f.VerifyImportFixModuleSpecifiers(t, "", []string{"dep"}, nil /*preferences*/)
}
